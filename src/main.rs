mod disaster;
mod material;
mod math;
#[cfg(target_os = "macos")]
mod native;
mod render;
mod scene;
mod ui;
#[cfg(target_os = "macos")]
mod worker;
use render::Camera;
use scene::Tag;
use std::time::Instant;
const W: usize = 1100;
const H: usize = 756;
struct Audio {
    child: Option<std::process::Child>,
    path: std::path::PathBuf,
    pub enabled: bool,
}
impl Audio {
    fn new() -> Self {
        Self {
            child: None,
            path: std::env::temp_dir()
                .join(format!("hell-creek-{}-forest.mp3", std::process::id())),
            enabled: false,
        }
    }
    fn start(&mut self) -> std::io::Result<()> {
        if !self.path.exists() {
            std::fs::write(&self.path, include_bytes!("../assets/jurassic-forest.mp3"))?;
        }
        self.child = Some(
            std::process::Command::new("/usr/bin/afplay")
                .arg(&self.path)
                .arg("-v")
                .arg("0.55")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?,
        );
        self.enabled = true;
        Ok(())
    }
    fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        self.enabled = false;
    }
    fn toggle(&mut self) -> std::io::Result<()> {
        if self.enabled {
            self.stop();
            Ok(())
        } else {
            self.start()
        }
    }
    fn tick(&mut self) -> std::io::Result<()> {
        if self.enabled {
            if let Some(c) = self.child.as_mut() {
                if let Some(status) = c.try_wait()? {
                    self.child = None;
                    if status.success() {
                        self.start()?;
                    } else {
                        self.enabled = false;
                        return Err(std::io::Error::other("afplay no pudo reproducir el audio"));
                    }
                }
            }
        }
        Ok(())
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_file(&self.path);
    }
}
fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|s| s == "--help") {
        println!("Diorama - Rust / ray tracing CPU\n--render salida.ppm [--view 1..16] [--info] : imagen sin ventana\n--frames N : comprobacion de ventana\n1: paisaje. 2: Triceratops. 3: T. rex. 4: Edmontosaurus.\n5: Ankylosaurus. 6: ave. 7: anfibio. 8: montanas. 9: Golfo de Mexico. 0: pozas costeras.\nVistas --view 10: mosasaurio, 11: plesiosaurio, 12: pozas, 13: minerales. G: minerales. V: reflejos del oceano. B: crater. T: tsunamis. --view 15: crater, 16: tsunamis.\nArrastrar: girar. Rueda +/-: zoom. WASD: rotar. M: sonido. C: cerrar ficha. R: inicio. Esc: salir.\nClic sobre un animal: zoom e informacion. X: volver. Boton meteorito: impacto/reiniciar. --impact SEG: estado de la secuencia en --render.");
        return Ok(());
    }
    let view = args
        .iter()
        .position(|s| s == "--view")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, 16);
    if args.iter().any(|s| s == "--benchmark") {
        let scene = scene::biome();
        for view in [1, 3, 7] {
            for full in [false, true] {
                let (w, h) = if full { (W, H) } else { (W / 3, H / 3) };
                let mut timings = Vec::new();
                for _ in 0..3 {
                    let start = Instant::now();
                    std::hint::black_box(render::render(
                        &scene,
                        Camera::preset(view),
                        w,
                        h,
                        0.,
                        full,
                    ));
                    timings.push(start.elapsed().as_secs_f64() * 1000.);
                }
                timings.sort_by(f64::total_cmp);
                println!(
                    "vista {view} / {} / {w}x{h}: {:.1} ms (mediana de 3)",
                    if full { "final" } else { "movimiento" },
                    timings[1]
                );
            }
        }
        return Ok(());
    }
    if let Some(i) = args.iter().position(|s| s == "--render") {
        let original = scene::biome();
        let disaster = args
            .iter()
            .position(|s| s == "--impact")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<f32>().ok())
            .map(disaster::Disaster::at)
            .unwrap_or_default();
        let scene = match disaster.stage() {
            0 => original,
            1 => scene::impacted(&original),
            2 => scene::ruins(&original),
            _ => scene::aftermath(&original),
        };
        let now = Instant::now();
        let (w, h) = (1600, 1100);
        let camera = if args.iter().any(|s| s == "--survivors") {
            Camera {
                yaw: 0.3,
                pitch: 0.38,
                distance: 9.,
                target: scene::SURVIVOR_SITE,
            }
        } else {
            Camera::preset(view)
        };
        let mut pixels = render::render_effects(
            &scene,
            camera,
            w,
            h,
            disaster.elapsed.unwrap_or(0.),
            true,
            None,
            disaster,
        )
        .unwrap();
        ui::controls(&mut pixels, w, h, disaster, false);
        if args.iter().any(|s| s == "--info") {
            ui::overlay(
                &mut pixels,
                w,
                h,
                false,
                describe(match view {
                    7 => Tag::Scapherpeton,
                    9 | 10 => Tag::Mosasaur,
                    11 => Tag::Plesiosaur,
                    12 => Tag::HorseshoeCrab,
                    13 => Tag::Mineral,
                    _ => Tag::Tyrannosaurus,
                }),
                view,
            );
        }
        let path = args
            .get(i + 1)
            .filter(|s| !s.starts_with("--"))
            .map(String::as_str)
            .unwrap_or("diorama.ppm");
        render::ppm(path, &pixels, w, h)?;
        println!(
            "{} cubos / {:.2}s / {path}",
            scene.cubes.len(),
            now.elapsed().as_secs_f32()
        );
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        let frames = args
            .iter()
            .position(|s| s == "--frames")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok());
        run(frames, view)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(std::io::Error::other(
            "La ventana nativa requiere macOS. Usa --render salida.ppm en otros sistemas.",
        ))
    }
}
fn describe(tag: Tag) -> &'static str {
    match tag{
 Tag::Tyrannosaurus=>"TYRANNOSAURUS REX|Hace unos 66 millones de anos. Norteamerica occidental.|Dieta: carnivora. Longitud aproximada: 12-13 m.|Su craneo robusto y sus potentes mandibulas le permitian procesar grandes presas. Tenia dos dedos en cada mano.",
 Tag::Triceratops=>"TRICERATOPS PRORSUS|Final del Cretacico. Norteamerica occidental.|Dieta: herbivora. Longitud aproximada: 8-9 m.|Su pico cortaba plantas y sus baterias de dientes las procesaban. Presentaba tres cuernos y una gola osea.",
 Tag::Edmontosaurus=>"EDMONTOSAURUS ANNECTENS|Hace unos 66 millones de anos.|Dieta: herbivora. Longitud aproximada: 10-12 m.|Dinosaurio de pico ancho y numerosos dientes. Podia desplazarse sobre dos o cuatro patas. La agrupacion es artistica.",
 Tag::Ankylosaurus=>"ANKYLOSAURUS MAGNIVENTRIS|Final del Cretacico. Norteamerica.|Dieta: herbivora. Longitud aproximada: 6-8 m.|Su cuerpo estaba protegido por osteodermos. La cola terminaba en una maza osea. La disposicion exacta de su armadura es incierta.",
 Tag::Enantiornithine=>"ENANTIORNITHES INDET.|Aves arcaicas del Cretacico final.|El registro documenta su presencia cerca del limite de extincion.|No se asigna una especie concreta. Plumaje, color y postura son una reconstruccion artistica.",
 Tag::Scapherpeton=>"SCAPHERPETON TECTUM|Anfibio documentado en el Cretacico final de Norteamerica.|Ambiente: riberas y zonas humedas.|Conocido por restos fosiles, incluidas vertebras. Su aspecto y color son una reconstruccion estilizada; no se afirma que pereciera en el impacto.",
 Tag::Mineral=>"CRISTAL MINERAL|Material transparente de textura con inclusiones.|Los rayos se desvian al entrar y salir del cristal. Reflexion Fresnel y reflexion interna total.|Indice de refraccion configurado: 1.52. Mineral estilizado, sin identificacion de especie mineralogica.",
 Tag::Mosasaur=>"MOSASAURIO - MOSASAURIDAE|REPTIL MARINO, NO UN DINOSAURIO.|Reconstruccion de un mosasaurio del Cretacico final, hace unos 66 millones de anos.|Depredador con cuatro aletas, cola propulsora y pulmones. Se extinguio en la crisis K-Pg.|Modelo generico, sin asignacion de especie ni localidad exacta.",
 Tag::Plesiosaur=>"PLESIOSAURIO - ELASMOSAURIDAE|REPTIL MARINO de cuello largo y cuatro aletas.|Representacion de linajes presentes en el Cretacico final. Respiraba aire.|Los plesiosaurios desaparecieron en la extincion K-Pg. No es el genero jurasico Plesiosaurus.",
 Tag::Ammonite=>"AMMONITA - AMMONOIDEA|MOLUSCO CEFALOPODO de concha enrollada.|Representacion del Cretacico final, sin especie concreta. No es un caracol.|Las ammonitas desaparecen en la crisis del limite K-Pg; se retiran del paisaje posterior.",
 Tag::HorseshoeCrab=>"CANGREJO HERRADURA - XIPHOSURA|ARTROPODO QUELICERADO, no un verdadero cangrejo.|Habitante de fondos y costas someras. Su cola ayuda a enderezarse.|Su linaje sobrevivio al K-Pg. No se representa una especie actual como si hubiera vivido hace 66 millones de anos.",
 Tag::Bivalve=>"BIVALVO INDETERMINADO|MOLUSCO con dos valvas protectoras.|Representacion de linajes filtradores costeros que atravesaron el K-Pg.|Muchos bivalvos se extinguieron, incluidos los rudistas. Este modelo generico no representa a todos los moluscos.",
 Tag::Nautiloid=>"NAUTILOIDEO INDETERMINADO|MOLUSCO CEFALOPODO de concha con camaras.|Sus linajes sobrevivieron al K-Pg, a diferencia de las ammonitas.|Reconstruccion generica; no es una especie moderna de Nautilus ni una identificacion fosil concreta.",
 Tag::Mammal=>"MULTITUBERCULATA INDET.|Pequeno mamifero: representacion de un linaje que atraveso la extincion K-Pg.|No se asigna una especie concreta. El aspecto y el comportamiento son artisticos.|Esta escena representa una etapa posterior, no la supervivencia en el punto de impacto.",
 Tag::CrownBird=>"AVE MODERNA BASAL - INDET.|Representacion de linajes de aves modernas que sobrevivieron al limite K-Pg.|La vida cerca del suelo pudo favorecer a sus ancestros tras el colapso de los bosques.|No es una enantiornita ni una especie moderna concreta. Reconstruccion artistica.",
 Tag::Conifer=>"CONIFERA|Arbol productor de semillas y conos.|Parte del bosque cretacico. Su forma es estilizada; no se asigna una especie vegetal moderna.",
 Tag::Broadleaf=>"ARBOL DE HOJA ANCHA|Angiosperma del bosque.|Las plantas con flores ya formaban parte de los ecosistemas del Cretacico final.",
 Tag::Fern=>"HELECHO|Planta del sotobosque humedo.|Se reproduce mediante esporas. Las riberas y los lugares sombreados ofrecen condiciones favorables.",Tag::None=>""}
}
#[cfg(target_os = "macos")]
fn run(max_frames: Option<u64>, initial_view: usize) -> std::io::Result<()> {
    use native::{Event, Point};
    use std::sync::Arc;
    use std::time::Duration;
    let scene = Arc::new(scene::biome());
    let scenes = [
        scene.clone(),
        Arc::new(scene::impacted(&scene)),
        Arc::new(scene::ruins(&scene)),
        Arc::new(scene::aftermath(&scene)),
    ];
    let window = native::Window::new(W, H);
    let mut renderer = worker::Renderer::new(scenes.clone(), W, H);
    let mut audio = Audio::new();
    let mut view = initial_view;
    let mut cam = Camera::preset(view);
    let mut displayed_cam = cam;
    let mut return_camera: Option<Camera> = None;
    let mut transition: Option<(Camera, Camera, Instant)> = None;
    let mut impact_start: Option<Instant> = None;
    let mut disaster = disaster::Disaster::default();
    let mut displayed_disaster = disaster;
    let mut last_animation = Instant::now();
    let mut info = String::new();
    let mut drag: Option<(Point, Point, bool)> = None;
    let mut keys = [false; 128];
    let start = Instant::now();
    let mut last_tick = start;
    let mut last_motion = start;
    let mut revision = 0;
    let mut preview_revision = None;
    let mut final_revision = None;
    let mut divisor = 3usize;
    let mut background: Option<Vec<u8>> = None;
    let mut redraw = false;
    let mut count = 0u64;
    println!(
        "Diorama: {} cubos. WASD: rotar. Clic: ficha. Vistas 1-8. M: audio.",
        scene.cubes.len()
    );
    'app: loop {
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32().min(0.05);
        last_tick = now;
        let mut changed = false;
        for event in window.events() {
            match event {
                Event::Quit => break 'app,
                Event::FocusLost => {
                    keys.fill(false);
                    drag = None;
                }
                Event::KeyUp(key) => {
                    if let Some(held) = keys.get_mut(key as usize) {
                        *held = false;
                    }
                }
                Event::Down(p) => {
                    transition = None;
                    drag = Some((p, p, false));
                }
                Event::Drag(p) => {
                    if let Some((origin, last, moved)) = drag.as_mut() {
                        if (p.x - origin.x).hypot(p.y - origin.y) > 4. {
                            *moved = true;
                        }
                        cam.orbit(
                            -(p.x - last.x) as f32 * 0.005,
                            (p.y - last.y) as f32 * 0.005,
                        );
                        *last = p;
                        changed = true;
                    }
                }
                Event::Up(p) => {
                    if let Some((_, _, moved)) = drag.take() {
                        if !moved {
                            if let Some(action) = ui::action_at(p.x, p.y, W, H, disaster) {
                                match action {
                                    ui::Action::ZoomIn => {
                                        cam.zoom(1.5);
                                        changed = true;
                                    }
                                    ui::Action::ZoomOut => {
                                        cam.zoom(-1.5);
                                        changed = true;
                                    }
                                    ui::Action::Meteor if !disaster.active() => {
                                        if disaster.elapsed.is_some() {
                                            impact_start = None;
                                            disaster = disaster::Disaster::default();
                                        } else {
                                            impact_start = Some(now);
                                            disaster = disaster::Disaster::at(0.);
                                            audio.stop();
                                        }
                                        return_camera = None;
                                        info.clear();
                                        cam = Camera::overview();
                                        changed = true;
                                    }
                                    ui::Action::Coast => {
                                        return_camera.get_or_insert(cam);
                                        transition = Some((cam, Camera::preset(9), now));
                                        info.clear();
                                    }
                                    ui::Action::Survivors => {
                                        return_camera.get_or_insert(cam);
                                        transition = Some((
                                            cam,
                                            Camera {
                                                yaw: 0.3,
                                                pitch: 0.38,
                                                distance: 9.,
                                                target: scene::SURVIVOR_SITE,
                                            },
                                            now,
                                        ));
                                        info.clear();
                                    }
                                    _ => {}
                                }
                            } else if !info.is_empty() && ui::close_hit(p.x, p.y, W) {
                                info.clear();
                                if let Some(previous) = return_camera.take() {
                                    transition = Some((cam, previous, now));
                                }
                            } else if !ui::card_contains(p.x, p.y, W, &info) {
                                let visible_scene = &scenes[displayed_disaster.stage()];
                                let hit = render::pick(
                                    visible_scene,
                                    displayed_cam.ray(p.x as f32, p.y as f32, W, H),
                                    displayed_disaster,
                                );
                                info = hit
                                    .map(|h| describe(visible_scene.cubes[h.index].tag))
                                    .unwrap_or("")
                                    .into();
                                if let Some(bounds) =
                                    hit.and_then(|h| visible_scene.animal_bounds(h.index))
                                {
                                    return_camera.get_or_insert(cam);
                                    transition = Some((cam, displayed_cam.focus(bounds), now));
                                }
                            }
                            redraw = true;
                        }
                    }
                }
                Event::Scroll(d) => {
                    transition = None;
                    cam.zoom(d as f32 * 0.3);
                    changed = true;
                }
                Event::Key(key) => {
                    let held = keys.get_mut(key as usize);
                    if let Some(held) = held {
                        if *held {
                            continue;
                        } // OS repeat must not multiply camera speed.
                        *held = true;
                    }
                    if matches!(key, 0 | 2 | 13 | 1 | 123..=126 | 24 | 27) {
                        transition = None;
                    }
                    match key {
                        53 => {
                            if info.is_empty() {
                                break 'app;
                            } else {
                                info.clear();
                            }
                        }
                        7 => {
                            info.clear();
                            let previous = return_camera.take().unwrap_or_else(Camera::overview);
                            transition = Some((cam, previous, now));
                        }
                        8 => info.clear(),
                        46 => {
                            if let Err(e) = audio.toggle() {
                                info = format!("AUDIO|{e}");
                            }
                        }
                        15 => {
                            view = 1;
                            cam = Camera::overview();
                            return_camera = None;
                            transition = None;
                            impact_start = None;
                            disaster = disaster::Disaster::default();
                            info.clear();
                            changed = true;
                        }
                        18 | 19 | 20 | 21 | 23 | 22 | 26 | 28 | 25 | 29 => {
                            view = match key {
                                18 => 1,
                                19 => 2,
                                20 => 3,
                                21 => 4,
                                23 => 5,
                                22 => 6,
                                26 => 7,
                                28 => 8,
                                25 => 9,
                                _ => 12,
                            };
                            return_camera = None;
                            transition = None;
                            cam = Camera::preset(view);
                            info.clear();
                            changed = true;
                        }
                        5 => {
                            view = 13;
                            return_camera = None;
                            transition = None;
                            cam = Camera::preset(13);
                            info.clear();
                            changed = true;
                        }
                        9 => {
                            view = 14;
                            return_camera = None;
                            transition = None;
                            cam = Camera::preset(14);
                            info.clear();
                            changed = true;
                        }
                        11 | 17 => {
                            view = if key == 11 { 15 } else { 16 };
                            return_camera = None;
                            transition = None;
                            cam = Camera::preset(view);
                            info.clear();
                            changed = true;
                        }
                        0 | 123 => {
                            cam.orbit(-0.025, 0.);
                            changed = true;
                        }
                        2 | 124 => {
                            cam.orbit(0.025, 0.);
                            changed = true;
                        }
                        13 | 126 => {
                            cam.orbit(0., 0.025);
                            changed = true;
                        }
                        1 | 125 => {
                            cam.orbit(0., -0.025);
                            changed = true;
                        }
                        24 => {
                            cam.zoom(0.5);
                            changed = true;
                        }
                        27 => {
                            cam.zoom(-0.5);
                            changed = true;
                        }
                        _ => {}
                    }
                    redraw = true;
                }
            }
        }
        let horizontal = (keys[2] || keys[124]) as i32 - (keys[0] || keys[123]) as i32;
        let vertical = (keys[13] || keys[126]) as i32 - (keys[1] || keys[125]) as i32;
        if horizontal != 0 || vertical != 0 {
            cam.orbit(horizontal as f32 * dt * 1.2, vertical as f32 * dt * 0.9);
            changed = true;
        }
        if keys[24] || keys[27] {
            cam.zoom((keys[24] as i32 - keys[27] as i32) as f32 * dt * 10.);
            changed = true;
        }
        if let Some((from, to, started)) = transition {
            let t = now.duration_since(started).as_secs_f32() / 0.55;
            cam = from.interpolate(to, t);
            changed = true;
            if t >= 1. {
                transition = None;
            }
        }
        if let Some(started) = impact_start {
            if disaster.active() && now.duration_since(last_animation) >= Duration::from_millis(40)
            {
                disaster = disaster::Disaster::at(now.duration_since(started).as_secs_f32());
                last_animation = now;
                changed = true;
            }
        }
        if changed {
            revision = renderer.invalidate();
            last_motion = now;
        }
        if let Err(e) = audio.tick() {
            info = format!("AUDIO|{e}");
            redraw = true;
        }
        if let Some(frame) = renderer.poll() {
            if !frame.full {
                // Adjust only the moving preview; the settled image keeps full quality.
                if frame.seconds > 0.055 {
                    divisor = (divisor + 1).min(5);
                } else if frame.seconds < 0.020 {
                    divisor = divisor.saturating_sub(1).max(2);
                }
            }
            // Never display a pre-impact frame after a reset, or extinct fauna after the scene swap.
            if (frame.disaster.elapsed.is_some() == disaster.elapsed.is_some())
                && frame.disaster.stage() == disaster.stage()
                && (!frame.full || frame.generation == revision)
            {
                if let Some(pixels) = frame.pixels {
                    background = Some(pixels);
                    displayed_cam = frame.camera;
                    displayed_disaster = frame.disaster;
                    redraw = true;
                    count += 1;
                }
            }
        }
        if redraw {
            if let Some(background) = &background {
                let mut pixels = background.clone();
                ui::overlay(&mut pixels, W, H, audio.enabled, &info, view);
                ui::controls(&mut pixels, W, H, disaster, return_camera.is_some());
                window.present(&pixels);
            }
            redraw = false;
        }
        if max_frames.is_some_and(|n| count >= n) {
            break;
        }
        if !renderer.busy {
            let settled = !disaster.active()
                && transition.is_none()
                && now.duration_since(last_motion) >= Duration::from_millis(180);
            if settled && final_revision != Some(revision) {
                renderer.request(
                    cam,
                    revision,
                    start.elapsed().as_secs_f32(),
                    true,
                    1,
                    disaster,
                );
                final_revision = Some(revision);
                preview_revision = Some(revision);
            } else if preview_revision != Some(revision) {
                renderer.request(
                    cam,
                    revision,
                    start.elapsed().as_secs_f32(),
                    false,
                    divisor,
                    disaster,
                );
                preview_revision = Some(revision);
            } else if max_frames.is_some() {
                revision = renderer.invalidate();
            }
        }
        std::thread::sleep(Duration::from_millis(4));
    }
    Ok(())
}
