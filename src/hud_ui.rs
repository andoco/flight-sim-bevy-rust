use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier3d::prelude::Velocity;

use crate::plane::{Plane, PlaneFlight};

pub struct HudUiPlugin;

impl Plugin for HudUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(hud_ui);
    }
}

fn hud_ui(
    mut contexts: EguiContexts,
    plane_query: Query<(&GlobalTransform, &Velocity, &PlaneFlight), With<Plane>>,
) {
    let Ok((global_tx, velocity, flight)) = plane_query.get_single() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.label(format!(
            "altitude = {:.4}, velocity = {:.4}, aoa = {:.4}, lift = {:.4}, thrust = {:.4}",
            global_tx.translation().y,
            velocity.linvel.length(),
            flight.angle_of_attack.to_degrees(),
            flight.lift,
            flight.thrust
        ));
    });
}
