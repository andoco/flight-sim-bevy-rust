use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update_fog, attach_to_follow));
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Follow(pub FollowKind);

pub enum FollowKind {
    Behind,
    Above,
    Inside,
}

fn setup(mut commands: Commands) {
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

#[derive(Component)]
pub struct FogControl {
    pub visibility: f32,
    pub extinction_color: Color,
    pub inscattering_color: Color,
}

fn update_fog(mut query: Query<(&mut FogSettings, &FogControl), Changed<FogControl>>) {
    let Ok((mut fog_settings, fog_control)) = query.get_single_mut() else {
        return;
    };

    let new_falloff = FogFalloff::from_visibility_colors(
        fog_control.visibility,
        fog_control.extinction_color,
        fog_control.inscattering_color,
    );

    fog_settings.falloff = new_falloff;
}

fn attach_to_follow(mut commands: Commands, follow_query: Query<(Entity, &Follow), Added<Follow>>) {
    let Ok((follow_entity, Follow(follow_kind))) = follow_query.get_single() else {
        return;
    };

    info!("Creating follow camera");

    let mut camera_tx = Transform::default();

    match follow_kind {
        FollowKind::Behind => {
            info!("Follow behind");
            camera_tx.translation = Vec3::new(0., 5.0, 20.);
            camera_tx.rotation = Quat::default();
        }
        FollowKind::Above => {
            info!("Follow above");
            camera_tx.translation = Vec3::new(0., 150.0, 0.);
            camera_tx.rotation = Quat::from_rotation_x(-90_f32.to_radians());
        }
        FollowKind::Inside => {
            info!("Follow inside");
            camera_tx.translation = Vec3::new(0., 0., 0.);
            camera_tx.rotation = Quat::default();
        }
    };

    commands
        .spawn((
            MainCamera,
            Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::rgb(0.5, 0.5, 0.8)),
                    ..default()
                },
                camera: Camera {
                    order: 0,
                    ..default()
                },
                transform: camera_tx,
                ..default()
            },
            FogSettings {
                color: Color::rgba(0.1, 0.2, 0.4, 1.0),
                directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
                directional_light_exponent: 30.0,
                falloff: FogFalloff::from_visibility_colors(
                    1500.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
            },
            FogControl {
                visibility: 1500.0,
                extinction_color: Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                inscattering_color: Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
            },
        ))
        .set_parent(follow_entity);

    info!("Following {:?}", follow_entity);
}
