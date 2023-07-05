use bevy::prelude::*;
use bevy_rapier3d::prelude::ReadMassProperties;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_centre_of_gravity);
    }
}

#[derive(Component, Default)]
pub struct CentreOfGravity {
    pub local: Vec3,
    pub global: Vec3,
}

fn update_centre_of_gravity(
    mut query: Query<(&GlobalTransform, &ReadMassProperties, &mut CentreOfGravity)>,
) {
    for (global_tx, ReadMassProperties(mass_properties), mut centre_of_gravity) in query.iter_mut()
    {
        let global_centre_of_mass = global_tx.mul_transform(Transform::from_translation(
            mass_properties.local_center_of_mass,
        ));

        centre_of_gravity.local = mass_properties.local_center_of_mass;
        centre_of_gravity.global = global_centre_of_mass.translation();
    }
}
