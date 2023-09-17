//! Entrypoint for the game jam
// Turn on some more aggressive warnings from clippy. They shouldn't break the
// build, but should tell you if you're doing something crazy.
#![warn(clippy::pedantic)]
// I hate broken links.
#![deny(rustdoc::broken_intra_doc_links)]
// Bevy passes queries and things by default as values which is a bit hard to
// work around.
#![allow(clippy::needless_pass_by_value)]
// If it turns out we're killing precision we can open these up but they're off
// by default so probably not a big deal
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![warn(
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    missing_docs,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications,
    variant_size_differences
)]
// I'm not sure i like this 2018 idiom. Can debate it later.
#![allow(elided_lifetimes_in_paths)]

use bevy::{prelude::*, window::WindowResolution};
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;

#[derive(Component, Clone, Default)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

// TODO: If I was really clever I could figure out how to make this a proper full
// asset with image and animations combined...
#[derive(Resource)]
struct AnimationTables {
    player: PlayerAnimationTable,
}

struct PlayerAnimationTable {
    // idle: AnimationIndices,
    flying: AnimationIndices,
}

#[derive(Component, Default)]
struct AnimationTimer {
    timer: Timer,
}

#[derive(Component, Default)]
struct Player {}

#[derive(Resource)]
struct GameTimer {
    ends_in: f32,
}

#[derive(Bundle, Default)]
struct TimerBundle {
    #[bundle()]
    text: TextBundle,
    widget: TimerWidget,
}

#[derive(Component, Default)]
struct TimerWidget {}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    #[bundle()]
    sprite: SpriteSheetBundle,
    animation: AnimationIndices,
    animation_timer: AnimationTimer,
}

#[derive(AssetCollection, Resource)]
struct GBJAssets {
    #[asset(texture_atlas(tile_size_x = 18., tile_size_y = 18., columns = 6, rows = 1))]
    #[asset(path = "player.png")]
    player: Handle<TextureAtlas>,

    //    #[asset(path = "baddie1.png")]
    //   baddie1: Handle<Image>,

    //  #[asset(path = "bg.png")]
    // bg: Handle<Image>,
    #[asset(path = "ThatBoy.ttf")]
    font: Handle<Font>,
}

#[derive(States, Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Setup,
    Playing,
}

const C0: &str = "000000";
const C1: &str = "F0F8BF";
//const C2: &str = "DF904F";
const C3: &str = "AF2820";

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::default();

    let loading_game_state = GameState::Loading;
    let loading_state = LoadingState::new(loading_game_state);
    let loading_plugin = ProgressPlugin::new(loading_game_state).continue_to(GameState::Setup);

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    mode: bevy::window::WindowMode::Windowed,
                    transparent: true,
                    fit_canvas_to_parent: false,
                    canvas: Some("#bevy".to_owned()),
                    resolution: WindowResolution::new(160f32, 140f32),
                    resizable: false,
                    ..Default::default()
                }),
                ..default()
            })
            // Fix sprite blur
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins(loading_plugin)
    .insert_resource(ClearColor(Color::hex(C3).unwrap()))
    .add_loading_state(loading_state)
    .add_collection_to_loading_state::<_, GBJAssets>(loading_game_state)
    .add_state::<GameState>()
    // Fix sprite bleed
    .insert_resource(Msaa::Off)
    .insert_resource(AnimationTables {
        player: PlayerAnimationTable {
            // idle: AnimationIndices { first: 0, last: 0 },
            flying: AnimationIndices { first: 1, last: 5 },
        },
    })
    .add_systems(Update, bevy::window::close_on_esc)
    .add_systems(OnEnter(GameState::Setup), setup)
    .add_systems(
        Update,
        (animate, fly_in_a_circle, update_timer).run_if(in_state(GameState::Playing)),
    )
    .run();
}

fn setup(
    assets: Res<GBJAssets>,
    mut commands: Commands,
    animation_table: Res<AnimationTables>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(PlayerBundle {
        player: Player {},
        sprite: SpriteSheetBundle {
            texture_atlas: assets.player.clone(),
            sprite: TextureAtlasSprite {
                index: animation_table.player.flying.first,
                ..default()
            },
            ..default()
        },
        animation_timer: AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        },
        animation: animation_table.player.flying.clone(),
    });

    setup_hud(assets, commands);

    next_state.set(GameState::Playing);
}

fn setup_hud(assets: Res<'_, GBJAssets>, mut commands: Commands<'_, '_>) {
    let text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 28.0,
        color: Color::hex(C0).unwrap(),
    };

    commands.insert_resource(GameTimer { ends_in: 90f32 });

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.),
                width: Val::Percent(100.),
                height: Val::Px(14.),
                ..default()
            },
            background_color: BackgroundColor::from(Color::hex(C1).unwrap()),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((TimerBundle {
                text: TextBundle::from_section("Hi", text_style).with_style(Style {
                    margin: UiRect::top(Val::Px(-12.)),
                    ..default()
                }),
                ..default()
            },));
        });
}

fn update_timer(
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

fn fly_in_a_circle(time: Res<Time>, mut player: Query<&mut Transform, With<Player>>) {
    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    *player = Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, time.elapsed_seconds()))
        .with_translation(Vec3::X * 30.);
}

fn animate(
    time: Res<Time>,
    mut animated_sprites: Query<(
        &mut AnimationTimer,
        &AnimationIndices,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut timer, indices, mut sprite) in &mut animated_sprites {
        timer.timer.tick(time.delta());
        if timer.timer.finished() {
            let mut new_index = sprite.index + 1;
            if new_index > indices.last {
                new_index = indices.first;
            }
            sprite.index = new_index;
        }
    }
}
