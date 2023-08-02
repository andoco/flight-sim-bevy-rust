use bevy::{math::vec3, prelude::*};

use crate::plane::spec::{FuselageSpec, PlaneSpec, TailSpec, WingSpec};

use super::Vec3Model;

#[derive(Component, Default)]
pub struct PlaneSpecModel {
    pub thrust: String,
    pub fuselage: BodyModel,
    pub wings: WingModel,
    pub tail: Vec3Model,
    pub tail_horizontal: WingModel,
    pub tail_vertical: WingModel,
}

#[derive(Default)]
pub struct BodyModel {
    pub size: Vec3Model,
    pub mass: String,
}

impl BodyModel {
    pub fn new(size: Vec3, mass: f32) -> Self {
        Self {
            size: Vec3Model::new(size),
            mass: mass.to_string(),
        }
    }
}

#[derive(Default)]
pub struct WingModel {
    pub size: Vec3Model,
    pub lift_coefficient_curve: Vec<(String, String)>,
    pub control_lift_coefficient_curve: Vec<(String, String)>,
    pub drag_coefficient_curve: Vec<(String, String)>,
}

impl WingModel {
    fn new(value: &WingSpec) -> Self {
        Self {
            size: Vec3Model::new(value.size),
            lift_coefficient_curve: value
                .lift_coefficient_curve
                .iter()
                .map(|(l, a)| (l.to_string(), a.to_string()))
                .collect(),
            control_lift_coefficient_curve: value
                .control_lift_coefficient_curve
                .iter()
                .map(|(l, a)| (l.to_string(), a.to_string()))
                .collect(),
            drag_coefficient_curve: value
                .drag_coefficient_curve
                .iter()
                .map(|(l, a)| (l.to_string(), a.to_string()))
                .collect(),
        }
    }

    fn to_spec(&self) -> WingSpec {
        WingSpec {
            size: vec3(
                self.size.x.parse().unwrap_or_default(),
                self.size.y.parse().unwrap_or_default(),
                self.size.z.parse().unwrap_or_default(),
            ),
            lift_coefficient_curve: self
                .lift_coefficient_curve
                .iter()
                .map(|(l, a)| (l.parse().unwrap_or_default(), a.parse().unwrap_or_default()))
                .collect(),
            control_lift_coefficient_curve: self
                .control_lift_coefficient_curve
                .iter()
                .map(|(l, a)| (l.parse().unwrap_or_default(), a.parse().unwrap_or_default()))
                .collect(),
            drag_coefficient_curve: self
                .drag_coefficient_curve
                .iter()
                .map(|(l, a)| (l.parse().unwrap_or_default(), a.parse().unwrap_or_default()))
                .collect(),
            ..default()
        }
    }
}

impl PlaneSpecModel {
    pub fn new(spec: &PlaneSpec) -> Self {
        Self {
            thrust: spec.thrust.to_string(),
            fuselage: BodyModel::new(spec.fuselage.size, spec.fuselage.mass),
            wings: WingModel::new(&spec.wings),
            tail: Vec3Model::new(spec.tail.size),
            tail_horizontal: WingModel::new(&spec.tail.horizontal),
            tail_vertical: WingModel::new(&spec.tail.vertical),
        }
    }
}

impl PlaneSpecModel {
    pub fn to_spec(&self) -> PlaneSpec {
        PlaneSpec {
            thrust: self.thrust.parse().unwrap_or_default(),
            fuselage: FuselageSpec {
                size: vec3(
                    self.fuselage.size.x.parse().unwrap_or_default(),
                    self.fuselage.size.y.parse().unwrap_or_default(),
                    self.fuselage.size.z.parse().unwrap_or_default(),
                ),
                mass: self.fuselage.mass.parse().unwrap_or_default(),
            },
            wings: self.wings.to_spec(),
            tail: TailSpec {
                size: vec3(
                    self.tail.x.parse().unwrap_or_default(),
                    self.tail.y.parse().unwrap_or_default(),
                    self.tail.z.parse().unwrap_or_default(),
                ),
                horizontal: self.tail_horizontal.to_spec(),
                vertical: self.tail_vertical.to_spec(),
            },
            ..default()
        }
    }
}
