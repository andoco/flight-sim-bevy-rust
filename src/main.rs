mod camera;
mod hud_ui;
mod plane;
mod world;

use bevy::prelude::*;
use hud_ui::HudUiPlugin;
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
        .add_plugin(WorldPlugin)
        .add_plugin(HudUiPlugin)
        .run();
}
