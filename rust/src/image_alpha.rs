//! Extend sprite edge RGB under transparent texels, avoiding straight-alpha
//! filtering's dark fringe without changing any visible artwork or alpha.
pub fn bleed_edges(rgba: &mut [u8], width: usize, height: usize) {
    assert_eq!(rgba.len(), width * height * 4);
    if rgba.chunks_exact(4).all(|p| p[3] > 0) {
        return;
    }
    let mut known: Vec<bool> = rgba.chunks_exact(4).map(|p| p[3] > 0).collect();
    for _ in 0..2 {
        let original = rgba.to_vec();
        let previous = known.clone();
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                if previous[index] {
                    continue;
                }
                let mut total = 0u32;
                let mut color = [0u32; 3];
                for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        let (nx, ny) = (x as isize + dx, y as isize + dy);
                        if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                            continue;
                        }
                        let near = ny as usize * width + nx as usize;
                        if !previous[near] {
                            continue;
                        }
                        let weight = u32::from(original[near * 4 + 3]).max(1);
                        total += weight;
                        for c in 0..3 {
                            color[c] += u32::from(original[near * 4 + c]) * weight;
                        }
                    }
                }
                if total > 0 {
                    known[index] = true;
                    for c in 0..3 {
                        rgba[index * 4 + c] = (color[c] / total) as u8;
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transparent_filtering_retains_edge_color_and_original_alpha() {
        let mut image = vec![0, 0, 0, 0, 220, 80, 30, 255, 0, 0, 0, 0];
        let original = image.clone();
        bleed_edges(&mut image, 3, 1);
        assert_eq!(&image[0..3], &[220, 80, 30]);
        assert_eq!(&image[8..11], &[220, 80, 30]);
        for (a, b) in image.chunks_exact(4).zip(original.chunks_exact(4)) {
            assert_eq!(a[3], b[3]);
            if b[3] > 0 {
                assert_eq!(a, b);
            }
        }
    }
}
