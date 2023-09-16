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

#[derive(Bundle)]
struct PlayerBundle {
    #[bundle()]
    sprite: SpriteBundle,
}

#[derive(AssetCollection, Resource)]
struct GBJAssets {
    #[asset(path = "player.png")]
    player: Handle<Image>,
}

#[derive(States, Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Setup,
    Playing,
}

/*
const C0: &str = "000000";
const C1: &str = "F0F8BF";
const C2: &str = "DF904F";
 */
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
    .insert_resource(ClearColor(Color::hex(C3).unwrap()))
    .add_plugins(loading_plugin)
    .add_loading_state(loading_state)
    .add_collection_to_loading_state::<_, GBJAssets>(loading_game_state)
    .add_state::<GameState>()
    // Fix sprite bleed
    .insert_resource(Msaa::Off)
    .add_systems(Update, bevy::window::close_on_esc)
    .add_systems(OnEnter(GameState::Setup), setup)
    .run();
}

fn setup(assets: Res<GBJAssets>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(PlayerBundle {
        sprite: SpriteBundle {
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::Center,
                ..default()
            },
            texture: assets.player.clone(),
            ..default()
        },
    });
}
