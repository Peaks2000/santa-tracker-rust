//! Render the loading illustration away from the window/input thread.
use macroquad::prelude::*;
use std::{path::PathBuf, sync::mpsc, thread};
enum Request {
    Render(u32, u32, f32),
    Stop,
}
pub struct LoaderAnimation {
    requests: mpsc::Sender<Request>,
    frames: mpsc::Receiver<(u32, u32, Vec<u8>)>,
    worker: Option<thread::JoinHandle<()>>,
    texture: Option<Texture2D>,
    material: Material,
}
impl LoaderAnimation {
    pub fn new(root: PathBuf) -> Self {
        let (requests, rx) = mpsc::channel();
        let (tx, frames) = mpsc::sync_channel(2);
        let worker = thread::spawn(move || {
            let run = || -> Result<(), String> {
                let engine = thorvg::Thorvg::init(0).map_err(|e| e.to_string())?;
                let mut clip = santa_tracker::animation::Clip::load(
                    &engine,
                    &root.join("img/interlude/loader.json"),
                )?;
                loop {
                    let Ok(mut request) = rx.recv() else {
                        break;
                    };
                    if matches!(request, Request::Stop) {
                        break;
                    }
                    for newer in rx.try_iter() {
                        if matches!(newer, Request::Stop) {
                            return Ok(());
                        }
                        request = newer;
                    }
                    if let Request::Render(w, h, ratio) = request {
                        let pixels = clip.rasterize(w, h, ratio * clip.frames)?;
                        if let Err(mpsc::TrySendError::Disconnected(_)) =
                            tx.try_send((w, h, pixels))
                        {
                            break;
                        }
                    }
                }
                Ok(())
            };
            if let Err(error) = run() {
                eprintln!("Loading animation: {error}");
            }
        });
        Self {
            requests,
            frames,
            worker: Some(worker),
            texture: None,
            material: crate::ui_render::premultiplied_material()
                .expect("Loader compositing material"),
        }
    }
    pub fn draw(&mut self, y: f32, ratio: f32) {
        let scale = (screen_width() / 1440.).min(screen_height() / 1000.);
        let width = (1440. * scale).ceil() as u32;
        let height = (1000. * scale).ceil() as u32;
        self.requests
            .send(Request::Render(width, height, ratio))
            .ok();
        if let Some((w, h, bytes)) = self.frames.try_iter().last() {
            if self
                .texture
                .as_ref()
                .is_some_and(|t| t.width() == w as f32 && t.height() == h as f32)
            {
                self.texture.as_ref().unwrap().update(&Image {
                    bytes,
                    width: w as u16,
                    height: h as u16,
                });
            } else {
                let texture = Texture2D::from_rgba8(w as u16, h as u16, &bytes);
                texture.set_filter(FilterMode::Linear);
                self.texture = Some(texture);
            }
        }
        if let Some(texture) = &self.texture {
            gl_use_material(&self.material);
            draw_texture_ex(
                texture,
                (screen_width() - width as f32) / 2.,
                y + (screen_height() - height as f32) / 2.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width as f32, height as f32)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
        }
    }
}
impl Drop for LoaderAnimation {
    fn drop(&mut self) {
        self.requests.send(Request::Stop).ok();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
