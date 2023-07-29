mod build;
pub mod spec;

use core::f32;
use std::ops::AddAssign;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::physics::CentreOfGravity;

use self::spec::PlaneSpec;

pub struct PlanePlugin;

impl Plugin for PlanePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BuildPlaneEvent>()
            .add_systems(Startup, (setup_plane, apply_deferred).chain())
            .add_systems(
                Update,
                (
                    (build_plane, build::build_plane).chain(),
                    update_propellor,
                    update_airfoil_control_surfaces,
                    update_airspeed,
                    update_thrust_forces,
                    update_airfoil_forces,
                )
                    .chain(),
            );
    }
}

#[derive(Event)]
pub struct BuildPlaneEvent;

#[derive(Component)]
pub struct Plane;

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

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum AirfoilPosition {
    Wing(Side),
    TailWing(Side),
    VerticalTail,
}

#[derive(Debug, Clone, Copy)]
pub enum AirfoilOrientation {
    Horizontal,
    Vertical,
}

#[derive(Component)]
pub struct Airfoil {
    pub orientation: AirfoilOrientation,
    pub area: f32,
    pub lift_coefficient_samples: Vec<f32>,
}

impl Airfoil {
    pub fn force_base_dir(&self, global_tx: &GlobalTransform) -> Vec3 {
        match self.orientation {
            AirfoilOrientation::Horizontal => global_tx.up(),
            AirfoilOrientation::Vertical => global_tx.right(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn offset(&self) -> f32 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }
}

#[derive(Component)]
pub struct Propellor;

fn setup_plane(mut build_plane_event: EventWriter<BuildPlaneEvent>) {
    build_plane_event.send(BuildPlaneEvent);
}

fn build_plane(
    mut commands: Commands,
    plane_query: Query<Entity, With<Plane>>,
    mut build_plane_event: EventReader<BuildPlaneEvent>,
) {
    for _ in build_plane_event.iter() {
        if let Ok(entity) = plane_query.get_single() {
            info!("Removing existing plane");
            commands.entity(entity).despawn_recursive();
        }

        info!("Building plane");
        commands.spawn(PlaneSpec::new("Test plane"));
    }
}

fn angle_of_attack(velocity: Vec3, up: Vec3, forward: Vec3) -> f32 {
    let a1 = up.angle_between(forward);
    let a2 = up.angle_between(velocity.normalize());

    a2 - a1
}

fn update_airfoil_control_surfaces(
    control_query: Query<&PlaneControl>,
    wing_query: Query<(&AirfoilPosition, &Parent, &Children)>,
    mut aileron_query: Query<&mut Transform, With<Airfoil>>,
) {
    for (position, entity, children) in wing_query.iter() {
        if let Ok(control) = control_query.get(**entity) {
            for child in children.iter() {
                if let Ok(mut aileron_tx) = aileron_query.get_mut(*child) {
                    match position {
                        AirfoilPosition::Wing(Side::Left) => {
                            aileron_tx.rotation = Quat::from_rotation_x(-control.ailerons);
                        }
                        AirfoilPosition::Wing(Side::Right) => {
                            aileron_tx.rotation = Quat::from_rotation_x(control.ailerons);
                        }
                        AirfoilPosition::TailWing(_) => {
                            aileron_tx.rotation = Quat::from_rotation_x(control.elevators);
                        }
                        AirfoilPosition::VerticalTail => {
                            aileron_tx.rotation = Quat::from_rotation_y(control.rudder);
                        }
                    }
                }
            }
        }
    }
}

fn update_propellor(
    plane_query: Query<(&Thrust, &PlaneSpec)>,
    mut propellor_query: Query<&mut Transform, With<Propellor>>,
    time: Res<Time>,
) {
    let Ok((Thrust(thrust), spec)) = plane_query.get_single() else {
        return;
    };

    for mut tx in propellor_query.iter_mut() {
        let rate = (*thrust / spec.thrust) * 3600_f32.to_radians();
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
            &PlaneSpec,
            &Thrust,
            &GlobalTransform,
            &CentreOfGravity,
            &mut ExternalForce,
        ),
        With<Plane>,
    >,
) {
    for (spec, Thrust(thrust), global_tx, centre_of_gravity, mut external_force) in
        plane_query.iter_mut()
    {
        external_force.force = Vec3::ZERO;
        external_force.torque = Vec3::ZERO;
        external_force.add_assign(ExternalForce::at_point(
            global_tx.forward() * *thrust,
            global_tx.translation() + (global_tx.forward() * spec.fuselage.size.z * 0.5),
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
