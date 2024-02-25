use rand;
use three_d::{
    degrees, vec3, Camera, ClearState, Context, CpuMaterial, CpuMesh, DirectionalLight, Event,
    FrameOutput, Gm, InnerSpace, Mat4, Mesh, OrbitControl, PhysicalMaterial, Srgba, Vector3,
    Window, WindowSettings,
};

struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    mass: f32,
    sphere: Gm<Mesh, PhysicalMaterial>,
}

impl Particle {
    pub fn new(context: &Context, position: Vector3<f32>, mass: f32, color: Srgba) -> Self {
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
            position: position,
            velocity: vec3(0.0, 0.0, 0.0),
            mass,
            sphere,
        }
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

    let amount = 100;
    let mut red_particles = initialize_particles(&context, 1.0, Srgba::RED, amount);
    let mut green_particles = initialize_particles(&context, 1.5, Srgba::GREEN, amount);
    let mut blue_particles = initialize_particles(&context, 2.0, Srgba::BLUE, amount);

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        apply_bidirectional_gravity_rule(&mut red_particles, &mut green_particles, 1.0);
        apply_bidirectional_gravity_rule(&mut red_particles, &mut blue_particles, 1.0);
        apply_bidirectional_gravity_rule(&mut blue_particles, &mut green_particles, 1.0);
        bidirectional_gravity_rule(&mut red_particles, &red_particles, 1.0);
        bidirectional_gravity_rule(&mut blue_particles, &blue_particles, 1.0);
        bidirectional_gravity_rule(&mut green_particles, &green_particles, 1.0);

        let spheres = red_particles
            .iter()
            .chain(green_particles.iter())
            .chain(blue_particles.iter())
            .map(|p| &p.sphere)
            .collect::<Vec<_>>();

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, &spheres, &[&light0, &light1]);

        FrameOutput::default()
    });
}

fn initialize_particles(context: &Context, mass: f32, color: Srgba, amount: usize) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..amount {
        let factor = 20.0;
        let x = rand::random::<f32>() * factor;
        let y = rand::random::<f32>() * factor;
        let z = rand::random::<f32>() * factor;

        particles.push(Particle::new(context, vec3(x, y, z), mass, color));
    }
    particles
}

fn apply_bidirectional_gravity_rule(
    particles_0: &mut Vec<Particle>,
    particles_1: &mut Vec<Particle>,
    g: f32,
) {
    
    bidirectional_gravity_rule(particles_0, particles_1, g);
    bidirectional_gravity_rule(particles_1, particles_0, g);
}

fn bidirectional_gravity_rule(
    affected_particles: &mut Vec<Particle>,
    acting_particles: &Vec<Particle>,
    g: f32,
) {
    for affected_particle in affected_particles {
        for acting_particle in acting_particles {
           gravity_rule(affected_particle, acting_particle, g); 
        }
    }
}

fn unidirectional_gravity_rule(particles: &mut Vec<Particle>, g: f32) {
    for i in 0..particles.len() {
        for j in 0..particles.len() {
            if i != j {
                gravity_rule(&mut particles[i], &particles[j], g);
            }
        }
    }
}

fn gravity_rule(particle_0: &mut Particle, particle_1: &Particle, g: f32) {
    let border = 100.0;
    let throttle = 0.005;

    let distance = particle_0.position - particle_1.position;
    let distance_squared = distance.dot(distance);
    let mut directed_acceleration = vec3(0.0, 0.0, 0.0);
    if distance_squared > 0.0001 {
        let acceleration = g * particle_1.mass / distance_squared;
        directed_acceleration = distance.normalize() * acceleration * throttle;
    }

    particle_0.velocity = particle_0.velocity + directed_acceleration;

    if particle_0.position.x.abs() > border {
       particle_0.velocity.x *= -1.0;
    }

    if particle_0.position.y.abs() > border {
       particle_0.velocity.y *= -1.0;
    }

    if particle_0.position.z.abs() > border {
       particle_0.velocity.z *= -1.0;
    }

    particle_0.position = particle_0.position + particle_0.velocity;
    particle_0 
        .sphere
        .set_transformation(Mat4::from_translation(particle_0.position));
}