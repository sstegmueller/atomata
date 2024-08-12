use std::fmt::{Display, Formatter};

pub enum Mode {
    Default, // < Default mode with graphical user interface and rendering
    #[allow(dead_code)]
    Search, // < No graphical user interface and no rendering, only simulation and persistence of data
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InteractionType {
    Attraction,
    Repulsion,
    Neutral,
}

impl Display for InteractionType{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Debug)]
pub struct ParticleParameters {
    pub mass: f32,
    pub index: usize,
}

pub struct Parameters {
    pub amount: usize,
    pub border: f32,
    pub timestep: f32,
    pub gravity_constant: f32,
    pub friction: f32,
    pub particle_parameters: Vec<ParticleParameters>,
    pub interactions: Vec<InteractionType>,
    pub max_velocity: f32,
    pub bucket_size: f32,
    pub mode: Mode,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            amount: 10,
            border: 200.0,
            friction: 0.005,
            timestep: 0.0002,
            gravity_constant: 1.0,
            particle_parameters: vec![
                ParticleParameters {
                    mass: 3.0,
                    index: 0,
                },
                ParticleParameters {
                    mass: 250.0,
                    index: 1,
                },
                ParticleParameters {
                    mass: 1000.0,
                    index: 2,
                },
            ],
            interactions: vec![
                InteractionType::Repulsion,  // 0 <-> 0
                InteractionType::Attraction, // 1 <-> 0
                InteractionType::Attraction, // 2 <-> 0
                InteractionType::Repulsion,  // 1 <-> 1
                InteractionType::Attraction, // 1 <-> 2
                InteractionType::Neutral,    // 2 <-> 2
            ],
            max_velocity: 20000.0,
            bucket_size: 10.0,
            mode: Mode::Default,
        }
    }
}

impl Parameters {
    /// Returns the interaction type between two particles given their indices from the
    /// flat symmetric triangle interactions matrix.
    ///
    /// Example:
    ///                     Index 0 1 2
    ///                       0   3 4 5
    ///  3 4 5 6 7 8  --->    1   4 6 7   
    ///                       2   5 7 8
    pub fn interaction_by_indices(&self, i: usize, j: usize) -> Result<InteractionType, String> {
        let num_particle_kinds = self.particle_parameters.len();
        if i > num_particle_kinds - 1 || j > num_particle_kinds - 1 {
            return Err("Index out of bounds".to_string());
        }

        let (i, j) = if i > j { (j, i) } else { (i, j) };
        let index = (i * (2 * num_particle_kinds - i + 1)) / 2 + (j - i);

        Ok(self.interactions[index])
    }

    pub fn parameter_space() -> Vec<Self> {
        vec![Parameters {
            amount: 10,
            border: 200.0,
            friction: 0.005,
            timestep: 0.0002,
            gravity_constant: 1.0,
            particle_parameters: vec![
                ParticleParameters {
                    mass: 3.0,
                    index: 0,
                },
                ParticleParameters {
                    mass: 250.0,
                    index: 1,
                },
                ParticleParameters {
                    mass: 1000.0,
                    index: 2,
                },
            ],
            interactions: vec![
                InteractionType::Repulsion,  // 0 <-> 0
                InteractionType::Attraction, // 1 <-> 0
                InteractionType::Attraction, // 2 <-> 0
                InteractionType::Repulsion,  // 1 <-> 1
                InteractionType::Attraction, // 1 <-> 2
                InteractionType::Neutral,    // 2 <-> 2
            ],
            max_velocity: 20000.0,
            bucket_size: 10.0,
            mode: Mode::Default,
        }]
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use pretty_assertions_sorted::assert_eq;

    fn test_parameters() -> Parameters {
        Parameters {
            amount: 10,
            border: 200.0,
            friction: 0.0,
            timestep: 0.0002,
            gravity_constant: 1.0,
            particle_parameters: vec![
                ParticleParameters {
                    mass: 3.0,
                    index: 0,
                },
                ParticleParameters {
                    mass: 250.0,
                    index: 1,
                },
                ParticleParameters {
                    mass: 10000.0,
                    index: 2,
                },
                ParticleParameters {
                    mass: 10000.0,
                    index: 3,
                },
            ],
            interactions: vec![
                InteractionType::Attraction, // 0 <-> 0
                InteractionType::Neutral,    // 1 <-> 0
                InteractionType::Repulsion,  // 2 <-> 0
                InteractionType::Repulsion,  // 3 <-> 0
                InteractionType::Neutral,    // 1 <-> 1
                InteractionType::Attraction, // 1 <-> 2
                InteractionType::Attraction, // 1 <-> 3
                InteractionType::Repulsion,  // 2 <-> 2
                InteractionType::Repulsion,  // 2 <-> 3
                InteractionType::Repulsion,  // 3 <-> 3
            ],
            max_velocity: 20000.0,
            bucket_size: 10.0,
            mode: Mode::Default,
        }
    }

    #[test]
    fn test_interaction_by_indices_success() {
        let parameters = test_parameters();

        assert_eq!(
            parameters.interaction_by_indices(0, 0).unwrap(),
            InteractionType::Attraction
        );
        assert_eq!(
            parameters.interaction_by_indices(1, 0).unwrap(),
            InteractionType::Neutral
        );
        assert_eq!(
            parameters.interaction_by_indices(2, 0).unwrap(),
            InteractionType::Repulsion
        );
        assert_eq!(
            parameters.interaction_by_indices(1, 1).unwrap(),
            InteractionType::Neutral
        );
        assert_eq!(
            parameters.interaction_by_indices(1, 2).unwrap(),
            InteractionType::Attraction
        );
        assert_eq!(
            parameters.interaction_by_indices(2, 2).unwrap(),
            InteractionType::Repulsion
        );
    }

    #[test]
    fn test_interaction_by_indices_failure() {
        let parameters = test_parameters();

        let one_off = parameters.particle_parameters.len();

        assert_eq!(
            parameters.interaction_by_indices(one_off, 1).unwrap_err(),
            "Index out of bounds"
        );
        assert_eq!(
            parameters.interaction_by_indices(1, one_off).unwrap_err(),
            "Index out of bounds"
        );
    }
}
