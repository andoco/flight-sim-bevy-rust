use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(attach_to_follow);
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Follow;

fn setup(mut commands: Commands) {
    commands
        .spawn((Camera3dBundle { ..default() },))
        .insert(MainCamera);

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        camera: Camera {
            order: 1,

            ..default()
        },
        ..default()
    });
}

fn attach_to_follow(
    mut commands: Commands,
    follow_query: Query<Entity, Added<Follow>>,
    mut camera_query: Query<(Entity, &mut Transform), With<MainCamera>>,
) {
    let Ok(follow_entity) = follow_query.get_single() else {
        return;
    };
    let Ok((camera_entity, mut camera_tx)) = camera_query.get_single_mut() else {
        return;
    };

    commands.entity(camera_entity).set_parent(follow_entity);

    camera_tx.translation = Vec3::new(0., 5.0, 20.);
}
