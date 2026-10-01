//! One background render at a time. Camera changes cancel expensive final frames.
use crate::disaster::Disaster;
use crate::{
    render::{self, Camera},
    scene::Scene,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc,
};
use std::time::Instant;
struct Request {
    disaster: Disaster,
    camera: Camera,
    generation: u64,
    time: f32,
    full: bool,
    divisor: usize,
}
pub struct Frame {
    pub disaster: Disaster,
    pub camera: Camera,
    pub generation: u64,
    pub pixels: Option<Vec<u8>>,
    pub full: bool,
    pub seconds: f32,
}
pub struct Renderer {
    tx: Option<mpsc::Sender<Request>>,
    rx: mpsc::Receiver<Frame>,
    thread: Option<std::thread::JoinHandle<()>>,
    pub generation: Arc<AtomicU64>,
    pub busy: bool,
}
impl Renderer {
    pub fn new(scene: Arc<Scene>, aftermath: Arc<Scene>, width: usize, height: usize) -> Self {
        let (tx, requests) = mpsc::channel::<Request>();
        let (results, rx) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let cancel = generation.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(request) = requests.recv() {
                let now = Instant::now();
                let divisor = if request.full { 1 } else { request.divisor };
                let (w, h) = (width / divisor, height / divisor);
                let pixels = render::render_effects(
                    if request.disaster.aftermath() {
                        &aftermath
                    } else {
                        &scene
                    },
                    request.camera,
                    w,
                    h,
                    request.time,
                    request.full,
                    request.full.then_some((&cancel, request.generation)),
                    request.disaster,
                )
                .map(|pixels| {
                    if divisor == 1 {
                        pixels
                    } else {
                        render::upscale(&pixels, w, h, width, height)
                    }
                });
                if results
                    .send(Frame {
                        disaster: request.disaster,
                        camera: request.camera,
                        generation: request.generation,
                        pixels,
                        full: request.full,
                        seconds: now.elapsed().as_secs_f32(),
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            tx: Some(tx),
            rx,
            thread: Some(thread),
            generation,
            busy: false,
        }
    }
    pub fn invalidate(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::Relaxed) + 1
    }
    pub fn request(
        &mut self,
        camera: Camera,
        generation: u64,
        time: f32,
        full: bool,
        divisor: usize,
        disaster: Disaster,
    ) {
        assert!(!self.busy, "only one frame may be in flight");
        self.tx
            .as_ref()
            .unwrap()
            .send(Request {
                disaster,
                camera,
                generation,
                time,
                full,
                divisor,
            })
            .unwrap();
        self.busy = true;
    }
    pub fn poll(&mut self) -> Option<Frame> {
        let frame = self.rx.try_recv().ok()?;
        self.busy = false;
        Some(frame)
    }
}
impl Drop for Renderer {
    fn drop(&mut self) {
        self.invalidate();
        self.tx.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
