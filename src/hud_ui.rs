use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Ui},
    EguiContexts, EguiPlugin,
};
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

    let width = 10;

    let float_label = |ui: &mut Ui, txt: &str, val: f32| {
        ui.label(format!(
            "{txt}: {:0width$.4}",
            val,
            txt = txt,
            width = width
        ));
    };

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            float_label(ui, "altitude", global_tx.translation().y);
            float_label(ui, "velocity", velocity.linvel.length());
            float_label(ui, "airspeed", flight.airspeed);
            float_label(ui, "aoa", flight.angle_of_attack.to_degrees());
            float_label(ui, "weight", flight.weight);
            float_label(ui, "lift", flight.lift);
            float_label(ui, "drag", flight.drag);
            float_label(ui, "thrust", flight.thrust);
        });
    });
}
