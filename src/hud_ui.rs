use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, RichText, Ui},
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

    let float_label = |ui: &mut Ui, txt: &str, val: f32, color: Color32| {
        ui.label(
            RichText::new(format!(
                "{txt}: {:0width$.4}",
                val,
                txt = txt,
                width = width
            ))
            .color(color),
        );
    };

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        let c = Color32::WHITE;

        let lift_color = match flight.lift {
            l if l < flight.weight => Color32::RED,
            _ => Color32::GREEN,
        };

        ui.horizontal(|ui| {
            float_label(ui, "altitude", global_tx.translation().y, c);
            float_label(ui, "velocity", velocity.linvel.length(), c);
            float_label(ui, "airspeed", flight.airspeed, c);
            float_label(ui, "aoa", flight.angle_of_attack.to_degrees(), c);
            float_label(ui, "weight", flight.weight, c);
            float_label(ui, "lift", flight.lift, lift_color);
            float_label(ui, "drag", flight.drag, c);
            float_label(ui, "thrust", flight.thrust, c);
        });
    });
}
