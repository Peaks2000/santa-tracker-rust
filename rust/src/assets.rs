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
        texture.set_filter(FilterMode::Linear);
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
                    texture.set_filter(FilterMode::Linear);
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
    let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let image = if key.ends_with(".svg") {
        let options = resvg::usvg::Options {
            resources_dir: path.parent().map(Path::to_path_buf),
            ..Default::default()
        };
        let tree =
            resvg::usvg::Tree::from_data(&data, &options).map_err(|e| format!("{key}: {e}"))?;
        let render_scale = if key.ends_with("jetpack/img/levelclouds.svg") {
            5.
        } else {
            1.
        };
        let size = tree
            .size()
            .to_int_size()
            .scale_by(render_scale)
            .ok_or("SVG scale failed")?;
        if size.width() > 16384 || size.height() > 16384 {
            return Err(format!("Texture too large: {key}"));
        }
        let mut pixels = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("SVG allocation failed")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(render_scale, render_scale),
            &mut pixels.as_mut(),
        );
        let mut rgba = pixels.data().to_vec();
        for p in rgba.chunks_exact_mut(4) {
            if p[3] > 0 {
                for i in 0..3 {
                    p[i] = (u32::from(p[i]) * 255 / u32::from(p[3])).min(255) as u8;
                }
            }
        }
        Image {
            bytes: rgba,
            width: size.width() as u16,
            height: size.height() as u16,
        }
    } else if key.starts_with("scenes/jetpack/") && key.ends_with(".png") {
        let mut image =
            Image::from_file_with_format(&data, None).map_err(|e| format!("{key}: {e}"))?;
        santa_tracker::image_alpha::bleed_edges(
            &mut image.bytes,
            image.width as usize,
            image.height as usize,
        );
        image
    } else {
        Image::from_file_with_format(&data, None).map_err(|e| format!("{key}: {e}"))?
    };
    Ok(image)
}
fn scene_keys(root: &Path, scene: &str) -> Result<Vec<String>, String> {
    let mut keys = vec![];
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
