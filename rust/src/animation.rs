//! Native playback of the original Lottie artwork; no browser is involved.
use std::path::Path;
use thorvg::{Animation, ColorSpace, Paint, Thorvg};

pub struct Clip<'a> {
    engine: &'a Thorvg,
    animation: Animation<'a>,
    pub frames: f32,
    pub duration: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_lottie_art_renders_natively() {
        let engine = Thorvg::init(0).unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../static/scenes/modvil/img/modules/gym/1.json");
        let mut clip = Clip::load(&engine, &path).unwrap();
        let a = clip.rasterize(320, 289, 0.).unwrap();
        let b = clip.rasterize(320, 289, clip.frames * 0.5).unwrap();
        assert_eq!(a.len(), 320 * 289 * 4);
        assert!(a.chunks_exact(4).any(|p| p[3] > 0));
        assert_ne!(
            a, b,
            "Animation must advance rather than display a static poster"
        );
    }
}
impl<'a> Clip<'a> {
    pub fn load(engine: &'a Thorvg, path: &Path) -> Result<Self, String> {
        let mut animation = engine.animation().map_err(|e| e.to_string())?;
        animation
            .picture_mut()
            .load_from_str(&path.to_string_lossy())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let frames = animation.total_frame().map_err(|e| e.to_string())?;
        let duration = animation.duration().map_err(|e| e.to_string())?;
        if frames <= 0. || duration <= 0. {
            return Err(format!("Empty animation: {}", path.display()));
        }
        Ok(Self {
            engine,
            animation,
            frames,
            duration,
        })
    }
    pub fn rasterize(&mut self, width: u32, height: u32, frame: f32) -> Result<Vec<u8>, String> {
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("Invalid animation target size".into());
        }
        let frame = frame.clamp(0., self.frames - 0.001);
        if (self.animation.frame().map_err(|e| e.to_string())? - frame).abs() >= 0.001 {
            self.animation.set_frame(frame).map_err(|e| e.to_string())?;
        }
        self.animation
            .picture_mut()
            .set_size(width as f32, height as f32)
            .map_err(|e| e.to_string())?;
        let mut buffer = vec![0u32; (width * height) as usize];
        {
            let mut canvas = self
                .engine
                .sw_canvas(Default::default())
                .map_err(|e| e.to_string())?;
            // The allocation is never resized, and the canvas is synchronously
            // destroyed before we inspect or release its target buffer.
            unsafe { canvas.set_target(&mut buffer, width, width, height, ColorSpace::ABGR8888S) }
                .map_err(|e| e.to_string())?;
            let picture = self
                .animation
                .picture()
                .duplicate()
                .ok_or("Could not duplicate animation picture")?;
            canvas.add(picture).map_err(|e| e.to_string())?;
            canvas.draw(true).map_err(|e| e.to_string())?;
            canvas.sync().map_err(|e| e.to_string())?;
        }
        Ok(buffer.into_iter().flat_map(u32::to_le_bytes).collect())
    }
}
