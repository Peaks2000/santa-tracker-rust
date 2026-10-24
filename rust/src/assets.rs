use macroquad::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct Assets {
    pub root: PathBuf,
    textures: HashMap<String, Texture2D>,
    loading: Option<SceneLoad>,
}
impl Assets {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            textures: HashMap::new(),
            loading: None,
        }
    }
    pub fn load(&mut self, key: &str) -> Result<(), String> {
        if self.textures.contains_key(key) {
            return Ok(());
        }
        let texture = Texture2D::from_image(&decode_image(&self.root, key)?);
        configure_texture(key, &texture);
        self.textures.insert(key.into(), texture);
        Ok(())
    }
    pub fn start_scene_load(&mut self, scene: &str) -> Result<(), String> {
        if self.loading.is_some() {
            return Err("A scene is already loading".into());
        }
        let keys = scene_keys(&self.root, scene)?
            .into_iter()
            .filter(|key| !self.has(key))
            .collect::<Vec<_>>();
        let root = self.root.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel(2);
        std::thread::spawn(move || {
            for key in keys {
                match decode_image(&root, &key) {
                    Ok(image) => {
                        if tx.send(LoadEvent::Image(key, image)).is_err() {
                            return;
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(LoadEvent::Done(Err(error)));
                        return;
                    }
                }
            }
            let _ = tx.send(LoadEvent::Done(Ok(())));
        });
        self.loading = Some(SceneLoad {
            scene: scene.into(),
            rx,
        });
        Ok(())
    }
    /// Bound uploads per frame; SVG/PNG work stays on the decoder thread.
    pub fn poll_scene_load(&mut self) -> Option<(String, Result<(), String>)> {
        for _ in 0..2 {
            let event = self.loading.as_ref()?.rx.try_recv();
            match event {
                Ok(LoadEvent::Image(key, image)) => {
                    let texture = Texture2D::from_image(&image);
                    configure_texture(&key, &texture);
                    self.textures.insert(key, texture);
                }
                Ok(LoadEvent::Done(result)) => {
                    return Some((self.loading.take().unwrap().scene, result))
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => return None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Some((
                        self.loading.take().unwrap().scene,
                        Err("Asset decoder stopped unexpectedly".into()),
                    ))
                }
            }
        }
        None
    }
    pub fn texture(&self, key: &str) -> &Texture2D {
        &self.textures[key]
    }
    pub fn has(&self, key: &str) -> bool {
        self.textures.contains_key(key)
    }
    pub fn runner(&self, name: &str) -> &Texture2D {
        self.texture(&format!("scenes/runner/img/{name}"))
    }
    pub fn load_scene(&mut self, scene: &str) -> Result<(), String> {
        for key in scene_keys(&self.root, scene)? {
            self.load(&key)?;
        }
        Ok(())
    }
    /// CSS/SVG dimensions stay independent of texture sampling resolution.
    pub fn scene_size(&self, scene: &str, name: &str) -> Vec2 {
        let key = format!("scenes/{scene}/img/{name}");
        let texture = self.texture(&key);
        let scale = if key.ends_with(".svg") {
            svg_raster_scale(&key)
        } else {
            1.
        };
        vec2(texture.width() / scale, texture.height() / scale)
    }
    pub fn scene(&self, scene: &str, name: &str) -> &Texture2D {
        self.texture(&format!("scenes/{scene}/img/{name}"))
    }
}

