use bevy::{math::vec3, prelude::*};
use enterpolation::{linear::Linear, Curve};

#[derive(Component, Debug, Clone)]
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
            thrust: 500.0,
            fuselage: FuselageSpec {
                size: vec3(1.12, 2.0, 5.3),
                mass: 100.0,
            },
            wings: WingSpec {
                size: vec3(5.5, 0.2, 1.5),
                lift_coefficient_curve: vec![
                    (0.0, -90.0),
                    (0.0, -5.0),
                    (0.35, 0.0),
                    (1.4, 10.0),
                    (0.8, 15.0),
                    (0.0, 90.0),
                ],
                ..default()
            },
            tail: TailSpec {
                size: vec3(0.25, 0.25, 3.0),
                vertical: WingSpec {
                    size: vec3(0.1, 2., 0.5),
                    lift_coefficient_curve: vec![
                        (-0.0, -90.0),
                        (-0.1, -10.0),
                        (0.0, -2.5),
                        (0.0, 0.0),
                        (0.0, 2.5),
                        (0.1, 10.0),
                        (0.0, 90.0),
                    ],
                    ..default()
                },
                horizontal: WingSpec {
                    size: vec3(2., 0.2, 1.0),
                    lift_coefficient_curve: vec![
                        (0.0, -90.0),
                        (-0.15, -10.0),
                        (-0.025, 0.0),
                        (0.25, 10.0),
                        (0.0, 90.0),
                    ],
                    ..default()
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct WingSpec {
    pub size: Vec3,
    pub lift_coefficient_curve: Vec<(f32, f32)>,
    pub drag_coefficient_curve: Vec<(f32, f32)>,
}

impl Default for WingSpec {
    fn default() -> Self {
        Self {
            size: vec3(2., 0.2, 1.0),
            lift_coefficient_curve: vec![(-0.0, -90.0), (-0.15, -10.0), (0.15, 10.0), (0.0, 90.0)],
            drag_coefficient_curve: vec![(0.032, -90.0), (0.032, 90.0)],
        }
    }
}

impl WingSpec {
    fn build_samples(curve: Vec<(f32, f32)>) -> Vec<f32> {
        let elements: Vec<_> = curve.iter().map(|(l, _)| *l).collect();
        let knots: Vec<_> = curve.iter().map(|(_, a)| *a).collect();

        info!("Building curve elements {:?} knots {:?}", elements, knots);

        Linear::builder()
            .elements(elements)
            .knots(knots)
            .build()
            .unwrap()
            .take(180)
            .collect()
    }

    pub fn lift_coefficient_samples(&self) -> Vec<f32> {
        Self::build_samples(self.lift_coefficient_curve.clone())
    }

    pub fn drag_coefficient_samples(&self) -> Vec<f32> {
        Self::build_samples(self.drag_coefficient_curve.clone())
    }
}

#[derive(Debug, Clone)]
pub struct FuselageSpec {
    pub size: Vec3,
    pub mass: f32,
}

#[derive(Debug, Clone, Default)]
pub struct TailSpec {
    pub size: Vec3,
    pub vertical: WingSpec,
    pub horizontal: WingSpec,
}
