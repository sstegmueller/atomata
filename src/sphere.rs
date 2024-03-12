use three_d::{Context, CpuMaterial, CpuMesh, Gm, Mat4, Mesh, PhysicalMaterial, Srgba, Vector3};

pub trait PositionableRender {
    fn set_position(&mut self, position: Vector3<f32>);
    fn get_geometry(&self) -> &Gm<Mesh, PhysicalMaterial>;
}

pub struct Sphere {
    pub geometry: Gm<Mesh, PhysicalMaterial>,
}

impl Sphere {
    pub fn new(context: &Context, color: Srgba) -> Self {
        let geometry = Gm::new(
            Mesh::new(context, &CpuMesh::sphere(16)),
            PhysicalMaterial::new_transparent(
                context,
                &CpuMaterial {
                    albedo: color,
                    ..Default::default()
                },
            ),
        );

        Self { geometry }
    }
}

impl PositionableRender for Sphere {
    fn set_position(&mut self, position: Vector3<f32>) {
        self.geometry
            .set_transformation(Mat4::from_translation(position));
    }
    fn get_geometry(&self) -> &Gm<Mesh, PhysicalMaterial> {
        &self.geometry
    }
}
