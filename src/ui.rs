mod spec;

use std::{f32::consts::PI, time::Duration};

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_egui::{
    egui::{self, Color32, FontDefinitions, RichText, Ui},
    EguiContexts, EguiPlugin,
};

use crate::{
    camera::FogControl,
    plane::{
        spec::PlaneSpec, AirfoilPosition, Airspeed, AngleOfAttack, BuildPlaneEvent, Lift,
        PlaneFlight, Side, Thrust,
    },
    world::SunControl,
};

use self::spec::{PlaneSpecModel, WingModel};

pub struct HudUiPlugin;

impl Plugin for HudUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, (setup, setup_indicators))
            .add_systems(Update, update_hud_ui)
            .add_systems(
                Update,
                update_hud_model.run_if(on_timer(Duration::from_millis(100))),
            );
    }
}

#[derive(Default)]
struct AirfoilModel {
    lift: f32,
    aoa: f32,
}

#[derive(Component, Default)]
struct HudModel {
    fps: f32,
    altitude: f32,
    thrust: f32,
    airspeed: f32,
    bearing: f32,
    wing_left: AirfoilModel,
    wing_right: AirfoilModel,
    tail_wing_left: AirfoilModel,
    tail_wing_right: AirfoilModel,
    weight: f32,
    drag: f32,
    show_environment: bool,
    show_build: bool,
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

    commands.spawn((
        HudModel::default(),
        PlaneSpecModel::new(&PlaneSpec::default()),
    ));
}

fn update_hud_model(
    plane_query: Query<(&GlobalTransform, &PlaneFlight, &Thrust, &Airspeed)>,
    airfoil_query: Query<(&AirfoilPosition, &AngleOfAttack, &Lift)>,
    mut model_query: Query<&mut HudModel>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok((global_tx,  flight, Thrust(thrust), Airspeed(airspeed))) = plane_query.get_single() else {
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
    model.airspeed = *airspeed;
    model.drag = flight.drag;
    model.thrust = *thrust;
    model.weight = flight.weight;
    model.bearing = global_tx
        .compute_transform()
        .rotation
        .to_euler(EulerRot::XYZ)
        .1
        .to_degrees();

    for (position, AngleOfAttack(aoa), Lift(lift)) in airfoil_query.iter() {
        match position {
            crate::plane::AirfoilPosition::Wing(Side::Left) => {
                model.wing_left = AirfoilModel {
                    lift: *lift,
                    aoa: aoa.to_degrees(),
                };
            }
            crate::plane::AirfoilPosition::Wing(Side::Right) => {
                model.wing_right = AirfoilModel {
                    lift: *lift,
                    aoa: aoa.to_degrees(),
                };
            }
            crate::plane::AirfoilPosition::TailWing(Side::Left) => {
                model.tail_wing_left = AirfoilModel {
                    lift: *lift,
                    aoa: aoa.to_degrees(),
                };
            }
            crate::plane::AirfoilPosition::TailWing(Side::Right) => {
                model.tail_wing_right = AirfoilModel {
                    lift: *lift,
                    aoa: aoa.to_degrees(),
                };
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct Vec3Model {
    x: String,
    y: String,
    z: String,
}

impl Vec3Model {
    fn new(value: Vec3) -> Self {
        Self {
            x: value.x.to_string(),
            y: value.y.to_string(),
            z: value.z.to_string(),
        }
    }
}

trait UiExt {
    fn vec3(&mut self, label: &str, value: &mut Vec3Model);
    fn wing(&mut self, label: &str, value: &mut WingModel);
}

impl UiExt for Ui {
    fn vec3(&mut self, label: &str, value: &mut Vec3Model) {
        self.label(label);
        self.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("x");
                ui.text_edit_singleline(&mut value.x);
            });
            ui.horizontal(|ui| {
                ui.label("y");
                ui.text_edit_singleline(&mut value.y);
            });
            ui.horizontal(|ui| {
                ui.label("z");
                ui.text_edit_singleline(&mut value.z);
            });
        });
    }

    fn wing(&mut self, label: &str, value: &mut WingModel) {
        self.label(label);
        self.group(|ui| {
            ui.vec3("size", &mut value.size);
            ui.label("lift coefficient");
            ui.group(|ui| {
                ui.label("elements");
                ui.text_edit_singleline(&mut value.elements);
                ui.label("knots");
                ui.text_edit_singleline(&mut value.knots);
            });
        });
    }
}

pub fn float_label(ui: &mut Ui, txt: &str, val: f32, color: Color32, width: usize) {
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
}

fn update_hud_ui(
    mut contexts: EguiContexts,
    mut model_query: Query<&mut HudModel>,
    mut plane_spec_model_query: Query<&mut PlaneSpecModel>,
    mut fog_control: Query<&mut FogControl>,
    mut sun_control: Query<&mut SunControl>,
    mut build_plane_event: EventWriter<BuildPlaneEvent>,
) {
    let Ok(mut model) = model_query.get_single_mut() else {
        return;
    };
    let Ok(mut plane_spec_model) = plane_spec_model_query.get_single_mut() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    let width = 10;
    let normal_color = Color32::WHITE;

    let lift_color = |lift: f32| -> Color32 {
        match lift {
            l if l < 0.0 => Color32::RED,
            _ => Color32::GREEN,
        }
    };

    egui::Window::new("Environment")
        .open(&mut model.show_environment)
        .show(ctx, |ui| {
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

    egui::Window::new("Build")
        .open(&mut model.show_build)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let space = 10.;
                ui.vec3("fuselage", &mut plane_spec_model.fuselage);
                ui.add_space(space);
                ui.wing("wings", &mut plane_spec_model.wings);
                ui.add_space(space);
                ui.vec3("tail", &mut plane_spec_model.tail);
                ui.add_space(space);
                ui.wing("tail horizontal", &mut plane_spec_model.tail_horizontal);
                ui.add_space(space);
                ui.wing("tail vertical", &mut plane_spec_model.tail_vertical);
                ui.add_space(space);

                if ui.button("Build").clicked() {
                    build_plane_event.send(BuildPlaneEvent(plane_spec_model.to_spec()));
                }
            });
        });

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Restart").clicked() {
                build_plane_event.send(BuildPlaneEvent(plane_spec_model.to_spec()));
            }
            if ui.button("Build").clicked() {
                model.show_build = !model.show_build;
            }
            if ui.button("Environment").clicked() {
                model.show_environment = !model.show_environment;
            }

            float_label(ui, "fps", model.fps, normal_color, width);
            float_label(ui, "weight", model.weight, normal_color, width);
            float_label(ui, "altitude", model.altitude, normal_color, width);
            float_label(ui, "airspeed", model.airspeed, normal_color, width);
            float_label(ui, "drag", model.drag, normal_color, width);
            float_label(ui, "thrust", model.thrust, normal_color, width);
            float_label(ui, "bearing", model.bearing, normal_color, width);
        });

        ui.horizontal(|ui| {
            let groups = [
                ("wing_left", model.wing_left.lift, model.wing_left.aoa),
                ("wing_right", model.wing_right.lift, model.wing_right.aoa),
                (
                    "tail_left",
                    model.tail_wing_left.lift,
                    model.tail_wing_left.aoa,
                ),
                (
                    "tail_right",
                    model.tail_wing_right.lift,
                    model.tail_wing_right.aoa,
                ),
            ];

            for (label, lift, aoa) in groups.iter() {
                ui.group(|ui| {
                    ui.label(*label);
                    float_label(ui, "aoa", *aoa, normal_color, width);
                    float_label(ui, "lift", *lift, lift_color(*lift), width);
                });
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
