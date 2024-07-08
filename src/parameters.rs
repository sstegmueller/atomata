pub enum Mode {
    Default, // < Default mode with graphical user interface and rendering
    #[allow(dead_code)]
    Search, // < No graphical user interface and no rendering, only simulation and persistence of data
}

pub struct Parameters {
    pub amount: usize,
    pub border: f32,
    pub timestep: f32,
    pub gravity_constant: f32,
    pub friction: f32,
    pub mass_red: f32,
    pub mass_green: f32,
    pub mass_blue: f32,
    pub max_velocity: f32,
    pub database_path: String,
    pub bucket_size: f32,
    pub mode: Mode,
}
