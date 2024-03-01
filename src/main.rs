use three_d::{
    degrees, vec3, Camera, ClearState, Context, CpuMaterial, CpuMesh, DirectionalLight,
    FrameOutput, Gm, InnerSpace, Mat4, Mesh, OrbitControl, PhysicalMaterial, Srgba, Vector3,
    Window, WindowSettings,
    egui::{Slider, SidePanel}
};

struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    mass: f32,
    sphere: Gm<Mesh, PhysicalMaterial>,
}

impl Particle {
    pub fn new(context: &Context, border: f32, mass: f32, color: Srgba) -> Self {
        let factor = border / 2.0;
        // generate random position in the range of -1 to +1 time factor
        let x = (rand::random::<f32>() - 0.5) * factor;
        let y = (rand::random::<f32>() - 0.5) * factor;
        let z = (rand::random::<f32>() - 0.5) * factor;
        let position = vec3(x, y, z);

        let mut sphere = Gm::new(
            Mesh::new(context, &CpuMesh::sphere(16)),
            PhysicalMaterial::new_transparent(
                context,
                &CpuMaterial {
                    albedo: color,
                    ..Default::default()
                },
            ),
        );

        sphere.set_transformation(Mat4::from_translation(position) * Mat4::from_scale(0.2));

        Self {
            position,
            velocity: vec3(0.0, 0.0, 0.0),
            mass,
            sphere,
        }
    }

    pub fn update_velocity(
        &mut self,
        other_position: Vector3<f32>,
        other_mass: f32,
        gravity_constant: f32,
    ) {
        let distance = self.position - other_position;
        let distance_squared = distance.dot(distance);
        let mut directed_acceleration = vec3(0.0, 0.0, 0.0);
        if distance_squared > 0.0001 {
            let acceleration = gravity_constant * other_mass / distance_squared;
            directed_acceleration = distance.normalize() * acceleration;
        }

        self.velocity += directed_acceleration;
    }

    pub fn update_position(&mut self, time_step: f32) {
        self.position += self.velocity * time_step;
        self.sphere
            .set_transformation(Mat4::from_translation(self.position));
    }

    pub fn apply_friction(&mut self, friction: f32) {
        self.velocity *= 1.0 - friction;
    }
}

pub fn main() {
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

    let mut amount = 100;
    let mut border = 200.0;

    let mut red_particles = initialize_particles(&context, border, 3.0, Srgba::RED, amount);
    let mut green_particles = initialize_particles(&context, border, 250.0, Srgba::GREEN, amount);
    let mut blue_particles = initialize_particles(&context, border, 10000.0, Srgba::BLUE, 10);

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    let mut gui = three_d::GUI::new(&context);

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        apply_mutual_gravity_rule(&mut red_particles, &mut green_particles, -1.0);
        apply_mutual_gravity_rule(&mut red_particles, &mut blue_particles, -1.0);
        apply_mutual_gravity_rule(&mut blue_particles, &mut green_particles, -1.0);
        apply_identity_gravity_rule(&mut red_particles, -1.0);
        apply_identity_gravity_rule(&mut blue_particles, -1.0);
        apply_identity_gravity_rule(&mut green_particles, -1.0);

        for particle in red_particles
            .iter_mut()
            .chain(green_particles.iter_mut())
            .chain(blue_particles.iter_mut())
        {
            particle.apply_friction(0.005);
            particle.update_position(0.01);

            // apply spherical border collision
            let distance_from_center = particle.position.magnitude();

            if distance_from_center.abs() > border {
                particle.velocity = -particle.velocity;
            }
        }

        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");
                    ui.add(Slider::new(&mut amount, 1..=200).text("Amount"));
                    ui.add(Slider::new(&mut border, 50.0..=500.0).text("Border"));
                });
                panel_width = gui_context.used_rect().width();
            },
        );

        let spheres = red_particles
            .iter()
            .chain(green_particles.iter())
            .chain(blue_particles.iter())
            .map(|p| &p.sphere)
            .collect::<Vec<_>>();

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, &spheres, &[&light0, &light1])
            .write(|| gui.render());

        FrameOutput::default()
    });
}

fn initialize_particles(
    context: &Context,
    border: f32,
    mass: f32,
    color: Srgba,
    amount: usize,
) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..amount {
        particles.push(Particle::new(context, border, mass, color));
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

fn apply_identity_gravity_rule(particles: &mut Vec<Particle>, g: f32) {
    let postion_clones = particles.iter().map(|p| p.position).collect::<Vec<_>>();
    let mass_clones = particles.iter().map(|p| p.mass).collect::<Vec<_>>();
    let len = particles.len();
    for i in 0..len {
        for j in 0..len {
            if i == j {
                continue;
            }
            particles[i].update_velocity(postion_clones[j], mass_clones[j], g);
        }
    }
}
