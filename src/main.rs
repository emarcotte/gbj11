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
// Sadly some systems have super complex type signatures and I'm not sure how to refactor it right now?
#![allow(clippy::type_complexity)]
// Turn on some stuff that isn't in pedantic.
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

use std::time::Duration;

use animation::{animate, AnimationIndices, AnimationTimer};
use bevy::{prelude::*, window::WindowResolution};
use bevy_asset_loader::prelude::*;
use bevy_xpbd_2d::prelude::*;
use iyes_progress::prelude::*;
use leafwing_input_manager::prelude::*;
//use rand::prelude::SmallRng;
//use rand::Rng;
//use rand_seeder::Seeder;

mod animation;
mod hud;

#[derive(PhysicsLayer)]
enum Layer {
    Player,
    PlayerAttack,
    Enemy,
}

/*
#[derive(Resource)]

pub(crate) struct Randomizer {
    rng: SmallRng,
}

impl Default for Randomizer {
    fn default() -> Self {
        Randomizer {
            rng: Seeder::from("sup").make_rng(),
        }
    }
}
*/

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Move,
    A,
    B,
    Select,
    Start,
}

// TODO: If I was really clever I could figure out how to make this a proper full
// asset with image and animations combined...
#[derive(Resource)]
struct AnimationTables {
    player: PlayerAnimationTable,
    baddie1: BaddieAnimationTable,
    missle: MissleAnimationTable,
}

struct MissleAnimationTable {
    flying: AnimationIndices,
}
struct PlayerAnimationTable {
    // idle: AnimationIndices,
    flying: AnimationIndices,
}

struct BaddieAnimationTable {
    idle: AnimationIndices,
}

#[derive(Component, Default)]
struct Player {
    hp: u8,
    xp_to_level: u32,
}

#[derive(Component, Default)]
struct Baddie {}

#[derive(Resource)]
struct GameTimer {
    ends_in: f32,
}

#[derive(Bundle)]
struct PlayerBundle {
    #[bundle()]
    input_manager: InputManagerBundle<Action>,
    player: Player,
    #[bundle()]
    sprite: SpriteSheetBundle,
    animation: AnimationIndices,
    animation_timer: AnimationTimer,
    rigid_body: RigidBody,
    collider: Collider,
    external_force: ExternalForce,
    external_torque: ExternalTorque,
    angular_dampening: AngularDamping,
    linear_dampening: LinearDamping,
    collision_layer: CollisionLayers,
}

#[derive(Bundle)]
struct BaddieBundle {
    #[bundle()]
    sprite: SpriteSheetBundle,
    animation: AnimationIndices,
    animation_timer: AnimationTimer,
    baddie: Baddie,
    rigid_body: RigidBody,
    collider: Collider,
    external_force: ExternalForce,
    external_torque: ExternalTorque,
    angular_dampening: AngularDamping,
    collision_layer: CollisionLayers,
}