struct SceneLoad {
    scene: String,
    rx: std::sync::mpsc::Receiver<LoadEvent>,
}
enum LoadEvent {
    Image(String, Image),
    Done(Result<(), String>),
}
fn decode_image(root: &Path, key: &str) -> Result<Image, String> {
    let path = root.join(key);
    let data = if key == "native/tutorial/elfski.png" {
        include_bytes!("../assets/tutorial/elfski.png").to_vec()
    } else {
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?
    };
    let premultiplied = key.starts_with("scenes/santaselfie/")
        || key.starts_with("scenes/_shared/img/button/")
        || key.starts_with("scenes/boatload/")
        || key.starts_with("img/sleigh/");
    let mut image = if key.ends_with(".svg") {
        let options = resvg::usvg::Options {
            resources_dir: path.parent().map(Path::to_path_buf),
            ..Default::default()
        };
        let tree =
            resvg::usvg::Tree::from_data(&data, &options).map_err(|e| format!("{key}: {e}"))?;
        let render_scale = svg_raster_scale(key);
        // CSS background images retain their intrinsic SVG proportions inside
        // the element box. The elf is centered; boat1 is top-left aligned.
        let background = match key {
            "scenes/boatload/img/player-elf.svg" => Some((44., 44., true)),
            "scenes/boatload/img/boat1.svg" => Some((88., 194., false)),
            _ => None,
        };
        let (size, transform) = if let Some((w, h, center)) = background {
            let x = if center {
                (w - tree.size().width()) / 2.
            } else {
                0.
            };
            let y = if center {
                (h - tree.size().height()) / 2.
            } else {
                0.
            };
            (
                resvg::tiny_skia::IntSize::from_wh(
                    (w * render_scale).ceil() as u32,
                    (h * render_scale).ceil() as u32,
                )
                .ok_or("SVG background size failed")?,
                resvg::tiny_skia::Transform::from_row(
                    render_scale,
                    0.,
                    0.,
                    render_scale,
                    x * render_scale,
                    y * render_scale,
                ),
            )
        } else {
            (
                tree.size()
                    .to_int_size()
                    .scale_by(render_scale)
                    .ok_or("SVG scale failed")?,
                resvg::tiny_skia::Transform::from_scale(render_scale, render_scale),
            )
        };
        if size.width() > 16384 || size.height() > 16384 {
            return Err(format!("Texture too large: {key}"));
        }
        let mut pixels = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("SVG allocation failed")?;
        resvg::render(&tree, transform, &mut pixels.as_mut());
        let mut rgba = pixels.data().to_vec();
        if !premultiplied {
            for p in rgba.chunks_exact_mut(4) {
                if p[3] > 0 {
                    for i in 0..3 {
                        p[i] = (u32::from(p[i]) * 255 / u32::from(p[3])).min(255) as u8;
                    }
                }
            }
        }
        Image {
            bytes: rgba,
            width: size.width() as u16,
            height: size.height() as u16,
        }
    } else {
        let mut image =
            Image::from_file_with_format(&data, None).map_err(|e| format!("{key}: {e}"))?;
        if premultiplied {
            santa_tracker::image_alpha::premultiply(&mut image.bytes);
        }
        image
    };
    // All CSS-style 2D images share the same straight-alpha filtering issue.
    // Leave the source 3D maps alone: their mipmaps follow THREE's renderer.
    if !premultiplied && !key.starts_with("scenes/elfski/") {
        santa_tracker::image_alpha::bleed_edges(
            &mut image.bytes,
            image.width as usize,
            image.height as usize,
        );
    }
    Ok(image)
}
fn scene_keys(root: &Path, scene: &str) -> Result<Vec<String>, String> {
    let mut keys = vec![];
    let tutorials: &[&str] = match scene {
        "penguindash" => &["keys-arrows", "device-tilt"],
        "glider" => &["keys-arrows", "keys-space", "device-tilt"],
        "runner" => &["touch-updown", "keys-updown"],
        "jetpack" => &["touch-updown", "keys-leftright", "keys-updown"],
        "presentdrop" => &["touch-leftright", "keys-space", "keys-leftright"],
        "boatload" => &["touch-updown", "keys-space", "keys-updown"],
        _ => &[],
    };
    keys.extend(
        tutorials
            .iter()
            .map(|name| format!("img/tutorial/input/{name}.svg")),
    );
    if scene == "elfski" {
        keys.push("native/tutorial/elfski.png".into());
    }

    if scene == "elfski" {
        keys.push("scenes/elfski/models/elf-tidy.png".into());
    }
    if scene == "elfmaker" || scene == "jamband" {
        keys.extend(
            [
                "shuffle.svg",
                "share.svg",
                "arrow-key-left-dark.svg",
                "arrow-key-right-dark.svg",
            ]
            .map(|s| format!("scenes/_shared/img/button/{s}")),
        );
    }
    if scene == "santaselfie" {
        keys.extend([
            "scenes/_shared/img/button/share.svg".into(),
            "scenes/_shared/img/button/refresh.svg".into(),
        ]);
    }
    let mut pending = vec![root.join(format!("scenes/{scene}/img"))];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                pending.push(path);
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("svg" | "png" | "jpg" | "webp")
            ) {
                keys.push(
                    path.strip_prefix(root)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    keys.sort();
    Ok(keys)
}

/// Keep the source THREE maps on trilinear filtering. Boatload CSS images also
/// need mipmaps when minified; their RGBA is premultiplied before filtering.
pub fn configure_texture(key: &str, texture: &Texture2D) {
    texture.set_filter(FilterMode::Linear);
    if key.starts_with("scenes/elfski/") || key.starts_with("scenes/boatload/") {
        let id = texture.raw_miniquad_id();
        // Upload is complete and no draw references a newly created texture.
        unsafe {
            let gl = get_internal_gl();
            gl.quad_context.texture_generate_mipmaps(id);
            gl.quad_context.texture_set_min_filter(
                id,
                FilterMode::Linear,
                macroquad::miniquad::MipmapFilterMode::Linear,
            );
        }
    }
}

fn svg_raster_scale(key: &str) -> f32 {
    if key.starts_with("img/sleigh/") {
        4.
    } else if key.ends_with("glider/img/glider-sprite.svg") {
        3.
    } else if key.starts_with("scenes/_shared/img/button/") {
        4.
    } else if key.ends_with("jetpack/img/levelclouds.svg") {
        5.
    } else {
        1.
    }
}
