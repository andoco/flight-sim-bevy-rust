use bevy::{math::vec3, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    camera,
    physics::CentreOfGravity,
    world::{self, BlockPos},
};

use super::{
    spec::{FuselageSpec, PlaneSpec, TailSpec, WingSpec},
    Airfoil, AirfoilOrientation, AirfoilPosition, Airspeed, Altitude, AngleOfAttack, Lift, Plane,
    PlaneControl, PlaneFlight, Propellor, Side, Thrust,
};

pub fn build_plane(
    mut commands: Commands,
    plane_query: Query<(Entity, &PlaneSpec), Added<PlaneSpec>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, plane) in plane_query.iter() {
        let metal_color = Color::hex("d5d5d7").unwrap();
        let fuselage_color = Color::rgb(1.0, 0.0, 0.0);
        let propellor_color = metal_color;
        let wing_color = metal_color;

        commands
            .entity(entity)
            .insert((
                Plane,
                PlaneControl::default(),
                PlaneFlight::default(),
                CentreOfGravity::default(),
                Thrust(0.0),
                Airspeed::default(),
                Altitude::default(),
                SpatialBundle::from_transform(Transform::from_xyz(
                    world::SPACING as f32 * 0.5,
                    plane.fuselage.size.y * 0.5 + 0.6,
                    0.,
                )),
                RigidBody::Dynamic,
                Velocity::zero(),
                ExternalForce::default(),
                ReadMassProperties::default(),
                camera::Follow(camera::FollowKind::Behind),
                BlockPos(0, 0),
            ))
            .with_children(|parent| {
                build_fuselage(
                    parent,
                    &mut meshes,
                    &mut materials,
                    &plane.fuselage,
                    fuselage_color,
                );

                build_wheels(parent, &mut meshes, &mut materials, &plane.fuselage);

                build_propellor(
                    parent,
                    &mut meshes,
                    &mut materials,
                    &plane.fuselage,
                    propellor_color,
                );

                build_wings(
                    parent,
                    &mut meshes,
                    &mut materials,
                    vec3(0., 0., 1.0),
                    &plane.wings,
                    wing_color,
                );

                build_tail(
                    parent,
                    &mut meshes,
                    &mut materials,
                    vec3(0., 0., plane.fuselage.size.z / 2.0),
                    &plane.tail,
                    fuselage_color,
                );
            });
    }
}

pub fn build_fuselage(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spec: &FuselageSpec,
    fuselage_color: Color,
) {
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                spec.size.x,
                spec.size.y,
                spec.size.z,
            ))),
            material: materials.add(fuselage_color.into()),
            ..default()
        },
        Friction::new(0.0),
        Collider::cuboid(spec.size.x * 0.5, spec.size.y * 0.5, spec.size.z * 0.5),
        ColliderMassProperties::Mass(spec.mass),
    ));
}

pub fn build_wheels(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spec: &FuselageSpec,
) {
    let wheel_y = -spec.size.y * 0.5 + 0.5;

    for side in [Side::Left, Side::Right] {
        parent.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: 0.2,
                    height: 0.1,
                    ..default()
                })),
                transform: Transform::from_xyz(
                    spec.size.x * 0.5 * side.offset(),
                    wheel_y,
                    -spec.size.z * 0.5,
                )
                .with_rotation(Quat::from_rotation_z(90_f32.to_radians())),
                material: materials.add(Color::BLACK.into()),
                ..default()
            },
            Friction::new(0.0),
            Collider::ball(0.2),
        ));
    }

    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 0.2,
                height: 0.1,
                ..default()
            })),
            transform: Transform::from_xyz(0.0, wheel_y, 5.)
                .with_rotation(Quat::from_rotation_z(90_f32.to_radians())),
            material: materials.add(Color::BLACK.into()),
            ..default()
        },
        Friction::new(0.0),
        Collider::ball(0.2),
    ));
}

