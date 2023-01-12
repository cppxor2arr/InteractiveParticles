use egui_macroquad::{
    egui,
    macroquad::{self, prelude::*},
};
use std::f32::consts::PI;

#[macroquad::main("Particle Interaction")]
async fn main() {
    let bounds = Bounds {
        bottom_left: vec2(-screen_width() / 2.0, -screen_height() / 2.0),
        top_right: vec2(screen_width() / 2.0, screen_height() / 2.0),
    };
    let mut config = Config {
        simulation_speed: 1,
        num_particles: 10000,
        particle_radius: 1.2,
        interact_force: 5.5,
        drag: 2.0,
        trail_length: 66.0,
    };
    let mut particles = initialize_particles(&bounds, config.num_particles);

    let render_target = render_target(screen_width() as u32, screen_height() as u32);
    let texture_camera = {
        let mut camera = Camera2D::from_display_rect(Rect::new(
            -screen_width() / 2.0,
            -screen_height() / 2.0,
            screen_width(),
            screen_height(),
        ));
        camera.render_target = Some(render_target);
        camera
    };

    loop {
        if user_quit() {
            break;
        }
        config_ui(&mut config, &bounds, &mut particles);

        for _ in 0..config.simulation_speed {
            update_particles(
                &mut particles,
                get_frame_time(),
                convert_interact_force(config.interact_force),
                convert_drag(config.drag),
                &bounds,
                |screen| {
                    let mut world = texture_camera.screen_to_world(screen);
                    world.y *= -1.0;
                    world
                },
            );
        }

        // drawing to texture
        set_camera(&texture_camera);
        draw_rectangle(
            bounds.bottom_left.x,
            bounds.bottom_left.y,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, convert_trail_length(config.trail_length)),
        );
        draw_particles(&particles, config.particle_radius);

        // drawing to the screen
        set_default_camera();
        clear_background(BLACK);
        draw_texture(render_target.texture, 0.0, 0.0, WHITE);
        let fps_text = format!("FPS: {}", get_fps());
        let dimensions = measure_text(&fps_text, None, 30, 1.0);
        draw_text(
            &fps_text,
            screen_width() - (dimensions.width + 5.0),
            dimensions.height + 5.0,
            30.0,
            WHITE,
        );
        egui_macroquad::draw();

        next_frame().await;
    }
}

fn initialize_particles(bounds: &Bounds, num_particles: usize) -> Vec<Particle> {
    (0..num_particles)
        .map(|_| Particle {
            pos: vec2(
                rand::gen_range(bounds.bottom_left.x, bounds.top_right.x),
                rand::gen_range(bounds.bottom_left.y, bounds.top_right.y),
            ),
            vel: vec2(0.0, 0.0),
            acc: vec2(0.0, 0.0),
        })
        .collect()
}

fn update_particles(
    particles: &mut [Particle],
    dt: f32,
    interact_force: f32,
    drag: f32,
    bounds: &Bounds,
    screen_to_world: impl Fn(Vec2) -> Vec2,
) {
    let attract = is_key_down(KeyCode::Z);
    let repel = is_key_down(KeyCode::X);
    let swirl = is_key_down(KeyCode::C);
    let is_interacting = attract || repel || swirl;

    for p in particles {
        // bounce off walls
        if p.pos.x < bounds.bottom_left.x || p.pos.x > bounds.top_right.x {
            p.pos.x = p.pos.x.clamp(bounds.bottom_left.x, bounds.top_right.x);
            p.vel.x *= -1.0;
        }
        if p.pos.y < bounds.bottom_left.y || p.pos.y > bounds.top_right.y {
            p.pos.y = p.pos.y.clamp(bounds.bottom_left.y, bounds.top_right.y);
            p.vel.y *= -1.0;
        }

        // mouse interaction
        p.acc = vec2(0.0, 0.0);
        if is_interacting {
            let interact_point = screen_to_world(mouse_position().into());
            let diff = p.pos - interact_point;
            let distance_squared = diff.length_squared();
            let mut acc = interact_force * diff / distance_squared / distance_squared.sqrt();
            acc = acc.clamp_length_max(2000.0);

            if attract {
                p.acc -= acc;
            }
            if repel {
                p.acc += acc;
            }
            if swirl {
                p.acc += Mat2::from_angle(PI / 2.0).mul_vec2(acc);
            }
        }

        // drag
        p.acc -= drag * p.vel.length() * p.vel;

        // motion
        p.vel += p.acc * dt;
        p.pos += p.vel * dt;
    }
}

