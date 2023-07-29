use bevy::{math::vec3, prelude::*};
use enterpolation::{linear::Linear, Curve};

#[derive(Component)]
pub struct PlaneSpec {
    pub name: String,
    pub thrust: f32,
    pub fuselage: FuselageSpec,
    pub wings: WingSpec,
    pub tail: TailSpec,
}

impl PlaneSpec {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..default()
        }
    }
}

impl Default for PlaneSpec {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            thrust: 150.0,
            fuselage: FuselageSpec {
                size: vec3(1.12, 2.0, 5.3),
            },
            wings: WingSpec {
                size: vec3(5.5, 0.2, 1.5),
                lift_coefficient_elements: vec![0.0, 0.0, 0.35, 1.4, 0.8, 0.0],
                lift_coefficient_knots: vec![-90.0, -5.0, 0.0, 10.0, 15.0, 90.0],
            },
            tail: TailSpec {
                size: vec3(0.25, 0.25, 3.0),
                vertical: WingSpec {
                    size: vec3(0.1, 2., 0.5),
                    lift_coefficient_elements: vec![-0.0, -0.01, 0.0, 0.0, 0.0, 0.01, 0.0],
                    lift_coefficient_knots: vec![-90.0, -10.0, -2.5, 0.0, 2.5, 10.0, 90.0],
                },
                horizontal: WingSpec {
                    size: vec3(2., 0.2, 1.0),
                    lift_coefficient_elements: vec![-0.0, -0.15, 0.10, 0.15, 0.0],
                    lift_coefficient_knots: vec![-90.0, -10.0, 0.0, 10.0, 90.0],
                },
            },
        }
    }
}

pub struct WingSpec {
    pub size: Vec3,
    pub lift_coefficient_elements: Vec<f32>,
    pub lift_coefficient_knots: Vec<f32>,
}

impl WingSpec {
    pub fn lift_coefficient_curve(
        &self,
    ) -> Linear<enterpolation::Sorted<Vec<f32>>, Vec<f32>, enterpolation::Identity> {
        Linear::builder()
            .elements(self.lift_coefficient_elements.clone())
            .knots(self.lift_coefficient_knots.clone())
            .build()
            .unwrap()
    }

    pub fn lift_coefficient_samples(&self) -> Vec<f32> {
        self.lift_coefficient_curve().take(180).collect()
    }
}

pub struct FuselageSpec {
    pub size: Vec3,
}

pub struct TailSpec {
    pub size: Vec3,
    pub vertical: WingSpec,
    pub horizontal: WingSpec,
}