pub fn build_wings(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    spec: &WingSpec,
    wing_color: Color,
) {
    [Side::Left, Side::Right].iter().for_each(|side| {
        build_wing(
            parent,
            meshes,
            materials,
            pos,
            spec,
            wing_color,
            Some(*side),
            AirfoilPosition::Wing(*side),
            AirfoilOrientation::Horizontal,
        );
    });
}

fn build_wing(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    spec: &WingSpec,
    wing_color: Color,
    side: Option<Side>,
    position: AirfoilPosition,
    orientation: AirfoilOrientation,
) {
    let offset = match side {
        Some(side) => side.offset(),
        None => 0.0,
    };

    parent
        .spawn((
            position,
            Airfoil {
                orientation,
                area: spec.size.x * spec.size.z,
                lift_coefficient_samples: spec.lift_coefficient_samples(),
                drag_coefficient_samples: spec.drag_coefficient_samples(),
            },
            AngleOfAttack::default(),
            Lift::default(),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    spec.size.x,
                    spec.size.y,
                    spec.size.z,
                ))),
                material: materials.add(wing_color.into()),
                transform: Transform::from_translation(
                    pos + vec3(spec.size.x * 0.5 * offset, 0.0, 0.0),
                ),
                ..default()
            },
            Collider::cuboid(spec.size.x * 0.5, spec.size.y * 0.5, spec.size.z * 0.5),
        ))
        .with_children(|parent| {
            let control_width = spec.size.x;
            let control_height = spec.size.y;
            let control_length = spec.size.z * 0.25;

            parent.spawn((
                Airfoil {
                    orientation,
                    area: control_width * control_length,
                    lift_coefficient_samples: spec.control_lift_coefficient_samples(),
                    drag_coefficient_samples: spec.drag_coefficient_samples(),
                },
                AngleOfAttack::default(),
                Lift::default(),
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        control_width,
                        control_height,
                        control_length,
                    ))),
                    material: materials.add(wing_color.into()),
                    transform: Transform::from_xyz(
                        (spec.size.x * 0.5 - control_width * 0.5) * offset,
                        0.0,
                        spec.size.z / 2.0 + control_length / 2.0,
                    ),
                    ..default()
                },
                Collider::cuboid(
                    control_width * 0.5,
                    control_height * 0.5,
                    control_length * 0.5,
                ),
            ));
        });
}

pub fn build_propellor(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spec: &FuselageSpec,
    propellor_color: Color,
) {
    let size = Vec3::new(spec.size.x * 2.5, 0.4, 0.1);

    parent.spawn((
        Propellor,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z))),
            material: materials.add(propellor_color.into()),
            transform: Transform::from_xyz(0.0, 0.0, -spec.size.z * 0.5),
            ..default()
        },
        // Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
    ));
}

pub fn build_tail(
    parent: &mut ChildBuilder<'_, '_, '_>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    spec: &TailSpec,
    color: Color,
) {
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                spec.size.x,
                spec.size.y,
                spec.size.z,
            ))),
            transform: Transform::from_xyz(0.0, 0.0, pos.z + spec.size.z / 2.0),
            material: materials.add(color.into()),
            ..default()
        },
        Friction::new(0.01),
        Collider::cuboid(spec.size.x * 0.5, spec.size.y * 0.5, spec.size.z * 0.5),
        ColliderMassProperties::Mass(10.),
    ));

    let end_pos = pos + vec3(0., 0., spec.size.z);

    // Vertical tail surfaces
    build_wing(
        parent,
        meshes,
        materials,
        end_pos + Vec3::Y * spec.vertical.size.y * 0.5,
        &spec.vertical,
        Color::BLUE,
        None,
        AirfoilPosition::VerticalTail,
        AirfoilOrientation::Vertical,
    );

    // Horizontal tail surfaces
    for side in [Side::Left, Side::Right] {
        build_wing(
            parent,
            meshes,
            materials,
            end_pos,
            &spec.horizontal,
            Color::BLUE,
            Some(side),
            AirfoilPosition::TailWing(side),
            AirfoilOrientation::Horizontal,
        );
    }
}
