mod sdf;

use crate::sdf::{Cube, Scene, Sdf, Sphere};
use raylib::prelude::*;
use rayon::prelude::*;
use std::time::{Duration, Instant};

const RENDER_VIEWPORT: i32 = 300;
const ACTUAL_VIEWPORT: i32 = 1500;

pub struct Camera {
    eye: Vector3,
    target: Vector3,
    up: Vector3,
}

pub struct LightSource {
    /// Position
    pos: Vector3,
    /// Specular intensity
    specular: Vector3,
    /// Diffuse intensity
    diffuse: Vector3,
}

pub struct Lighting {
    /// Ambient intensity
    ia: f32,
    /// Shininess reflection constant
    alpha: f32,
    /// Light sources
    light_sources: Vec<LightSource>,
}

impl Lighting {
    fn illuminate(&self, camera: &Camera, p: Vector3, object: &Box<dyn Sdf>) -> Vector3 {
        let mut ip = Vector3::default();

        let n = object.surface_normal(p);

        // View direction
        let v = (camera.eye - p).normalized();

        for ls in &self.light_sources {
            // Light direction
            let l = (ls.pos - p).normalized();
            // Reflection direction
            let r = (n * (l.dot(n)) * 2.0 - l).normalized();

            let diffuse_f = l.dot(n).max(0.0);
            let specular_f = v.dot(r).max(0.0).powf(self.alpha);

            ip += ls.diffuse * (diffuse_f + self.ia) + ls.specular * specular_f
        }

        ip *= 255.;
        ip.clamp(0. ..255.)
    }
}

fn v3_into_color(v: Vector3) -> Color {
    let (r, g, b) = (v.x, v.y, v.z);

    assert!(r >= 0. && g >= 0. && b >= 0.);
    assert!(r <= 255. && g <= 255. && b <= 255.);

    Color::new(r as u8, g as u8, b as u8, 255_u8)
}

fn main() {
    let (mut rl, thread) = init()
        .size(ACTUAL_VIEWPORT, ACTUAL_VIEWPORT)
        .title("Space")
        .build();

    rl.set_target_fps(30);

    let mut camera = Camera {
        eye: Vector3::new(0., 0., -2.),
        target: Vector3::new(0., 0., 0.),
        up: Vector3::new(0., -1., 0.),
    };

    let sphere = Box::new(Sphere {
        id: 0,
        center: Vector3::new(0., 0., 0.),
        radius: 1.,
    });

    let cube = Box::new(Cube {
        id: 1,
        center: Vector3::new(-2., 0., 0.),
        length: 1.,
    });

    let lighting = Lighting {
        ia: 0.1,
        alpha: 32.0,
        light_sources: vec![LightSource {
            pos: Vector3::new(5., 5., 5.),
            specular: Vector3::new(1., 1., 1.),
            diffuse: Vector3::new(1., 0.4, 0.2),
        }],
    };

    let scene = Scene::new(vec![cube, sphere]);

    while !rl.window_should_close() {
        check_movement(&mut camera, &rl);

        let mut draw_handle = rl.begin_drawing(&thread);
        draw_handle.clear_background(Color::BLACK);

        let frame_time = draw(&mut draw_handle, 90., &camera, &lighting, &scene);

        draw_handle.draw_text(
            &format!("Frame time: {0:#?}", frame_time),
            14,
            ACTUAL_VIEWPORT - (14 * 2),
            14,
            Color::WHITE,
        );
    }
}

fn draw(
    dh: &mut RaylibDrawHandle,
    fov: f32,
    camera: &Camera,
    lighting: &Lighting,
    scene: &Scene,
) -> Duration {
    let ratio = (RENDER_VIEWPORT as f32) / (RENDER_VIEWPORT as f32);
    let scale = ((fov * 0.5).to_radians()).tan();

    let forward = (camera.target - camera.eye).normalized();
    let right = forward.cross(camera.up).normalized();
    let true_up = right.cross(forward);

    let start = Instant::now();

    let x_pixels = 0..RENDER_VIEWPORT;
    let y_pixels = 0..RENDER_VIEWPORT;

    let mut pixels = Vec::with_capacity(x_pixels.len() * y_pixels.len());

    for x in x_pixels {
        for y in y_pixels.clone() {
            pixels.push((x, y));
        }
    }

    let to_draw = pixels
        .par_iter()
        .map(|(x, y)| {
            let x = *x;
            let y = *y;
            let x_normalized = (x as f32 + 0.5) / (RENDER_VIEWPORT as f32) * 2.0 - 1.0;
            let y_normalized = (y as f32 + 0.5) / (RENDER_VIEWPORT as f32) * 2.0 - 1.0;

            let pixel_camera_space =
                Vector3::new(x_normalized * ratio * scale, y_normalized * scale, 1.); // forward

            let ray = ((right * pixel_camera_space.x)
                + (true_up * pixel_camera_space.y)
                + (forward * pixel_camera_space.z))
                .normalized();

            if let Some((point, object_id)) = scene.ray_march(camera.eye, ray) {
                let object = scene.get_object(object_id);
                let color = lighting.illuminate(camera, point, object);

                Some((x, y, color))
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    let ratio = ACTUAL_VIEWPORT / RENDER_VIEWPORT;

    for (x, y, color) in to_draw {
        dh.draw_rectangle(
            x * ratio,
            y * ratio,
            ratio.max(1),
            ratio.max(1),
            v3_into_color(color),
        );
    }

    let end = Instant::now();

    end - start
}

fn check_movement(camera: &mut Camera, rl: &RaylibHandle) {
    macro_rules! key {
        ($key:expr) => {{
            let kbd: KeyboardKey = $key;
            rl.is_key_pressed(kbd) || rl.is_key_down(kbd)
        }};
    }

    // Move
    // Forward-Back
    if key!(KeyboardKey::KEY_W) {
        camera.eye.z += 0.1;
        camera.target.z += 0.1;
    }
    if key!(KeyboardKey::KEY_S) {
        camera.eye.z -= 0.1;
        camera.target.z -= 0.1;
    }

    // Left-right
    if key!(KeyboardKey::KEY_A) {
        camera.eye.x -= 0.1;
        camera.target.x -= 0.1;
    }
    if key!(KeyboardKey::KEY_D) {
        camera.eye.x += 0.1;
        camera.target.x += 0.1;
    }

    if key!(KeyboardKey::KEY_SPACE) {
        camera.eye.y += 0.1;
    }
    if key!(KeyboardKey::KEY_C) {
        camera.eye.y -= 0.1;
    }

    // Pan
    if key!(KeyboardKey::KEY_Q) {}
    if key!(KeyboardKey::KEY_E) {}
}
