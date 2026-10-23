use macroquad::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct Assets {
    pub root: PathBuf,
    textures: HashMap<String, Texture2D>,
}
impl Assets {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            textures: HashMap::new(),
        }
    }
    pub fn load(&mut self, key: &str) -> Result<(), String> {
        if self.textures.contains_key(key) {
            return Ok(());
        }
        let path = self.root.join(key);
        let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let texture = if key.ends_with(".svg") {
            let options = resvg::usvg::Options {
                resources_dir: path.parent().map(Path::to_path_buf),
                ..Default::default()
            };
            let tree =
                resvg::usvg::Tree::from_data(&data, &options).map_err(|e| format!("{key}: {e}"))?;
            let size = tree.size().to_int_size();
            if size.width() > 16384 || size.height() > 16384 {
                return Err(format!("Texture too large: {key}"));
            }
            let mut pixels = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
                .ok_or("SVG allocation failed")?;
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
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
            Texture2D::from_rgba8(size.width() as u16, size.height() as u16, &rgba)
        } else {
            Texture2D::from_file_with_format(&data, None)
        };
        texture.set_filter(FilterMode::Linear);
        self.textures.insert(key.into(), texture);
        Ok(())
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
        if scene == "santaselfie" {
            self.load("scenes/_shared/img/button/share.svg")?;
            self.load("scenes/_shared/img/button/refresh.svg")?;
        }
        let root = self.root.join(format!("scenes/{scene}/img"));
        let mut pending = vec![root];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?.path();
                if path.is_dir() {
                    pending.push(path);
                } else if matches!(
                    path.extension().and_then(|x| x.to_str()),
                    Some("svg" | "png" | "jpg" | "webp")
                ) {
                    let key = path
                        .strip_prefix(&self.root)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .replace('\\', "/");
                    self.load(&key)?;
                }
            }
        }
        Ok(())
    }
    pub fn scene(&self, scene: &str, name: &str) -> &Texture2D {
        self.texture(&format!("scenes/{scene}/img/{name}"))
    }
}
