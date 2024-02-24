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
    pub fn new(context: &Context, position: Vector3<f32>, color: Srgba) -> Self {
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
            mass: 1.0,
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
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut red_particles = initialize_particles(&context, Srgba::RED, 2);
    let mut green_particles = initialize_particles(&context, Srgba::GREEN, 2);
    let mut blue_particles = initialize_particles(&context, Srgba::BLUE, 2);

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        let time = (frame_input.accumulated_time * 0.001) as f32;
        rule(&mut red_particles, &green_particles, -1.0, time);
        rule(&mut green_particles, &red_particles, -1.0, time);
        rule(&mut blue_particles, &red_particles, -1.0, time);
        rule(&mut red_particles, &blue_particles, -1.0, time);
        rule(&mut blue_particles, &green_particles, -1.0, time);
        rule(&mut green_particles, &blue_particles, -1.0, time);

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

fn initialize_particles(context: &Context, color: Srgba, amount: usize) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..amount {
        let factor = 20.0;
        let x = rand::random::<f32>() * factor;
        let y = rand::random::<f32>() * factor;
        let z = rand::random::<f32>() * factor;

        particles.push(Particle::new(context, vec3(x, y, z), color));
    }
    particles
}

fn rule(
    affected_particles: &mut Vec<Particle>,
    acting_particles: &Vec<Particle>,
    g: f32,
    time: f32,
) {
    for affected_particle in affected_particles {
        let mut force = vec3(0.0, 0.0, 0.0);

        for acting_particles in acting_particles {
            let distance = affected_particle.position - acting_particles.position;
            let distance_squared = distance.dot(distance);
            if distance_squared > 0.0001 {
                let force_magnitude = g / distance_squared;
                force += distance * force_magnitude;
            }

            affected_particle.velocity = affected_particle.velocity + force;
                        let border = 10.0;

            if affected_particle.position.x > border || affected_particle.position.x < -border {
                affected_particle.velocity.x = affected_particle.velocity.x * -1.0;
            }

            if affected_particle.position.y > border || affected_particle.position.y < -border {
                affected_particle.velocity.y = affected_particle.velocity.y * -1.0;
            }

            if affected_particle.position.z > border || affected_particle.position.z < -border {
                affected_particle.velocity.z = affected_particle.velocity.z * -1.0;
            }

            let mut new_position = affected_particle.position + affected_particle.velocity * time;
            
            if new_position.x > border {
                new_position.x = border;
            }
            if new_position.x < -border {
                new_position.x = -border;
            }
            if new_position.y > border {
                new_position.y = border;
            }
            if new_position.y < -border {
                new_position.y = -border;
            }
            if new_position.z > border {
                new_position.z = border;
            }
            if new_position.z < -border {
                new_position.z = -border;
            }

            affected_particle.position = new_position;
            affected_particle
                .sphere
                .set_transformation(Mat4::from_translation(affected_particle.position));


        }
    }
}
