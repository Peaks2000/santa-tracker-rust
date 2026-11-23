//! On-demand scenery tiles. CPU work is off-thread; ready/fallback pieces never overlap.
use macroquad::prelude::*;
use santa_tracker::svg_tiles::{self, Plan};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Layer {
    Stars,
    Mountains,
}
impl Layer {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "stars-tile.svg" => Some(Self::Stars),
            "bg-mountains.svg" => Some(Self::Mountains),
            _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Stars => "stars-tile.svg",
            Self::Mountains => "bg-mountains.svg",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Key {
    layer: Layer,
    plan: Plan,
    at: [u32; 2],
}
type ResultTile = (Key, Result<santa_tracker::svg_crop::Image, String>);
struct Worker {
    tx: mpsc::SyncSender<Key>,
    rx: mpsc::Receiver<ResultTile>,
}
fn worker(root: PathBuf) -> Worker {
    let (tx, tasks) = mpsc::sync_channel::<Key>(16);
    let (results, rx) = mpsc::sync_channel(2);
    std::thread::spawn(move || {
        let mut trees: HashMap<Layer, Result<svg_tiles::Rasterizer, String>> = HashMap::new();
        while let Ok(key) = tasks.recv() {
            let tree = trees.entry(key.layer).or_insert_with(|| {
                let path = root.join("scenes/glider/img").join(key.layer.name());
                let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                resvg::usvg::Tree::from_data(
                    &bytes,
                    &resvg::usvg::Options {
                        resources_dir: path.parent().map(|p| p.to_owned()),
                        ..Default::default()
                    },
                )
                .map(svg_tiles::Rasterizer::new)
                .map_err(|e| e.to_string())
            });
            let result = match tree {
                Ok(rasterizer) => rasterizer.rasterize(key.plan, key.at),
                Err(error) => Err(error.clone()),
            };
            if results.send((key, result)).is_err() {
                return;
            }
        }
    });
    Worker { tx, rx }
}
struct Cached {
    texture: Texture2D,
    bytes: u64,
    last_seen: u64,
}
#[derive(Default)]
pub struct Scenery {
    worker: Option<Worker>,
    cache: HashMap<Key, Cached>,
    pending: HashSet<Key>,
    failed: HashSet<Layer>,
    frame: u64,
    ready: bool,
}
impl Scenery {
    pub fn begin(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.ready = true;
        // One bounded tile upload per draw frame, alongside existing asset uploads.
        let result = self.worker.as_ref().map(|w| w.rx.try_recv());
        match result {
            Some(Ok((key, result))) => {
                self.pending.remove(&key);
                match result {
                    Ok(image) => {
                        let texture = Texture2D::from_image(&Image {
                            bytes: image.rgba,
                            width: image.width as u16,
                            height: image.height as u16,
                        });
                        texture.set_filter(FilterMode::Linear);
                        self.cache.insert(
                            key,
                            Cached {
                                texture,
                                bytes: image.width as u64 * image.height as u64 * 4,
                                last_seen: 0,
                            },
                        );
                    }
                    Err(error) => {
                        eprintln!("{} tile: {error}", key.layer.name());
                        self.failed.insert(key.layer);
                    }
                }
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.worker = None;
                self.pending.clear();
            }
            _ => {}
        }
    }
    pub fn draw(
        &mut self,
        root: &std::path::Path,
        name: &str,
        natural: Vec2,
        texture: &Texture2D,
        rects: &[Rect],
        material: &Material,
    ) {
        let Some(first) = rects.first() else {
            return;
        };
        let layer = Layer::from_name(name).expect("wide Glider scenery layer");
        let scale = ((first.w * screen_dpi_scale() / natural.x)
            .max(first.h * screen_dpi_scale() / natural.y))
        .ceil()
        .max(1.) as u32;
        let plan = Plan::new(
            [natural.x as u32, natural.y as u32],
            scale,
            svg_tiles::TILE_SIZE,
        );
        gl_use_material(material);
        if scale == 1 || natural.x.fract() != 0. || natural.y.fract() != 0. || plan.is_err() {
            if natural.x.fract() != 0. || natural.y.fract() != 0. || plan.is_err() {
                self.ready = false;
            }
            for r in rects {
                draw_texture_ex(
                    texture,
                    r.x,
                    r.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(r.size()),
                        ..Default::default()
                    },
                );
            }
            gl_use_default_material();
            return;
        }
        let plan = plan.unwrap();
        let mut pieces = vec![];
        for r in rects {
            let image = [r.x as f64, r.y as f64, r.w as f64, r.h as f64];
            for at in plan.visible(
                image,
                [0., 0., screen_width() as f64, screen_height() as f64],
            ) {
                pieces.push((
                    Key { layer, plan, at },
                    plan.bounds(at, image).unwrap(),
                    plan.core(at).unwrap(),
                ));
            }
        }
        // Visible requests come first. One neighbouring strip at either edge
        // warms the slow source pan before it enters the viewport.
        let mut wants: Vec<_> = pieces.iter().map(|(key, _, _)| *key).collect();
        for (key, _, _) in &pieces {
            if key.at[0] > 0 {
                wants.push(Key {
                    at: [key.at[0] - 1, 0],
                    ..*key
                });
            }
            if plan.core([key.at[0] + 1, 0]).is_some() {
                wants.push(Key {
                    at: [key.at[0] + 1, 0],
                    ..*key
                });
            }
        }
        let mut seen = HashSet::new();
        for key in wants {
            if !seen.insert(key) {
                continue;
            }
            if let Some(cached) = self.cache.get_mut(&key) {
                cached.last_seen = self.frame;
            } else if !self.failed.contains(&layer) && !self.pending.contains(&key) {
                let worker = self.worker.get_or_insert_with(|| worker(root.to_owned()));
                if worker.tx.try_send(key).is_ok() {
                    self.pending.insert(key);
                }
            }
        }
        // Switch a whole visible layer together. Mixed-resolution neighbouring
        // cores can have different AA; retain the exact old draw until ready.
        if pieces
            .iter()
            .all(|(key, _, _)| self.cache.contains_key(key))
        {
            for (key, bounds, core) in pieces {
                let cached = &self.cache[&key];
                piece(
                    &cached.texture,
                    bounds,
                    [
                        svg_tiles::GUARD as f32,
                        svg_tiles::GUARD as f32,
                        core[2] as f32,
                        core[3] as f32,
                    ],
                );
            }
        } else {
            for r in rects {
                draw_texture_ex(
                    texture,
                    r.x,
                    r.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(r.size()),
                        ..Default::default()
                    },
                );
            }
        }
        gl_use_default_material();
    }
    pub fn ready(&self) -> bool {
        self.ready
    }
    pub fn end(&mut self) {
        // Soft 64 MiB limit: keep every strip visible now and evict offscreen
        // generations first. Large/retina viewports can exceed this while visible.
        while self.cache.values().map(|v| v.bytes).sum::<u64>() > 64 * 1024 * 1024 {
            let oldest = self
                .cache
                .iter()
                .filter(|(_, v)| v.last_seen != self.frame)
                .min_by_key(|(_, v)| v.last_seen)
                .map(|(k, _)| *k);
            if let Some(key) = oldest {
                self.cache.remove(&key);
            } else {
                break;
            }
        }
    }
}
fn piece(texture: &Texture2D, bounds: [f64; 4], source: [f32; 4]) {
    let [x0, y0, x1, y1] = bounds.map(|v| v as f32);
    let [x, y, w, h] = source;
    let u0 = x / texture.width();
    let v0 = y / texture.height();
    let u1 = (x + w) / texture.width();
    let v1 = (y + h) / texture.height();
    draw_mesh(&Mesh {
        vertices: vec![
            Vertex::new(x0, y0, 0., u0, v0, WHITE),
            Vertex::new(x1, y0, 0., u1, v0, WHITE),
            Vertex::new(x1, y1, 0., u1, v1, WHITE),
            Vertex::new(x0, y1, 0., u0, v1, WHITE),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture.clone()),
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_worker_decodes_scenery_tiles_in_order_and_terminates() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static");
        let Worker { tx, rx } = worker(root);
        let plan = Plan::new([10000, 564], 2, 512).unwrap();
        for at in [[0, 0], [39, 0], [100, 0]] {
            tx.send(Key {
                layer: Layer::Mountains,
                plan,
                at,
            })
            .unwrap();
        }
        drop(tx);
        for at in [[0, 0], [39, 0], [100, 0]] {
            let (key, image) = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
            assert_eq!(key.at, at);
            if at[0] == 100 {
                assert!(image.is_err());
            } else {
                let image = image.unwrap();
                let core = plan.core(at).unwrap();
                assert_eq!([image.width, image.height], [core[2] + 2, core[3] + 2]);
                assert!(image
                    .rgba
                    .chunks_exact(4)
                    .all(|p| p[..3].iter().all(|c| *c <= p[3])));
            }
        }
        assert!(matches!(
            rx.recv_timeout(std::time::Duration::from_secs(10)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }
}
