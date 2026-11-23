//! CPU-only native crop/density diagnostics. No browser/GPU/window/input.
use santa_tracker::{
    animation::Clip,
    lottie_layout::{layout, Fit},
};
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("Pass original asset root"));
    let threads = std::env::args()
        .nth(3)
        .map(|n| n.parse::<u32>().unwrap())
        .unwrap_or(0);
    let engine = thorvg::Thorvg::init(threads).unwrap();
    let mut clip = Clip::load(&engine, &root.join("scenes/modvil/img/modules/gym/1.json")).unwrap();
    let mut rows = vec![];
    for frame in [0., clip.frames * 0.5] {
        let full = clip.rasterize(360, 325, frame).unwrap();
        let target = layout(
            [clip.intrinsic.0 as f64, clip.intrinsic.1 as f64],
            [180., 325.],
            1.,
            Fit::Slice,
        );
        let cropped = clip.rasterize_layout(&target, frame).unwrap();
        let (mut maximum, mut changed, mut farthest) = (0u8, 0usize, 0usize);
        for y in 0..325 {
            for x in 0..180 {
                for c in 0..4 {
                    let a = cropped[(y * 180 + x) * 4 + c];
                    let b = full[(y * 360 + x + 90) * 4 + c];
                    if a != b {
                        maximum = maximum.max(a.abs_diff(b));
                        changed += 1;
                        farthest = farthest.max(x.min(179 - x));
                    }
                }
            }
        }
        rows.push(serde_json::json!({"frame":frame,"full":[360,325],"crop":[180,325],"maximum_channel_difference":maximum,"changed_channels":changed,"farthest_edge_distance":farthest}));
    }
    let mut sampling = vec![];
    for density in [1., 2.] {
        let target = layout(
            [clip.intrinsic.0 as f64, clip.intrinsic.1 as f64],
            [1440., 1296.],
            density,
            Fit::Slice,
        );
        let mut elapsed = vec![];
        let mut rgba = vec![];
        for frame in [0., clip.frames * 0.5, clip.frames * 0.75] {
            let start = std::time::Instant::now();
            rgba = clip.rasterize_layout(&target, frame).unwrap();
            elapsed.push(start.elapsed().as_secs_f64() * 1000.);
        }
        if let Some(directory) = std::env::args().nth(4) {
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                std::path::Path::new(&directory).join(format!("{density}x.rgba")),
                &rgba,
            )
            .unwrap();
        }
        assert_eq!(rgba.len(), (target.width * target.height * 4) as usize);
        assert!(rgba
            .chunks_exact(4)
            .all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]));
        sampling.push(
            serde_json::json!({"density":density,"layout":target,"cpu_render_ms_samples":elapsed}),
        );
    }
    let report = serde_json::json!({"scope":"Native CPU clipping comparison and backing-density/associated-alpha smoke only. Not original/browser pixels, GPU presentation, resize event ordering, frame pacing or input latency. CPU timings are three diagnostic render samples per size, not application frame pacing or input latency.","threads":threads,"profile":if cfg!(debug_assertions){"debug"}else{"release"},"asset":"scenes/modvil/img/modules/gym/1.json","intrinsic":clip.intrinsic,"crop_rows":rows,"sampling":sampling});
    let text = serde_json::to_string_pretty(&report).unwrap() + "\n";
    if let Some(path) = std::env::args().nth(2) {
        std::fs::write(path, &text).unwrap();
    }
    print!("{text}");
}
