mod build;

use core::f32;
use std::ops::AddAssign;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use enterpolation::{linear::Linear, Curve};

use crate::{
    camera::{self},
    physics::CentreOfGravity,
    world::{self, BlockPos},
};

use self::build::{
    build_fuselage, build_horizontal_tails, build_propellor, build_vertical_tail, build_wings,
};

pub struct PlanePlugin;

impl Plugin for PlanePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_plane).add_systems(
            Update,
            (
                update_propellor,
                update_airfoil_rotations,
                update_ailerons,
                update_airspeed,
                update_thrust_forces,
                update_airfoil_forces,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct Plane;

#[derive(Component, Clone)]
pub struct PlaneLimits {
    pub thrust: f32,
    pub fuselage: Vec3,
    pub wings: Vec2,
    pub wing_offset_z: f32,
    pub lift_coefficient_samples: Vec<f32>,
}

#[derive(Component, Default)]
pub struct PlaneControl {
    pub ailerons: f32,
    pub elevators: f32,
    pub rudder: f32,
}

impl PlaneControl {
    pub fn clear(&mut self) {
        *self = Default::default();
    }
}

#[derive(Component, Default)]
pub struct Thrust(pub f32);

#[derive(Component, Default)]
pub struct Airspeed(pub f32);

#[derive(Component, Default)]
pub struct Altitude(pub f32);

#[derive(Component, Default)]
pub struct Lift(pub f32);

#[derive(Component, Default)]
pub struct AngleOfAttack(pub f32);

#[derive(Component, Default)]
pub struct PlaneFlight {
    pub angle_of_attack: f32,
    pub weight: f32,
    pub drag: f32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AirfoilPosition {
    WingLeft,
    WingRight,
    Aileron(Side),
    HorizontalTailLeft,
    HorizontalTailRight,
    VerticalTail,
}

#[derive(Component)]
pub struct Airfoil {
    pub position: AirfoilPosition,
    pub area: f32,
    pub lift_coefficient_samples: Vec<f32>,
}

impl Airfoil {
    pub fn up(&self) -> Vec3 {
        match self.position {
            AirfoilPosition::VerticalTail => Vec3::X,
            _ => Vec3::Y,
        }
    }

    pub fn force_base_dir(&self, global_tx: &GlobalTransform) -> Vec3 {
        match self.position {
            AirfoilPosition::VerticalTail => global_tx.right(),
            _ => global_tx.up(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

#[derive(Component)]
pub struct Wing(Side);

#[derive(Component)]
pub struct VerticalTailWing;

#[derive(Component)]
pub struct HorizontalTailWing(Side);

#[derive(Component)]
pub struct Propellor;

#[derive(Component)]
pub struct Aileron(Side);

fn setup_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lift_coefficient_curve = Linear::builder()
        .elements([0.0, 0.0, 0.35, 1.4, 0.8, 0.0])
        .knots([-90.0, -5.0, 0.0, 10.0, 15.0, 90.0])
        .build()
        .unwrap();

    let lift_coefficient_samples: Vec<_> = lift_coefficient_curve.take(180).collect();

    let limits = PlaneLimits {
        thrust: 150.0,
        fuselage: Vec3::new(1.12, 2.0, 8.3),
        wings: Vec2::new(11.0, 1.5),
        wing_offset_z: -0.0,
        lift_coefficient_samples: lift_coefficient_samples.clone(),
    };

    let metal_color = Color::hex("d5d5d7").unwrap();
    let fuselage_color = Color::rgb(1.0, 0.0, 0.0);
    let propellor_color = metal_color;
    let wing_color = metal_color;

    commands
        .spawn((
            Plane,
            PlaneControl::default(),
            limits.clone(),
            PlaneFlight::default(),
            CentreOfGravity::default(),
            Thrust(0.0),
            Airspeed::default(),
            Altitude::default(),
            SpatialBundle::from_transform(Transform::from_xyz(
                world::SPACING as f32 * 0.5,
                1.1,
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
            build_fuselage(parent, &mut meshes, &mut materials, &limits, fuselage_color);

            build_propellor(
                parent,
                &mut meshes,
                &mut materials,
                &limits,
                propellor_color,
            );

            build_wings(
                parent,
                &mut meshes,
                &mut materials,
                &limits,
                &lift_coefficient_samples,
                wing_color,
            );

            let tail_size = Vec3::new(0.2, limits.fuselage.y, limits.fuselage.y);

            build_vertical_tail(
                parent,
                &mut meshes,
                &mut materials,
                &limits,
                wing_color,
                tail_size,
            );

            build_horizontal_tails(parent, &mut meshes, &mut materials, &limits, wing_color);
        });
}

fn angle_of_attack(velocity: Vec3, up: Vec3, forward: Vec3) -> f32 {
    let a1 = up.angle_between(forward);
    let a2 = up.angle_between(velocity.normalize());

    a2 - a1
}

fn update_airfoil_rotations(
    query: Query<(&PlaneControl, &Children)>,
    mut airfoil_query: Query<(&Airfoil, &mut Transform)>,
) {
    for (control, children) in query.iter() {
        for child in children.iter() {
            if let Ok((airfoil, mut airfoil_tx)) = airfoil_query.get_mut(*child) {
                match airfoil.position {
                    AirfoilPosition::HorizontalTailLeft | AirfoilPosition::HorizontalTailRight => {
                        airfoil_tx.rotation = Quat::from_rotation_x(control.elevators);
                    }
                    AirfoilPosition::VerticalTail => {
                        airfoil_tx.rotation = Quat::from_rotation_y(control.rudder);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn update_ailerons(
    control_query: Query<&PlaneControl>,
    wing_query: Query<(&Parent, &Children), With<Wing>>,
    mut aileron_query: Query<(&mut Transform, &Aileron)>,
) {
    for (entity, children) in wing_query.iter() {
        if let Ok(control) = control_query.get(**entity) {
            for child in children.iter() {
                if let Ok((mut aileron_tx, Aileron(side))) = aileron_query.get_mut(*child) {
                    match side {
                        Side::Left => {
                            aileron_tx.rotation = Quat::from_rotation_x(-control.ailerons);
                        }
                        Side::Right => {
                            aileron_tx.rotation = Quat::from_rotation_x(control.ailerons);
                        }
                    }
                }
            }
        }
    }
}

fn update_propellor(
    plane_query: Query<(&Thrust, &PlaneLimits)>,
    mut propellor_query: Query<&mut Transform, With<Propellor>>,
    time: Res<Time>,
) {
    let Ok((Thrust(thrust), limits)) = plane_query.get_single() else {
        return;
    };

    for mut tx in propellor_query.iter_mut() {
        let rate = (*thrust / limits.thrust) * 3600_f32.to_radians();
        tx.rotate_local_z(rate * time.delta_seconds());
    }
}

fn update_airspeed(mut plane_query: Query<(&GlobalTransform, &Velocity, &mut Airspeed)>) {
    for (global_tx, velocity, mut airspeed) in plane_query.iter_mut() {
        let local_velocity = (global_tx.translation() + velocity.linvel) - global_tx.translation();
        airspeed.0 = -local_velocity.z;
    }
}

fn update_thrust_forces(
    mut plane_query: Query<
        (
            &PlaneLimits,
            &Thrust,
            &GlobalTransform,
            &CentreOfGravity,
            &mut ExternalForce,
        ),
        With<Plane>,
    >,
) {
    for (limits, Thrust(thrust), global_tx, centre_of_gravity, mut external_force) in
        plane_query.iter_mut()
    {
        external_force.force = Vec3::ZERO;
        external_force.torque = Vec3::ZERO;
        external_force.add_assign(ExternalForce::at_point(
            global_tx.forward() * *thrust,
            global_tx.translation() + (global_tx.forward() * limits.fuselage.z * 0.5),
            centre_of_gravity.global,
        ));
    }
}

fn update_airfoil_forces(
    mut plane_query: Query<
        (
            &mut PlaneFlight,
            &Airspeed,
            &Velocity,
            &CentreOfGravity,
            &mut ExternalForce,
        ),
        With<Plane>,
    >,
    mut airfoil_query: Query<(&Airfoil, &GlobalTransform, &mut AngleOfAttack, &mut Lift)>,
    mut gizmos: Gizmos,
) {
    for (mut flight, Airspeed(airspeed), velocity, centre_of_gravity, mut external_force) in
        plane_query.iter_mut()
    {
        let air_density = 1.225; // 1.225 kg/m^3 at sea level
        let dynamic_pressure = 0.5 * air_density * airspeed * airspeed;

        for (airfoil, airfoil_global_tx, mut aoa, mut airfoil_lift) in airfoil_query.iter_mut() {
            let angle_of_attack = angle_of_attack(
                velocity.linvel,
                airfoil.force_base_dir(airfoil_global_tx),
                airfoil_global_tx.forward(),
            );

            aoa.0 = angle_of_attack;

            let lift_coefficient_index = (angle_of_attack.to_degrees() + 90.0) as usize;

            let lift_coefficient = airfoil
                .lift_coefficient_samples
                .get(lift_coefficient_index)
                .unwrap_or(&0.0);

            let lift = lift_coefficient * dynamic_pressure * airfoil.area;
            airfoil_lift.0 = lift;

            external_force.add_assign(ExternalForce::at_point(
                airfoil.force_base_dir(airfoil_global_tx) * lift,
                airfoil_global_tx.translation(),
                centre_of_gravity.global,
            ));

            let drag_coefficient = 0.032; // For Cessna 172 at sea level and 100 knots at 0 degrees angle of attack
            let drag = drag_coefficient * dynamic_pressure * airfoil.area;
            external_force.force += -velocity.linvel.normalize_or_zero() * drag;

            flight.drag = drag;

            gizmos.line(
                airfoil_global_tx.translation(),
                airfoil_global_tx.translation() + airfoil.force_base_dir(airfoil_global_tx) * lift,
                Color::YELLOW,
            );
        }
    }
}
