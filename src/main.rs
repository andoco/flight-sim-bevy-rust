mod camera;
mod input;
mod physics;
mod plane;
mod ui;
mod world;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use ui::HudUiPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(HudUiPlugin)
        .run();
}
