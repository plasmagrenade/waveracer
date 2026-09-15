use kdam::{BarExt, tqdm};
use rt2::render::BVHNode;
use rt2::render::Film;
use rt2::render::MonteCarloRenderer;
use rt2::render::MultiFilePngWriter;
use rt2::scene::bunny;

fn main() {
    let render_w = 1000.0;
    let aspect_ratio = 0.6;

    let renderer = MonteCarloRenderer { max_depth: 10 };
    let film = Film {
        width: render_w as usize,
        height: (aspect_ratio * render_w) as usize,
        samples_per_pixel: 500,
    };

    let image_writer = MultiFilePngWriter::new("./output", "frame_{{frame_number}}");

    let mut scene = bunny(renderer, film);
    let root = BVHNode::new(&mut scene.world);

    let mut progress = tqdm!(total = scene.camera.total_frames);
    progress.refresh().unwrap();

    let mut errs = vec![];

    // for frame in scene.camera.render_accel_frames(&root) {
    for frame in scene.camera.render_frames_gpu(&scene.world) {
        if let Err(val) = image_writer.write(&frame) {
            errs.push(format!("frame {}: {}", frame.frame_number, val));
        }
        progress.update(1).unwrap();
    }

    if !errs.is_empty() {
        dbg!(errs);
    }
}
