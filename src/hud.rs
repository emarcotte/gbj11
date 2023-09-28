use bevy::prelude::*;

use crate::{GBJAssets, GameTimer, Player, C0, C1};

#[derive(Component, Default)]
pub(crate) struct TimerWidget {}

#[derive(Component, Default)]
pub(crate) struct HPWidget {}

#[derive(Component, Default)]
pub(crate) struct XPWidget {}

pub(crate) fn setup(assets: Res<'_, GBJAssets>, mut commands: Commands<'_, '_>) {
    let text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 14.0,
        color: Color::hex(C0).unwrap(),
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.),
                width: Val::Percent(100.),
                height: Val::Px(6.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: BackgroundColor::from(Color::hex(C1).unwrap()),
            ..default()
        })
        .with_children(|parent| {
            let text_widget_style = Style {
                margin: UiRect::top(Val::Px(-7.)),
                ..default()
            };
            parent.spawn((
                TextBundle::from_section("", text_style.clone())
                    .with_style(text_widget_style.clone()),
                TimerWidget {},
            ));

            parent.spawn((
                TextBundle::from_section("", text_style.clone())
                    .with_style(text_widget_style.clone()),
                HPWidget {},
            ));

            parent.spawn((
                TextBundle::from_section("", text_style).with_style(text_widget_style.clone()),
                XPWidget {},
            ));
        });
}

pub(crate) fn update_xp(player: Query<&Player>, mut text_widget: Query<&mut Text, With<XPWidget>>) {
    let Ok(mut text) = text_widget.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    text.sections[0].value = format!("Next Lvl: {}", player.xp_to_level);
}

// TODO: Could use an event here...
pub(crate) fn update_hp(player: Query<&Player>, mut text_widget: Query<&mut Text, With<HPWidget>>) {
    let Ok(mut text) = text_widget.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    text.sections[0].value = format!("HP: {}", player.hp);
}

pub(crate) fn update_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut text_widget: Query<&mut Text, With<TimerWidget>>,
) {
    let Ok(mut text) = text_widget.get_single_mut() else {
        return;
    };

    if game_timer.ends_in > 0.0f32 {
        text.sections[0].value = format!("{}", game_timer.ends_in as u32);
        game_timer.ends_in -= time.delta_seconds();
    } else {
        text.sections[0].value = "OH SHIT!".to_string();
    }
}
