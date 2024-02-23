use three_d::{Context, Camera, Window, WindowSettings, ClearState, FrameOutput, OrbitControl, DirectionalLight, Mesh, CpuMesh, CpuMaterial, PhysicalMaterial, Gm, Srgba, Vector3, Mat4, vec3, degrees};
use rand;

struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
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
            acceleration: vec3(0.0, 0.0, 0.0),
            mass: 1.0,
            sphere 
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

    let red = Srgba::new(255, 0, 0, 200);
    let green = Srgba::new(0, 255, 0, 200);
    let blue = Srgba::new(0, 0, 255, 200);

    let red_particles = initialize_particles(&context, red, 10);
    let green_particles = initialize_particles(&context, green, 10);
    let blue_particles = initialize_particles(&context, blue, 10);

    let particles = red_particles.into_iter()
        .chain(green_particles.into_iter())
        .chain(blue_particles.into_iter())
        .collect::<Vec<_>>();

    let spheres = particles.into_iter().map(|p| p.sphere).collect::<Vec<_>>();

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera,
                &spheres,
                &[&light0, &light1],
            );

        FrameOutput::default()
    });
}

fn initialize_particles(context: &Context, color: Srgba, amount: usize) -> Vec<Particle> {
    let mut particles = Vec::new();
    for _ in 0..amount {
        let x = (rand::random::<f32>() - 0.5) * 10.0;
        let y = (rand::random::<f32>() - 0.5) * 10.0;
        let z = (rand::random::<f32>() - 0.5) * 10.0;

        particles.push(Particle::new(context, vec3(x, y, z), color));
    }
    particles
}

//fn rule(affected_particles: Vec<Particle>, acting_particles: Vec<Particle>, g: f32) -> Vec<Particle>{
//    for affected_particles in &affected_particles {
//        let mut force = vec3(0.0, 0.0, 0.0);
//        
//        for acting_particle_transform in &acting_particle_transforms {
//            let distance = affected_particle_transform - acting_particle_transform;
//            let distance_squared = distance.dot(&distance);
//            if distance_squared > 0.0001 {
//                let force_magnitude = g / distance_squared;
//                force += distance.normalize() * force_magnitude;
//            }
//        }
//    }
//}