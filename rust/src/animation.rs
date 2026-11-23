//! Native playback of the original Lottie artwork; no browser is involved.
use std::path::Path;
use thorvg::{Animation, ColorSpace, Matrix, Paint, Thorvg};

pub struct Clip<'a> {
    engine: &'a Thorvg,
    animation: Animation<'a>,
    pub frames: f32,
    pub duration: f32,
    pub intrinsic: (f32, f32),
    buffer: Vec<u32>,
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
        assert!(
            a.chunks_exact(4).all(|p| p[..3].iter().all(|c| *c <= p[3])),
            "Lottie surfaces must stay premultiplied through texture filtering"
        );
        assert_ne!(
            a, b,
            "Animation must advance rather than display a static poster"
        );
    }
    #[test]
    fn village_slice_tracks_center_crop_with_bounded_edge_differences() {
        let engine = Thorvg::init(0).unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../static/scenes/modvil/img/modules/gym/1.json");
        let mut clip = Clip::load(&engine, &path).unwrap();
        let full = clip.rasterize(360, 325, 0.).unwrap();
        let layout = crate::lottie_layout::layout(
            [clip.intrinsic.0 as f64, clip.intrinsic.1 as f64],
            [180., 325.],
            1.,
            crate::lottie_layout::Fit::Slice,
        );
        let cropped = clip.rasterize_layout(&layout, 0.).unwrap();
        // ThorVG's clipped render differs at the left edge of this original
        // asset: <=3 byte values in <=128 channels. Preserve that known bound
        // and require the interior to match; this is not exact pixel parity.
        let mut maximum = 0;
        let mut changed = 0;
        for y in 0..325 {
            for x in 0..180 {
                for c in 0..4 {
                    let a = cropped[(y * 180 + x) * 4 + c];
                    let b = full[(y * 360 + x + 90) * 4 + c];
                    maximum = maximum.max(a.abs_diff(b));
                    if a != b {
                        changed += 1;
                        assert!(x < 16, "interior crop mismatch at{x},{y}");
                    }
                }
            }
        }
        assert!(
            maximum <= 3 && changed <= 128,
            "crop error max{maximum}, channels{changed}"
        );
    }
    #[test]
    fn wide_card_keeps_art_proportions_and_centers_it() {
        let engine = Thorvg::init(0).unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/img/card/jetpack.json");
        let mut clip = Clip::load(&engine, &path).unwrap();
        let fitted = clip.rasterize(128, 100, 0.).unwrap();
        let wide = clip.rasterize(256, 100, 0.).unwrap();
        for y in 0..100 {
            assert_eq!(
                &wide[(y * 256 + 64) * 4..(y * 256 + 192) * 4],
                &fitted[y * 128 * 4..(y + 1) * 128 * 4]
            );
            assert!(wide[y * 256 * 4..(y * 256 + 64) * 4]
                .chunks_exact(4)
                .all(|p| p[3] == 0));
            assert!(wide[(y * 256 + 192) * 4..(y + 1) * 256 * 4]
                .chunks_exact(4)
                .all(|p| p[3] == 0));
        }
    }
}
impl<'a> Clip<'a> {
    pub fn load(engine: &'a Thorvg, path: &Path) -> Result<Self, String> {
        let mut animation = engine.animation().map_err(|e| e.to_string())?;
        animation
            .picture_mut()
            .load_from_str(&path.to_string_lossy())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let intrinsic = animation.picture().size().map_err(|e| e.to_string())?;
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
            intrinsic,
            buffer: vec![],
        })
    }
    pub fn rasterize(&mut self, width: u32, height: u32, frame: f32) -> Result<Vec<u8>, String> {
        // Keep the existing public meet raster path byte-stable for its
        // non-village callers; village passes explicit source canvas metrics.
        let scale = (width as f32 / self.intrinsic.0).min(height as f32 / self.intrinsic.1);
        let fitted = (self.intrinsic.0 * scale, self.intrinsic.1 * scale);
        let layout = crate::lottie_layout::Layout {
            width,
            height,
            sx: scale as f64,
            sy: scale as f64,
            tx: ((width as f32 - fitted.0) / 2.) as f64,
            ty: ((height as f32 - fitted.1) / 2.) as f64,
        };
        self.rasterize_layout(&layout, frame)
    }
    pub fn rasterize_layout(
        &mut self,
        layout: &crate::lottie_layout::Layout,
        frame: f32,
    ) -> Result<Vec<u8>, String> {
        let (width, height) = (layout.width, layout.height);
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("Invalid animation target size".into());
        }
        let frame = frame.clamp(0., self.frames - 0.001);
        if (self.animation.frame().map_err(|e| e.to_string())? - frame).abs() >= 0.001 {
            self.animation.set_frame(frame).map_err(|e| e.to_string())?;
        }
        // ThorVG's loader applies independent axes on resize; explicitly use
        // the source CanvasRenderer's selected meet/slice transform.
        let fitted = (
            self.intrinsic.0 * layout.sx as f32,
            self.intrinsic.1 * layout.sy as f32,
        );
        self.animation
            .picture_mut()
            .set_size(fitted.0, fitted.1)
            .map_err(|e| e.to_string())?;
        self.animation
            .picture_mut()
            .set_transform(&Matrix::IDENTITY.translate(layout.tx as f32, layout.ty as f32))
            .map_err(|e| e.to_string())?;
        self.buffer.resize((width * height) as usize, 0);
        {
            let mut canvas = self
                .engine
                .sw_canvas(Default::default())
                .map_err(|e| e.to_string())?;
            // The allocation is never resized, and the canvas is synchronously
            // destroyed before we inspect or release its target buffer.
            unsafe {
                canvas.set_target(&mut self.buffer, width, width, height, ColorSpace::ABGR8888)
            }
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
        Ok(self.buffer.iter().flat_map(|p| p.to_le_bytes()).collect())
    }
}