impl BaddieBundle {
    fn new(assets: &Res<'_, GBJAssets>, animation_table: &BaddieAnimationTable) -> Self {
        BaddieBundle {
            baddie: Baddie {},
            rigid_body: RigidBody::Dynamic,
            sprite: SpriteSheetBundle {
                atlas: TextureAtlas {
                    layout: assets.baddie1_layout.clone(),
                    index: animation_table.idle.first,
                },
                texture: assets.baddie1_image.clone(),
                ..default()
            },
            animation_timer: AnimationTimer {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            animation: animation_table.idle.clone(),
            collider: Collider::circle(16.0),
            external_force: ExternalForce::ZERO,
            angular_dampening: AngularDamping(0.0),
            external_torque: ExternalTorque::ZERO,
            collision_layer: CollisionLayers::new(
                [Layer::Enemy],
                [Layer::PlayerAttack, Layer::Player],
            ),
        }
    }
}

#[derive(Component, Default)]
struct PlayerAttack;

#[derive(Component, Default)]
struct Missle;

#[derive(Bundle)]
struct MissleBundle {
    #[bundle()]
    sprite: SpriteSheetBundle,
    animation: AnimationIndices,
    animation_timer: AnimationTimer,
    missle: Missle,
    player_attack: PlayerAttack,
    rigid_body: RigidBody,
    collider: Collider,
    external_force: ExternalForce,
    collision_layer: CollisionLayers,
}

impl MissleBundle {
    fn new(
        assets: &Res<'_, GBJAssets>,
        animation_table: &MissleAnimationTable,
        dir: Vec2,
        player_pos: &Vec3,
    ) -> Self {
        let mut xform = Transform::from_translation(*player_pos + dir.extend(0.0));
        xform.rotate_local_z(dir.y.atan2(dir.x) - std::f32::consts::PI / 2.0);

        MissleBundle {
            rigid_body: RigidBody::Dynamic,
            sprite: SpriteSheetBundle {
                atlas: TextureAtlas {
                    layout: assets.missle_layout.clone(),
                    index: animation_table.flying.first,
                },
                texture: assets.missle_image.clone(),
                transform: xform,
                ..default()
            },
            animation_timer: AnimationTimer {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            animation: animation_table.flying.clone(),
            collider: Collider::capsule(7.0, 3.0),
            external_force: ExternalForce::new(dir * 100.0).with_persistence(true),
            player_attack: PlayerAttack,
            missle: Missle,
            collision_layer: CollisionLayers::new([Layer::PlayerAttack], [Layer::Enemy]),
        }
    }
}

fn player_input_map() -> InputMap<Action> {
    let mut input_map = InputMap::default();
    input_map.insert(Action::A, KeyCode::KeyA);
    input_map.insert(Action::B, KeyCode::KeyB);
    input_map.insert(
        Action::Move,
        VirtualDPad {
            up: KeyCode::ArrowUp.into(),
            down: KeyCode::ArrowDown.into(),
            left: KeyCode::ArrowLeft.into(),
            right: KeyCode::ArrowRight.into(),
        },
    );
    input_map.insert(Action::Start, KeyCode::Enter);
    input_map.insert(Action::Start, KeyCode::NumpadEnter);
    input_map.insert(Action::Select, KeyCode::ShiftLeft);
    input_map.insert(Action::Select, KeyCode::ShiftRight);
    input_map
}

impl PlayerBundle {
    fn new(assets: &Res<'_, GBJAssets>, animation_table: &PlayerAnimationTable) -> Self {
        PlayerBundle {
            rigid_body: RigidBody::Dynamic,
            input_manager: InputManagerBundle::<Action> {
                action_state: ActionState::default(),
                input_map: player_input_map(),
            },
            player: Player {
                hp: 10,
                xp_to_level: 10,
            },
            sprite: SpriteSheetBundle {
                atlas: TextureAtlas {
                    layout: assets.player_layout.clone(),
                    index: animation_table.flying.first,
                },
                texture: assets.player_sprite.clone(),
                ..default()
            },
            animation_timer: AnimationTimer {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            animation: animation_table.flying.clone(),
            collider: Collider::circle(9.0),
            external_force: ExternalForce::ZERO,
            angular_dampening: AngularDamping(0.20),
            external_torque: ExternalTorque::ZERO,
            linear_dampening: LinearDamping(0.0),
            collision_layer: CollisionLayers::new([Layer::Player], [Layer::Enemy]),
        }
    }
}

#[derive(AssetCollection, Resource)]
struct GBJAssets {
    #[asset(key = "player.layout")]
    player_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "player.image")]
    player_sprite: Handle<Image>,

    #[asset(key = "missle.layout")]
    missle_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "missle.image")]
    missle_image: Handle<Image>,

    //  #[asset(path = "bg.png")]
    // bg: Handle<Image>,
    #[asset(key = "baddie1.layout")]
    baddie1_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "baddie1.image")]
    baddie1_image: Handle<Image>,

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

#[derive(Resource)]
struct BaddieSpawner {
    timer: Timer,
    rotation: f32,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::default();

