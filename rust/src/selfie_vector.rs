//! Direct SVG sampling for Santa Selfie's CSS face/body layers. This deliberately
//! excludes the original low-resolution hair/beard canvases and raster artwork.
pub struct Raster {
    tree: resvg::usvg::Tree,
}
pub fn selected(name: &str) -> bool {
    name.ends_with(".svg") && (name.starts_with("face/") || name.starts_with("santa/"))
}
impl Raster {
    pub fn new(data: &[u8]) -> Result<Self, String> {
        Ok(Self {
            tree: resvg::usvg::Tree::from_data(data, &Default::default())
                .map_err(|e| e.to_string())?,
        })
    }
    pub fn natural_size(&self) -> (f32, f32) {
        (self.tree.size().width(), self.tree.size().height())
    }
    pub fn pixels(&self, width: u32, height: u32) -> Result<Vec<u8>, String> {
        let mut pixels =
            resvg::tiny_skia::Pixmap::new(width, height).ok_or("SVG allocation failed")?;
        let size = self.tree.size();
        // The caller already fitted the CSS background viewport. Each physical
        // bitmap maps to precisely that logical rectangle at presentation.
        resvg::render(
            &self.tree,
            resvg::tiny_skia::Transform::from_scale(
                width as f32 / size.width(),
                height as f32 / size.height(),
            ),
            &mut pixels.as_mut(),
        );
        Ok(pixels.data().to_vec())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn css_vectors_keep_associated_color_at_target_resolution_but_exclude_canvas() {
        assert!(selected("face/eye.svg") && selected("santa/face.svg"));
        for name in [
            "hair.svg",
            "face/cheek.png",
            "tools/decorations/bow.svg",
            "background/mirror.svg",
        ] {
            assert!(!selected(name));
        }
        let r=Raster::new(br##"<svg xmlns="http://www.w3.org/2000/svg" width="25" height="25"><circle cx="12.5" cy="12.5" r="10" fill="#cc4422" opacity="0.5"/></svg>"##).unwrap();
        assert_eq!(r.natural_size(), (25., 25.));
        let mut coverage: Option<f64> = None;
        for (w, h) in [(25, 25), (35, 35), (70, 70)] {
            let bytes = r.pixels(w, h).unwrap();
            assert_eq!(bytes.len(), (w * h * 4) as usize);
            assert!(bytes
                .chunks_exact(4)
                .all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]));
            let area = bytes
                .chunks_exact(4)
                .map(|p| p[3] as f64 / 255.)
                .sum::<f64>()
                / (w * h) as f64;
            if let Some(previous) = coverage {
                assert!((area - previous).abs() < 0.005);
            }
            coverage = Some(area);
        }
    }
}
