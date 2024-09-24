mod parameters;
mod particle;
#[cfg(not(target_arch = "wasm32"))]
mod persistence;
mod sphere;

use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use argh::FromArgs;
use log::info;
use parameters::{Mode, Parameters};
use particle::{Particle, StateVector};
#[cfg(not(target_arch = "wasm32"))]
use persistence::{
    commit_transaction, create_transaction_provider, increment_state_count, migrate_to_latest,
    open_database, persist_parameters, TransactionProvider,
};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use sphere::{PositionableRender, Sphere};
use three_d::{
    degrees,
    egui::{SidePanel, Slider},
    vec3, Camera, ClearState, Context, DirectionalLight, FrameOutput, OrbitControl, Srgba, Window,
    WindowSettings,
};

#[cfg(not(target_arch = "wasm32"))]
const LOG_FILE_NAME: &str = "atomata.log";

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, FromArgs)]
#[argh(description = "command line interface arguments")]
struct Cli {
    #[argh(
        switch,
        short = 's',
        description = "wheter to run experiements over parameter space in headless mode"
    )]
    search: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn set_log_hook(log_file_path: &str) {
    use log::{error, LevelFilter};
    use std::{ops::Deref, panic};

    simple_logging::log_to_file(log_file_path, LevelFilter::Info)
        .expect("Can't initialize logging");

    panic::set_hook(Box::new(|panic_info| {
        let (filename, line) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line()))
            .unwrap_or(("<unknown>", 0));

        let cause = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(String::deref);

        let cause = cause.unwrap_or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or("<cause unknown>")
        });

        error!("A panic occurred at {}:{}: {}", filename, line, cause);
    }));
}

// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();

    info!("Logging works!");

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run();
    Ok(())
}

