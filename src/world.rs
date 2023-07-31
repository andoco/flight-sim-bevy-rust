use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_rapier3d::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::{
    camera::{CameraPlugin, Follow},
    input::InputPlugin,
    physics::PhysicsPlugin,
    plane::PlanePlugin,
};

pub struct WorldPlugin;

#[derive(Resource)]
struct Rand {
    perlin: Perlin,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(CameraPlugin)
            .add_plugins(PhysicsPlugin)
            .add_plugins(PlanePlugin)
            .add_plugins(InputPlugin)
            .insert_resource(Rand {
                perlin: Perlin::new(1),
            })
            .add_systems(Startup, (setup_lighting, setup_ground))
            .add_systems(
                Update,
                (
                    update_sun,
                    update_block_positions,
                    generate_infinite_buildings,
                ),
            );
    }
}

fn setup_lighting(mut commands: Commands) {
    // Configure a properly scaled cascade shadow map for this scene (defaults are too large, mesh units are in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(0.98, 0.95, 0.82),
                shadows_enabled: true,
                ..default()
            },
            cascade_shadow_config,
            ..default()
        },
        SunControl {
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                -90_f32.to_radians(),
                0_f32.to_radians(),
                0_f32.to_radians(),
            ),
        },
    ));
}

#[derive(Component)]
pub struct SunControl {
    pub rotation: Quat,
}

fn update_sun(mut query: Query<(&SunControl, &mut Transform), Changed<SunControl>>) {
    let Ok((sun_control, mut tx)) = query.get_single_mut() else {
        return;
    };

    tx.rotation = sun_control.rotation;
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

pub const SPACING: i32 = 200;
const MAX_SIDE: f32 = 30.0;
const MAX_HEIGHT: f32 = 300.0;
const ACTIVE_BLOCK_DISTANCE: i32 = 20;

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
    rand: Res<Rand>,
) {
    let Ok(BlockPos(px, pz)) = query.get_single() else {
        return;
    };

    let px = *px;
    let pz = *pz;

    let mut active_block_positions = HashSet::new();

    let mut num_misses = 0;
    let mut num_hits = 0;

    for z in (pz - ACTIVE_BLOCK_DISTANCE)..(pz + ACTIVE_BLOCK_DISTANCE) {
        for x in (px - ACTIVE_BLOCK_DISTANCE)..(px + ACTIVE_BLOCK_DISTANCE) {
            let block_pos = (x, z);

            // Perlin always returns 0 for whole numbers so need to multiply by a coefficient to maker finer grained samplings
            let n = rand.perlin.get([x as f64 * 0.2, z as f64 * 0.2]);
            if n <= 0.0 {
                continue;
            }

            active_block_positions.insert(block_pos);

            if block_positions.contains(&block_pos) {
                num_hits += 1;
            } else {
                num_misses += 1;

                let height = MAX_HEIGHT * n as f32;
                let side = MAX_SIDE;

                let building_pos =
                    Vec3::new((x * SPACING) as f32, height * 0.5, (z * SPACING) as f32);

                let building_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(side, height, side))),
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_translation(building_pos),
                        ..default()
                    })
                    .insert(RigidBody::Fixed)
                    .insert(Collider::cuboid(side / 2.0, height / 2.0, side / 2.0))
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
