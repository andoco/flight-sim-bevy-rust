use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::plane::{Plane, PlaneForce};

pub struct HudUiPlugin;

impl Plugin for HudUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(hud_ui);
    }
}

fn hud_ui(
    mut contexts: EguiContexts,
    plane_query: Query<(&GlobalTransform, &PlaneForce), With<Plane>>,
) {
    let Ok((global_tx, plane_force)) = plane_query.get_single() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.label(format!(
            "altitude = {}, lift = {}",
            global_tx.translation().y,
            plane_force.lift
        ));
    });
}
