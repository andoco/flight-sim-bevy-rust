use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::{CameraPlugin, Follow},
    plane::PlanePlugin,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_plugin(CameraPlugin)
            .add_plugin(PlanePlugin)
            .add_startup_system(setup_lighting)
            .add_startup_system(setup_ground)
            .add_system(update_block_positions)
            .add_system(generate_infinite_buildings);
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-90_f32.to_radians())),
        ..default()
    });
}

fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((Collider::cuboid(10000.0, 0.1, 10000.0), Friction::new(0.01)))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 20000.,
                ..default()
            })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
}

const SPACING: i32 = 20;

#[derive(Component)]
pub struct BlockPos(pub i32, pub i32);

fn update_block_positions(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform, &BlockPos)>,
) {
    for (entity, global_tx, BlockPos(x, z)) in query.iter() {
        let position = global_tx.translation();
        let px = position.x as i32 / SPACING;
        let pz = position.z as i32 / SPACING;

        // info!("px={}, pz={}", px, pz);

        if px != *x || pz != *z {
            commands.entity(entity).insert(BlockPos(px, pz));
        }
    }
}

fn generate_infinite_buildings(
    mut commands: Commands,
    query: Query<&BlockPos, (Changed<BlockPos>, With<Follow>)>,
    mut block_positions: Local<HashSet<(i32, i32)>>,
    mut block_entities: Local<HashMap<(i32, i32), Entity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(BlockPos(px, pz)) = query.get_single() else {
        return;
    };

    let active_block_distance = 5;

    let px = *px;
    let pz = *pz;

    let mut active_block_positions = HashSet::new();

    let mut num_misses = 0;
    let mut num_hits = 0;

    for z in (pz - active_block_distance)..(pz + active_block_distance) {
        for x in (px - active_block_distance)..(px + active_block_distance) {
            let block_pos = (x, z);

            active_block_positions.insert(block_pos);

            if block_positions.contains(&block_pos) {
                num_hits += 1;
            } else {
                num_misses += 1;

                let building_pos = Vec3::new((x * SPACING) as f32, 0.5, (z * SPACING) as f32);

                let building_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_translation(building_pos),
                        ..default()
                    })
                    .insert(RigidBody::Fixed)
                    .insert(Collider::cuboid(0.5, 0.5, 0.5))
                    .id();

                block_entities.insert(block_pos, building_entity);
            }
        }
    }

    let new_positions: Vec<_> = active_block_positions
        .difference(&block_positions)
        .copied()
        .collect();

    let old_positions: Vec<_> = block_positions
        .difference(&active_block_positions)
        .copied()
        .collect();

    info!(
        "hits={}, misses={}, block_positions={}, old_positions={}, new_positions={}",
        num_hits,
        num_misses,
        block_positions.len(),
        old_positions.len(),
        new_positions.len()
    );

    info!("Pruning {} old positions", old_positions.len());
    for pos in old_positions {
        if let Some(entity) = block_entities.get(&pos) {
            commands.entity(*entity).despawn_recursive();
            block_entities.remove(&pos);
        }
        block_positions.remove(&pos);
    }

    info!("Adding {} new positions", new_positions.len());
    block_positions.extend(new_positions.into_iter());
}
