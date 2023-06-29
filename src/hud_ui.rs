use std::{f32::consts::PI, time::Duration};

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_egui::{
    egui::{self, Color32, FontDefinitions, RichText, Ui},
    EguiContexts, EguiPlugin,
};

use crate::{camera::FogControl, plane::PlaneFlight, world::SunControl};

pub struct HudUiPlugin;

impl Plugin for HudUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_startup_system(setup)
            .add_startup_system(setup_indicators)
            .add_system(update_hud_ui)
            .add_system(update_hud_model.run_if(on_timer(Duration::from_millis(100))));
    }
}

#[derive(Component, Default)]
struct HudModel {
    fps: f32,
    altitude: f32,
    thrust: f32,
    angle_of_attack: f32,
    airspeed: f32,
    lift: f32,
    weight: f32,
    drag: f32,
}

fn setup(mut commands: Commands, mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "FiraMono-Medium".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/FiraMono-Medium.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "FiraMono-Medium".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("FiraMono-Medium".to_owned());

    ctx.set_fonts(fonts);

    commands.spawn(HudModel::default());
}

fn update_hud_model(
    plane_query: Query<(&GlobalTransform, &PlaneFlight)>,
    mut model_query: Query<&mut HudModel>,
    diagnostics: Res<Diagnostics>,
) {
    let Ok((global_tx,  flight)) = plane_query.get_single() else {
        return;
    };

    let Ok(mut model) = model_query.get_single_mut() else {
        return;
    };

    model.fps = diagnostics
        .get_measurement(FrameTimeDiagnosticsPlugin::FPS)
        .map(|m| m.value)
        .unwrap_or(-1.0) as f32;

    model.altitude = global_tx.translation().y;
    model.airspeed = flight.airspeed;
    model.angle_of_attack = flight.angle_of_attack;
    model.drag = flight.drag;
    model.lift = flight.lift;
    model.thrust = flight.thrust;
    model.weight = flight.weight;
}

fn update_hud_ui(
    mut contexts: EguiContexts,
    model_query: Query<&HudModel>,
    mut fog_control: Query<&mut FogControl>,
    mut sun_control: Query<&mut SunControl>,
) {
    let Ok(model) = model_query.get_single() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    let width = 10;

    let float_label = |ui: &mut Ui, txt: &str, val: f32, color: Color32| {
        let sign_char = match val.is_sign_positive() {
            true => "+",
            false => "-",
        };

        ui.label(
            RichText::new(format!(
                "{txt}: {sign_char}{:0width$.4}",
                val.abs(),
                txt = txt,
                width = width,
                sign_char = sign_char,
            ))
            .color(color),
        );
    };

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        let c = Color32::WHITE;

        let lift_color = match model.lift {
            l if l < model.weight => Color32::RED,
            _ => Color32::GREEN,
        };

        ui.horizontal(|ui| {
            float_label(ui, "fps", model.fps, c);
            float_label(ui, "altitude", model.altitude, c);
            float_label(ui, "airspeed", model.airspeed, c);
            float_label(ui, "aoa", model.angle_of_attack.to_degrees(), c);
            float_label(ui, "weight", model.weight, c);
            float_label(ui, "lift", model.lift, lift_color);
            float_label(ui, "drag", model.drag, c);
            float_label(ui, "thrust", model.thrust, c);
        });

        ui.horizontal(|ui| {
            if let Ok(mut fog_control) = fog_control.get_single_mut() {
                ui.group(|ui| {
                    ui.label("Fog");
                    ui.add(
                        egui::Slider::new(&mut fog_control.visibility, 0.0..=5000.0)
                            .text("visibility"),
                    );
                });
            }

            if let Ok(mut sun_control) = sun_control.get_single_mut() {
                let (mut y, mut x, z) = sun_control.rotation.to_euler(EulerRot::YXZ);

                ui.group(|ui| {
                    ui.label("Sun direction");
                    ui.add(egui::Slider::new(&mut x, 0.0..=PI * 2.0).text("x"));
                    ui.add(egui::Slider::new(&mut y, 0.0..=PI * 2.0).text("y"));
                });

                sun_control.rotation = Quat::from_euler(EulerRot::YXZ, y, x, z);
            }
        });
    });
}

fn setup_indicators(mut commands: Commands) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::ORANGE,
            custom_size: Some(Vec2::new(50.0, 2.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    });
}