pub fn run() {
    let mut default_parameters = Parameters::default();

    #[cfg(not(target_arch = "wasm32"))]
    let args = argh::from_env::<Cli>();
    #[cfg(not(target_arch = "wasm32"))]
    let mode = match args.search {
        true => Mode::Search,
        false => Mode::Default,
    };
    #[cfg(target_arch = "wasm32")]
    let mode = Mode::Default;

    match mode {
        #[cfg(not(target_arch = "wasm32"))]
        Mode::Search => {
            info!("Running search mode");
            set_log_hook(LOG_FILE_NAME);
            info!("Initializing database...");
            let connection_provider = Arc::new(Mutex::new(open_database("./results.db3").unwrap()));

            info!("Migrating database...");
            {
                let mut connection = connection_provider.lock().unwrap();
                migrate_to_latest(&mut connection).unwrap();
            }

            let mut parameter_space = Parameters::parameter_space();

            info!("Persisting parameter space...");
            {
                let mut guard = connection_provider.lock().unwrap();
                let tx_provider = create_transaction_provider(&mut guard).unwrap();

                for parameters in parameter_space.iter_mut() {
                    persist_parameters(parameters, &tx_provider).unwrap();
                }

                tx_provider.commit().unwrap();
            }

            let size_parameter_space = parameter_space.len();
            let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
            let average_run_time = Arc::new(Mutex::new(0.0));

            // Iterate over parameters and perform the search in parallel
            parameter_space.par_iter().for_each(|parameters| {
                {
                    let counter = counter.lock().unwrap();
                    let average_run_time = average_run_time.lock().unwrap();
                    info!("Run {} / {}", *counter, size_parameter_space);
                    info!("Average run time: {:.2} s", *average_run_time);

                    let remaining_time_s =
                        *average_run_time * (size_parameter_space - *counter as usize) as f64;
                    // Print in HH:SS format
                    info!(
                        "Expected remaining time: {}:{} HH:MM",
                        (remaining_time_s / 3600.0) as u32,
                        ((remaining_time_s % 3600.0) / 60.0) as u32
                    );
                    info!("Parameters: {:?}", parameters);
                }
                let start_time = std::time::Instant::now();

                let mut particles = create_particles(None, &default_parameters);
                let iterations = 10000;

                // Perform the computation and persistence for each iteration
                let mut results: Vec<StateVector> = vec![];
                for _ in 0..iterations {
                    update_particles(&mut particles, &default_parameters).unwrap();
                    let mut state_vectors = particles
                        .iter()
                        .map(|p| {
                            let particle_parameters_id = parameters
                                .particle_parameters_by_index(p.index)
                                .unwrap()
                                .id
                                .unwrap();
                            p.to_state_vector(parameters.bucket_size, particle_parameters_id)
                        })
                        .collect::<Vec<_>>();
                    results.append(&mut state_vectors);
                }
                // Persist results sequentially/synchronous on the main thread
                let connection = Arc::clone(&connection_provider);
                let mut guard = connection.lock().unwrap();
                let tx_provider = create_transaction_provider(&mut guard).unwrap();
                for result in results {
                    increment_state_count(&result, &tx_provider).unwrap();
                }
                commit_transaction(tx_provider).unwrap();

                let mut counter = counter.lock().unwrap();
                *counter += 1;

                let elapsed_time = start_time.elapsed().as_secs_f64();
                let mut average_run_time = average_run_time.lock().unwrap();
                *average_run_time =
                    *average_run_time + (elapsed_time - *average_run_time) / (*counter as f64);
            });
        }
        #[cfg(target_arch = "wasm32")]
        Mode::Search => {
            // Search logic not supported in wasm architecture
            // Add appropriate error handling or fallback logic here
        }
        Mode::Default => {
            let window = Window::new(WindowSettings {
                title: "atomata".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            .unwrap();
            let context = window.gl();
            let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
            let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

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
            let mut gui = three_d::GUI::new(&context);

            let mut iteration_step = 0;

            let mut particles = create_particles(Some(&context), &default_parameters);
            window.render_loop(move |mut frame_input| {
                camera.set_viewport(frame_input.viewport);
                control.handle_events(&mut camera, &mut frame_input.events);

                update_particles(&mut particles, &default_parameters).unwrap();

                let mut panel_width = 0.0;
                gui.update(
                    &mut frame_input.events,
                    frame_input.accumulated_time,
                    frame_input.viewport,
                    frame_input.device_pixel_ratio,
                    |gui_context| {
                        SidePanel::left("side_panel").show(gui_context, |ui| {
                            ui.heading("Parameters");
                            ui.add(
                                Slider::new(&mut default_parameters.amount, 1..=500).text("Amount"),
                            );
                            if ui.button("Reset").clicked() {
                                particles = create_particles(Some(&context), &default_parameters);
                            };
                            ui.add(
                                Slider::new(&mut default_parameters.max_velocity, 50.0..=50000.0)
                                    .text("Max. velocity"),
                            );
                            ui.add(
                                Slider::new(&mut default_parameters.friction, 0.0..=0.01)
                                    .text("Friction"),
                            );
                            ui.add(
                                Slider::new(&mut default_parameters.border, 50.0..=500.0)
                                    .text("Border"),
                            );
                            ui.add(
                                Slider::new(&mut default_parameters.timestep, 0.0001..=0.001)
                                    .text("Timestep"),
                            );
                            ui.add(
                                Slider::new(&mut default_parameters.gravity_constant, 0.1..=20.0)
                                    .text("Gravity constant"),
                            );
                            for particle in default_parameters.particle_parameters.iter_mut() {
                                ui.collapsing(format!("Particle {}", particle.index), |ui| {
                                    ui.add(
                                        Slider::new(&mut particle.mass, 1.0..=10000.0).text("Mass"),
                                    );
                                });
                            }

                            ui.label(format!("Iteration step: {}", iteration_step));
                        });
                        panel_width = gui_context.used_rect().width();
                    },
                );

                let spheres = particles
                    .iter()
                    .map(|p| p.positionable.as_ref().unwrap().get_geometry())
                    .collect::<Vec<_>>();
                frame_input
                    .screen()
                    .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                    .render(&camera, &spheres, &[&light0, &light1])
                    .write(|| gui.render());

                iteration_step = iteration_step + 1;
                FrameOutput::default()
            });
        }
    }
}

/// Generates rgb n rgb color with the maximum possible contrast
fn generate_colors(num_colors: usize) -> Vec<Srgba> {
    let golden_ratio_conjugate = 0.618_034;
    let mut h = rand::random::<f32>(); // Start with a random hue
    let mut colors = Vec::with_capacity(num_colors);

    for _ in 0..num_colors {
        h += golden_ratio_conjugate;
        h %= 1.0;

        // HSV to RGB conversion
        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let p = 0.95 * (1.0 - 0.5);
        let q = 0.95 * (1.0 - f * 0.5);
        let t = 0.95 * (1.0 - (1.0 - f) * 0.5);

        let (r, g, b) = match i as u32 % 6 {
            0 => (0.95, t, p),
            1 => (q, 0.95, p),
            2 => (p, 0.95, t),
            3 => (p, q, 0.95),
            4 => (t, p, 0.95),
            _ => (0.95, p, q),
        };

        colors.push(Srgba::new(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            255,
        ));
    }

    colors
}

fn create_particles(context: Option<&Context>, parameters: &Parameters) -> Vec<Particle> {
    let mut particles: Vec<Particle> = Vec::new();
    let colors = generate_colors(parameters.particle_parameters.len());

    for (particle_params, color) in parameters.particle_parameters.iter().zip(colors) {
        let mut particle_kind = initialize_particle_kind(
            particle_params.index,
            context,
            parameters.border,
            particle_params.mass,
            color,
            parameters.amount,
            parameters.max_velocity,
        );
        particles.append(&mut particle_kind);
    }

    particles
}

fn initialize_particle_kind(
    id: usize,
    context: Option<&Context>,
    border: f32,
    mass: f32,
    color: Srgba,
    amount: usize,
    max_velocity: f32,
) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..amount {
        let positionable: Option<Box<dyn PositionableRender>> = match context {
            Some(context) => {
                let sphere = Sphere::new(context, color);
                Some(Box::new(sphere) as Box<dyn PositionableRender>)
            }
            None => None,
        };
        particles.push(Particle::new(id, positionable, border, mass, max_velocity));
    }
    particles
}

fn update_particles(particles: &mut [Particle], parameters: &Parameters) -> Result<(), String> {
    let id_clones = particles.iter().map(|p| p.index).collect::<Vec<_>>();
    let postion_clones = particles.iter().map(|p| p.position).collect::<Vec<_>>();
    let mass_clones = particles.iter().map(|p| p.mass).collect::<Vec<_>>();
    let len = particles.len();
    for (i, particle) in particles.iter_mut().enumerate() {
        for j in 0..len {
            if i == j {
                continue;
            }
            let interaction_type =
                parameters.interaction_by_indices(particle.index, id_clones[j])?;
            particle.update_velocity(
                postion_clones[j],
                mass_clones[j],
                interaction_type,
                parameters.gravity_constant,
            );
            particle.apply_friction(parameters.friction);
            particle.update_position(parameters);
        }
    }

    Ok(())
}
