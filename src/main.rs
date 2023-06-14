mod camera;
mod hud_ui;
mod plane;
mod world;

use bevy::prelude::*;
use hud_ui::HudUiPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldPlugin)
        .add_plugin(HudUiPlugin)
        .run();
}
