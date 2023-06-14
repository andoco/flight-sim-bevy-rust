mod camera;
mod plane;
mod world;

use bevy::prelude::*;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldPlugin)
        .run();
}