fn draw_particles(particles: &[Particle], radius: f32) {
    for p in particles {
        draw_circle(p.pos.x, p.pos.y, radius, GREEN);
    }
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
}

struct Bounds {
    bottom_left: Vec2,
    top_right: Vec2,
}

struct Config {
    simulation_speed: u32,
    num_particles: usize,
    particle_radius: f32,
    interact_force: f32,
    drag: f32,
    trail_length: f32,
}

impl Config {
    const MAX_SIMULATION_SPEED: u32 = 3;
    const MIN_NUM_PARTICLES: usize = 1;
    const MAX_NUM_PARTICLES: usize = 20000;
    const MIN_PARTICLE_RADIUS: f32 = 1.0;
    const MAX_PARTICLE_RADIUS: f32 = 5.0;
    const MIN_INTERACT_FORCE: f32 = 1.0;
    const MAX_INTERACT_FORCE: f32 = 10.0;
    const MIN_DRAG: f32 = 0.0;
    const MAX_DRAG: f32 = 10.0;
    const MAX_TRAIL_LENGTH: f32 = 100.0;
}

fn convert_interact_force(interact_force: f32) -> f32 {
    const MIN_VAL: f32 = 100000.0;
    const MAX_VAL: f32 = 1900000.0;
    MIN_VAL
        + (MAX_VAL - MIN_VAL) / (Config::MAX_INTERACT_FORCE - Config::MIN_INTERACT_FORCE)
            * (interact_force - Config::MIN_INTERACT_FORCE)
}

fn convert_trail_length(trail_length: f32) -> f32 {
    const TRAIL_LENGTH_POW: f32 = 0.05;
    1.0 - ((0.8 * trail_length) / Config::MAX_TRAIL_LENGTH).powf(TRAIL_LENGTH_POW)
}

fn convert_drag(drag: f32) -> f32 {
    const MIN_VAL: f32 = 0.0;
    const MAX_VAL: f32 = 0.01;
    MIN_VAL
        + (MAX_VAL - MIN_VAL) / (Config::MAX_DRAG - Config::MIN_DRAG) * (drag - Config::MIN_DRAG)
}

fn user_quit() -> bool {
    is_key_released(KeyCode::Q)
}

fn config_ui(config: &mut Config, bounds: &Bounds, particles: &mut Vec<Particle>) {
    egui_macroquad::ui(|ctx| {
        egui::Area::new("parameters")
            .fixed_pos((0.0, 0.0))
            .show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(
                        &mut config.simulation_speed,
                        1..=Config::MAX_SIMULATION_SPEED,
                    )
                    .text("Simulation speed")
                    .text_color(egui::Color32::WHITE)
                    .suffix("x"),
                );
                ui.add(
                    egui::Slider::new(
                        &mut config.num_particles,
                        Config::MIN_NUM_PARTICLES..=Config::MAX_NUM_PARTICLES,
                    )
                    .text("Number of particles")
                    .text_color(egui::Color32::WHITE),
                );
                ui.add(
                    egui::Slider::new(
                        &mut config.particle_radius,
                        Config::MIN_PARTICLE_RADIUS..=Config::MAX_PARTICLE_RADIUS,
                    )
                    .text("Particle size")
                    .text_color(egui::Color32::WHITE),
                );
                ui.add(
                    egui::Slider::new(
                        &mut config.interact_force,
                        Config::MIN_INTERACT_FORCE..=Config::MAX_INTERACT_FORCE,
                    )
                    .text("Interaction force")
                    .text_color(egui::Color32::WHITE),
                );
                ui.add(
                    egui::Slider::new(&mut config.drag, Config::MIN_DRAG..=Config::MAX_DRAG)
                        .text("Drag")
                        .text_color(egui::Color32::WHITE),
                );
                ui.add(
                    egui::Slider::new(&mut config.trail_length, 0.0..=Config::MAX_TRAIL_LENGTH)
                        .text("Trail length")
                        .text_color(egui::Color32::WHITE),
                );
                if ui.add(egui::Button::new("Reset")).clicked() {
                    *particles = initialize_particles(bounds, config.num_particles);
                }
            });
    });
}
