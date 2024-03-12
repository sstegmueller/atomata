mod parameters;
mod particle;
mod sphere;

use parameters::Parameters;
use particle::Particle;
use sphere::Sphere;
use three_d::{
    degrees,
    egui::{SidePanel, Slider},
    vec3, Camera, ClearState, Context, DirectionalLight, FrameOutput, OrbitControl, Srgba, Window,
    WindowSettings,
};

// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();

    use log::info;

    info!("Logging works!");

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run();
    Ok(())
}

pub fn run() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, 2.0, 2.5),
        vec3(0.0, 0.0, -0.5),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 1000.0);

    let mut parameters = Parameters {
        amount: 10,
        border: 200.0,
        mass_red: 3.0,
        mass_green: 250.0,
        mass_blue: 1000.0,
    };

    let (
        mut red_particles,
        mut green_particles,
        mut blue_particles,
    ) = create_particles(&context, &parameters);
    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    let mut gui = three_d::GUI::new(&context);

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        apply_mutual_gravity_rule(&mut red_particles, &mut green_particles, 1.0);
        apply_mutual_gravity_rule(&mut red_particles, &mut blue_particles, 1.0);
        apply_mutual_gravity_rule(&mut blue_particles, &mut green_particles, 1.0);
        apply_identity_gravity_rule(&mut red_particles, 1.0);
        apply_identity_gravity_rule(&mut blue_particles, 1.0);
        apply_identity_gravity_rule(&mut green_particles, 1.0);

        for particle in red_particles
            .iter_mut()
            .chain(green_particles.iter_mut())
            .chain(blue_particles.iter_mut())
        {
            particle.apply_friction(0.005);
            particle.update_position(0.0002, &parameters);
        }

        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Parameters");
                    ui.add(Slider::new(&mut parameters.amount, 1..=200).text("Amount"));
                    if ui.button("Reset").clicked() {
                        let (
                            new_red_particles,
                            new_green_particles,
                            new_blue_particles,
                        ) = create_particles(&context, &parameters);
                        red_particles = new_red_particles;
                        green_particles = new_green_particles;
                        blue_particles = new_blue_particles;
                    };
                    ui.add(Slider::new(&mut parameters.border, 50.0..=500.0).text("Border"));
                    ui.add(Slider::new(&mut parameters.mass_red, 1.0..=5000.0).text("Mass Red"));
                    ui.add(
                        Slider::new(&mut parameters.mass_green, 1.0..=5000.0).text("Mass Green"),
                    );
                    ui.add(Slider::new(&mut parameters.mass_blue, 1.0..=5000.0).text("Mass Blue"));
                });
                panel_width = gui_context.used_rect().width();
            },
        );

        let spheres = red_particles
            .iter()
            .chain(green_particles.iter())
            .chain(blue_particles.iter())
            .map(|p| p.positionable.get_geometry())
            .collect::<Vec<_>>();

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, &spheres, &[&light0, &light1])
            .write(|| gui.render());

        FrameOutput::default()
    });
}

fn create_particles(
    context: &Context,
    parameters: &Parameters,
) -> (
    Vec<Particle>,
    Vec<Particle>,
    Vec<Particle>,
) {
    let red_particles = initialize_particle_kind(
        context,
        parameters.border,
        3.0,
        Srgba::RED,
        parameters.amount,
    );
    let green_particles = initialize_particle_kind(
        context,
        parameters.border,
        250.0,
        Srgba::GREEN,
        parameters.amount,
    );
    let blue_particles = initialize_particle_kind(
        context,
        parameters.border,
        10000.0,
        Srgba::BLUE,
        parameters.amount,
    );
    (
        red_particles,
        green_particles,
        blue_particles,
    )
}

fn initialize_particle_kind<'a>(
    context: &'a Context,
    border: f32,
    mass: f32,
    color: Srgba,
    amount: usize,
) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..amount {
        let sphere = Sphere::new(context, color);
        particles.push(Particle::new(Box::new(sphere), border, mass));
    }
    particles
}

fn apply_mutual_gravity_rule(
    particles_0: &mut Vec<Particle>,
    particles_1: &mut Vec<Particle>,
    g: f32,
) {
    mutual_gravity_rule(particles_0, particles_1, g);
    mutual_gravity_rule(particles_1, particles_0, g);
}

fn mutual_gravity_rule(
    affected_particles: &mut Vec<Particle>,
    acting_particles: &Vec<Particle>,
    g: f32,
) {
    for affected_particle in affected_particles {
        for acting_particle in acting_particles {
            affected_particle.update_velocity(acting_particle.position, acting_particle.mass, g);
        }
    }
}

fn apply_identity_gravity_rule(particles: &mut [Particle], g: f32) {
    let postion_clones = particles.iter().map(|p| p.position).collect::<Vec<_>>();
    let mass_clones = particles.iter().map(|p| p.mass).collect::<Vec<_>>();
    let len = particles.len();
    for (i, particle) in particles.iter_mut().enumerate() {
        for j in 0..len {
            if i == j {
                continue;
            }
            particle.update_velocity(postion_clones[j], mass_clones[j], g);
        }
    }
}
