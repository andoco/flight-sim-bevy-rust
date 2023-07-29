use bevy::{math::vec3, prelude::*};

use crate::plane::spec::{FuselageSpec, PlaneSpec, TailSpec, WingSpec};

use super::Vec3Model;

#[derive(Component, Default)]
pub struct PlaneSpecModel {
    pub fuselage: Vec3Model,
    pub wings: WingModel,
    pub tail: Vec3Model,
    pub tail_horizontal: WingModel,
    pub tail_vertical: WingModel,
}

fn vec_to_string(values: &Vec<f32>) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn string_to_vec(value: &str) -> Vec<f32> {
    value
        .split(",")
        .map(|s| s.parse().unwrap_or_default())
        .collect()
}

#[derive(Default)]
pub struct WingModel {
    pub size: Vec3Model,
    pub elements: String,
    pub knots: String,
}

impl WingModel {
    fn new(value: &WingSpec) -> Self {
        Self {
            size: Vec3Model::new(value.size),
            elements: vec_to_string(&value.lift_coefficient_elements),
            knots: vec_to_string(&value.lift_coefficient_knots),
        }
    }

    fn to_spec(&self) -> WingSpec {
        WingSpec {
            size: vec3(
                self.size.x.parse().unwrap_or_default(),
                self.size.y.parse().unwrap_or_default(),
                self.size.z.parse().unwrap_or_default(),
            ),
            lift_coefficient_elements: string_to_vec(self.elements.as_str()),
            lift_coefficient_knots: string_to_vec(self.knots.as_str()),
        }
    }
}

impl PlaneSpecModel {
    pub fn new(spec: &PlaneSpec) -> Self {
        Self {
            fuselage: Vec3Model::new(spec.fuselage.size),
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
            fuselage: FuselageSpec {
                size: vec3(
                    self.fuselage.x.parse().unwrap_or_default(),
                    self.fuselage.y.parse().unwrap_or_default(),
                    self.fuselage.z.parse().unwrap_or_default(),
                ),
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