    let loading_game_state = GameState::Loading;
    let loading_state = LoadingState::new(loading_game_state)
        .with_dynamic_assets_file::<StandardDynamicAssetCollection>("entities.assets.ron")
        .load_collection::<GBJAssets>();
    let loading_plugin = ProgressPlugin::new(loading_game_state).continue_to(GameState::Setup);

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    mode: bevy::window::WindowMode::Windowed,
                    transparent: true,
                    canvas: Some("#bevy".to_owned()),
                    resolution: WindowResolution::new(160f32, 140f32),
                    resizable: false,
                    ..Default::default()
                }),
                ..default()
            })
            // Fix sprite blur
            .set(ImagePlugin::default_nearest()),
        loading_plugin,
        InputManagerPlugin::<Action>::default(),
        PhysicsPlugins::default(),
    ))
    .add_loading_state(loading_state)
    .init_state::<GameState>()
    .insert_resource(ClearColor(Color::hex(C3).unwrap()))
    // .insert_resource(PhysicsDebugConfig::all())
    .insert_resource(MissleSpawnTimer::default())
    // Fix sprite bleed
    .insert_resource(Msaa::Off)
    .insert_resource(BaddieSpawner {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        rotation: 0.0,
    })
    .insert_resource(AnimationTables {
        player: PlayerAnimationTable {
            // idle: AnimationIndices { first: 0, last: 0 },
            flying: AnimationIndices { first: 1, last: 5 },
        },
        baddie1: BaddieAnimationTable {
            idle: AnimationIndices { first: 0, last: 0 },
        },
        missle: MissleAnimationTable {
            flying: AnimationIndices { first: 1, last: 2 },
        },
    })
    .insert_resource(GameTimer { ends_in: 90f32 })
    .insert_resource(Gravity(Vec2::ZERO))
    .add_systems(Update, bevy::window::close_on_esc)
    .add_systems(OnEnter(GameState::Setup), setup)
    .add_systems(
        Update,
        (
            animate,
            hud::update_xp,
            hud::update_hp,
            hud::update_timer,
            spawn_missle,
            player_inputs,
            spawn_baddies,
            despawn_far_missles,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (player_baddie_collision_handler)
            .before(player_inputs)
            .run_if(in_state(GameState::Playing)),
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
    commands.spawn(PlayerBundle::new(&assets, &animation_table.player));

    hud::setup(assets, commands);

    next_state.set(GameState::Playing);
}

fn player_inputs(
    mut player_query: Query<
        (
            &mut ExternalForce,
            &mut ExternalTorque,
            &mut LinearDamping,
            &Transform,
            &ActionState<Action>,
        ),
        With<Player>,
    >,
) {
    let Ok((mut force, mut torque, mut linear_dampening, transform, action_state)) =
        player_query.get_single_mut()
    else {
        return;
    };

    if action_state.pressed(&Action::Move) {
        if let Some(axis) = action_state.clamped_axis_pair(&Action::Move) {
            // The "Y" axis is oriented in alignment with the player's "UP" rather than camera, so rotate by the rotation of the entity itself when applying thrust.
            let y_axis = transform
                .rotation
                .mul_vec3(Vec3::new(0., axis.y(), 0.0))
                .truncate();
            // Torque is applied in the "opposite" direction from inputs
            *torque = torque
                .apply_torque(axis.x() * -20000.0)
                .with_persistence(false);
            *force = force.apply_force(y_axis * 1000.0).with_persistence(false);
        }
    }

    if action_state.pressed(&Action::B) {
        linear_dampening.0 = 5.0;
    } else {
        linear_dampening.0 = 0.0;
    }
}

fn player_baddie_collision_handler(
    mut collision_event_reader: EventReader<Collision>,
    baddies: Query<&Baddie, Without<Player>>,
    mut player: Query<(&mut Player, Entity)>,
    mut commands: Commands,
) {
    let Ok((mut player, player_ent)) = player.get_single_mut() else {
        return;
    };

    for Collision(contact) in &mut collision_event_reader.read() {
        if [contact.entity1, contact.entity2].contains(&player_ent) {
            if let Some(baddie) = [contact.entity1, contact.entity2]
                .iter()
                .find(|e| baddies.contains(**e))
            {
                damage_player(&mut player);
                unspawn_baddie(*baddie, &mut commands);
            }
        }
    }
}

fn damage_player(player: &mut Player) {
    if player.hp > 0 {
        player.hp -= 1;
    }
}

fn unspawn_baddie(entity: Entity, commands: &mut Commands) {
    if let Some(baddie) = commands.get_entity(entity) {
        // TODO: Explody animation?
        baddie.despawn_recursive();
    }
}

fn spawn_baddies(
    time: Res<Time>,
    mut spawn_time: ResMut<BaddieSpawner>,
    mut commands: Commands,
    animation_table: Res<AnimationTables>,
    assets: Res<GBJAssets>,
) {
    spawn_time.timer.tick(time.delta());
    if spawn_time.timer.just_finished() {
        let mut ent = commands.spawn(BaddieBundle::new(&assets, &animation_table.baddie1));
        ent.insert(Transform::from_translation(Vec3::new(
            spawn_time.rotation.cos() * 80.0,
            spawn_time.rotation.sin() * 80.0,
            0.0,
        )));
        spawn_time.rotation += std::f32::consts::PI / 4.0;
    }
}

#[derive(Resource)]
struct MissleSpawnTimer(Timer);

impl Default for MissleSpawnTimer {
    fn default() -> Self {
        MissleSpawnTimer(Timer::new(
            Duration::from_secs_f32(0.5f32),
            TimerMode::Repeating,
        ))
    }
}

fn spawn_missle(
    time: Res<Time>,
    mut missle_timer: ResMut<MissleSpawnTimer>,
    mut commands: Commands,
    animation_table: Res<AnimationTables>,
    assets: Res<GBJAssets>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_xform) = player.get_single() else {
        return;
    };

    missle_timer.0.tick(time.delta());

    // TODO: Should probably have some player state check to see if the player has missles, too
    if missle_timer.0.just_finished() {
        let spawn_time = time.elapsed_seconds();
        let dir = Vec2::new(spawn_time.cos() * 20.0, spawn_time.sin() * 20.0);

        // TODO: Should inherit player velocity
        commands.spawn(MissleBundle::new(
            &assets,
            &animation_table.missle,
            dir,
            &player_xform.translation,
        ));
    }
}

fn despawn_far_missles(
    player: Query<&Transform, With<Player>>,
    missles: Query<(Entity, &Transform), With<Missle>>,
    mut commands: Commands,
) {
    let Ok(player_xform) = player.get_single() else {
        return;
    };

    for (missle, xform) in &missles {
        if xform.translation.distance(player_xform.translation).abs() > 180.0 {
            commands.entity(missle).despawn_recursive();
        }
    }
}
