use bevy::{
    math::{vec2, vec3},
    prelude::*,
};

use crate::plane::spec::PlaneSpec;

use super::HudModel;

#[derive(Component)]
pub struct HudAirspeed;

#[derive(Component)]
pub enum HudLabel {
    Altitude,
    Airspeed,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 16.0,
        color: Color::ORANGE,
    };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("000", text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(vec3(100., -10., 0.)),
            ..default()
        },
        HudLabel::Altitude,
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("000", text_style.clone())
                .with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(vec3(100., 10., 0.)),
            ..default()
        },
        HudAirspeed,
        HudLabel::Airspeed,
    ));
}

pub fn hud_indicators(
    hud_model: Query<&HudModel>,
    mut labels_query: Query<(&mut Text, &HudLabel)>,
) {
    let Ok(hud) = hud_model.get_single() else {
        return;
    };

    for (mut text, label) in labels_query.iter_mut() {
        match label {
            HudLabel::Airspeed => {
                text.sections[0].value = format!("{:0width$.1}", hud.airspeed.abs(), width = 5)
            }
            HudLabel::Altitude => {
                text.sections[0].value = format!("{:0width$.1}", hud.altitude.abs(), width = 5)
            }
        }
    }
}

pub fn hud_gizmos(
    hud_model: Query<&HudModel>,
    plane_spec_query: Query<&PlaneSpec>,
    mut gizmos: Gizmos,
) {
    let Ok(hud) = hud_model.get_single() else {
        return;
    };
    let Ok(spec) = plane_spec_query.get_single() else {
        return;
    };

    let h = 100.;
    let x = -100.;
    let y = (-h / 2.) + (h / hud.max_thrust * hud.thrust);
    gizmos.rect_2d(vec2(x, 0.), 0., vec2(12., h + 8.), Color::ORANGE);
    gizmos.line_2d(vec2(x - 5., y), vec2(x + 5., y), Color::ORANGE);

    let y = 150.;
    gizmos.line_2d(vec2(-100., y), vec2(100., y), Color::ORANGE);
    let x = 100. / spec.wings.max_control_angle * hud.ailerons;
    gizmos.line_2d(vec2(x, y - 5.), vec2(x, y + 5.), Color::ORANGE);

    let y = 180.;
    gizmos.line_2d(vec2(-100., y), vec2(100., y), Color::ORANGE);
    let x = 100. / spec.tail.vertical.max_control_angle * hud.rudder;
    gizmos.line_2d(vec2(x, y - 5.), vec2(x, y + 5.), Color::ORANGE);

    let x = 150.;
    gizmos.line_2d(vec2(x, -100.), vec2(x, 100.), Color::ORANGE);
    let y = 100. / spec.tail.horizontal.max_control_angle * hud.elevators;
    gizmos.line_2d(vec2(x - 5., y), vec2(x + 5., y), Color::ORANGE);
}
