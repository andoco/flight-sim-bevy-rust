use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use enterpolation::{linear::Linear, Curve};

use super::{
    Aileron, Airfoil, AirfoilPosition, AngleOfAttack, HorizontalTailWing, Lift, PlaneLimits,
    Propellor, Side, VerticalTailWing, Wing,
};

pub fn build_fuselage(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    fuselage_color: Color,
) {
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                limits.fuselage.x,
                limits.fuselage.y,
                limits.fuselage.z,
            ))),
            material: materials.add(fuselage_color.into()),
            ..default()
        },
        Friction::new(0.01),
        Collider::cuboid(
            limits.fuselage.x * 0.5,
            limits.fuselage.y * 0.5,
            limits.fuselage.z * 0.5,
        ),
        ColliderMassProperties::Density(2.0),
    ));
}

pub fn build_wings(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    lift_coefficient_samples: &Vec<f32>,
    wing_color: Color,
) {
    [Side::Left, Side::Right].iter().for_each(|side| {
        build_wing(
            parent,
            meshes,
            materials,
            limits,
            lift_coefficient_samples,
            wing_color,
            *side,
        );
    });
}

fn build_wing(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    lift_coefficient_samples: &Vec<f32>,
    wing_color: Color,
    side: Side,
) {
    let (position, offset) = match side {
        Side::Left => (AirfoilPosition::WingLeft, 1.0),
        Side::Right => (AirfoilPosition::WingRight, -1.0),
    };

    let width = limits.wings.x * 0.5;
    let length = limits.wings.y;

    parent
        .spawn((
            Wing(side),
            Airfoil {
                position,
                area: width * length,
                lift_coefficient_samples: lift_coefficient_samples.clone(),
            },
            AngleOfAttack::default(),
            Lift::default(),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(width, 0.2, length))),
                material: materials.add(wing_color.into()),
                transform: Transform::from_xyz(width * 0.5 * offset, 0.0, limits.wing_offset_z),
                ..default()
            },
            Collider::cuboid(width * 0.5, 0.1, length * 0.5),
        ))
        .with_children(|parent| {
            let aileron_width = width * 0.25;
            let aileron_height = 0.1;
            let aileron_length = length * 0.1;

            parent.spawn((
                Aileron(side),
                Airfoil {
                    position: AirfoilPosition::Aileron(side),
                    area: aileron_width * aileron_length,
                    lift_coefficient_samples: lift_coefficient_samples.clone(),
                },
                AngleOfAttack::default(),
                Lift::default(),
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        aileron_width,
                        aileron_height,
                        aileron_length,
                    ))),
                    material: materials.add(wing_color.into()),
                    transform: Transform::from_xyz(
                        (length - aileron_width * 0.5) * offset,
                        0.0,
                        limits.wing_offset_z + length / 2.0 + aileron_length / 2.0,
                    ),
                    ..default()
                },
                Collider::cuboid(
                    aileron_width * 0.5,
                    aileron_height * 0.5,
                    aileron_length * 0.5,
                ),
            ));
        });
}

pub fn build_propellor(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    propellor_color: Color,
) {
    let size = Vec3::new(limits.fuselage.x * 2.5, 0.4, 0.1);

    parent.spawn((
        Propellor,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z))),
            material: materials.add(propellor_color.into()),
            transform: Transform::from_xyz(0.0, 0.0, -limits.fuselage.z * 0.5),
            ..default()
        },
        Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
    ));
}

pub fn build_tail(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    color: Color,
) {
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                limits.tail.x,
                limits.tail.y,
                limits.tail.z,
            ))),
            transform: Transform::from_xyz(0.0, 0.0, limits.fuselage.z / 2.0 + limits.tail.z / 2.0),
            material: materials.add(color.into()),
            ..default()
        },
        Friction::new(0.01),
        Collider::cuboid(
            limits.tail.x * 0.5,
            limits.tail.y * 0.5,
            limits.tail.z * 0.5,
        ),
    ));
}

pub fn build_vertical_tail(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    color: Color,
    size: Vec3,
) {
    let lift_coefficient_curve = Linear::builder()
        .elements([-0.0, -0.01, 0.0, 0.0, 0.0, 0.01, 0.0])
        .knots([-90.0, -10.0, -2.5, 0.0, 2.5, 10.0, 90.0])
        .build()
        .unwrap();

    parent.spawn((
        VerticalTailWing,
        Airfoil {
            position: AirfoilPosition::VerticalTail,
            area: size.y * size.z,
            lift_coefficient_samples: lift_coefficient_curve.take(180).collect(),
        },
        AngleOfAttack::default(),
        Lift::default(),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z))),
            material: materials.add(color.into()),
            transform: Transform::from_xyz(
                0.,
                limits.tail.y * 0.5 + size.y * 0.5,
                limits.fuselage.z * 0.5 + limits.tail.z - size.z * 0.5,
            ),
            ..default()
        },
        Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
    ));
}

pub fn build_horizontal_tails(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    limits: &PlaneLimits,
    color: Color,
) {
    let tail_wing_lift_coefficient_curve = Linear::builder()
        .elements([-0.0, -0.25, 0.0, 0.0, 0.0, 0.25, 0.0])
        .knots([-90.0, -10.0, -2.5, 0.0, 2.5, 10.0, 90.0])
        .build()
        .unwrap();

    for (position, side) in [
        (AirfoilPosition::HorizontalTailLeft, Side::Left),
        (AirfoilPosition::HorizontalTailRight, Side::Right),
    ] {
        let width = 2.0;
        let height = 0.1;
        let length = 0.7;

        let offset = match position {
            AirfoilPosition::HorizontalTailLeft => -1.0,
            AirfoilPosition::HorizontalTailRight => 1.0,
            _ => panic!("Not a horizontal tail position"),
        };

        parent.spawn((
            HorizontalTailWing(side),
            Airfoil {
                position,
                area: width * length,
                lift_coefficient_samples: tail_wing_lift_coefficient_curve.take(180).collect(),
            },
            AngleOfAttack::default(),
            Lift::default(),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(width, height, length))),
                material: materials.add(color.into()),
                transform: Transform::from_xyz(
                    (limits.tail.x * 0.5 + width * 0.5) * offset,
                    0.0,
                    limits.fuselage.z * 0.5 + limits.tail.z - length * 0.5,
                ),
                ..default()
            },
            Collider::cuboid(width * 0.5, height * 0.5, length * 0.5),
        ));
    }
}
