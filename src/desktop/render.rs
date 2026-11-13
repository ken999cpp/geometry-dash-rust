use super::{Game, Screen};
use crate::{
    constants,
    geometry::Point,
    level::{GROUND, LevelObject},
    simulation::{CEILING, PlayerMode, Status},
};
use bevy::{camera::ScalingMode, prelude::*, sprite::Text2dShadow};
use std::{collections::HashMap, sync::LazyLock};

static NATIVE_SIZES: LazyLock<HashMap<String, [f32; 2]>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/native_ui/game-sprites.json"))
        .expect("native game sprite sizes")
});

mod depth {
    pub const BACKGROUND: f32 = -200.;
    pub const OBJECT: f32 = 10.;
    pub const CHECKPOINT: f32 = 30.;
    pub const PLAYER: f32 = 40.;
    pub const PORTAL_FRONT: f32 = 60.;
    pub const GROUND: f32 = 80.;
    pub const SHADE: f32 = 81.;
    pub const PATTERN: f32 = 82.;
    pub const LINE: f32 = 83.;
}
#[derive(Component)]
struct ShipRider;
#[derive(Component)]
struct CheckpointVisual;
#[derive(Resource)]
struct TriggerArt {
    circle: Handle<Mesh>,
    black: Handle<ColorMaterial>,
    white: Handle<ColorMaterial>,
    fills: [Handle<ColorMaterial>; 4],
}

pub struct PresentationPlugin;
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct LevelVisual;
#[derive(Component)]
struct PlayerVisual;
#[derive(Component)]
struct Background(i32);
#[derive(Component, Clone, Copy)]
enum Floor {
    Base,
    Shade,
    Pattern,
    Line,
}
#[derive(Component)]
struct Ceiling;
#[derive(Component)]
enum MenuVisual {
    Tile(i32),
    Floor,
    Pattern,
    Shade,
    Line,
    Runner,
}
const MENU_TILE_WIDTH: f32 = 512. * constants::GD_SCALE as f32;
const MENU_GROUND_TILE_WIDTH: f32 = 128. * constants::GD_SCALE as f32;
const MENU_FLOOR_Y: f32 = 240.;
const GROUND_TILE_WIDTH: f32 = 440.;
const GROUND_STRIP_WIDTH: f32 = GROUND_TILE_WIDTH * 32.;
#[derive(Resource, Default)]
struct Trail(Handle<Mesh>);
#[derive(Component)]
struct WaveTrailVisual;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                rebuild,
                finish_loading,
                animate,
                update_wave_trail,
                update_ship_rider,
                sync_checkpoints,
                animate_menu,
                draw_helpers,
            )
                .chain()
                .in_set(super::FrameSet::World),
        );
    }
}
#[derive(Resource)]
struct PlayerImages([Handle<Image>; 10]);
#[derive(Resource, Default)]
struct ModeAnimations {
    robot_run: Vec<Handle<Image>>,
    robot_jump: Vec<Handle<Image>>,
    robot_fall: Vec<Handle<Image>>,
    spider_run: Vec<Handle<Image>>,
    spider_fall: Vec<Handle<Image>>,
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let trail_mesh = meshes.add(super::wave::ribbon(&[], 16.));
    commands.spawn((
        WaveTrailVisual,
        bevy::camera::visibility::NoFrustumCulling,
        Mesh2d(trail_mesh.clone()),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_xyz(0., 0., depth::PLAYER - 1.),
        Visibility::Hidden,
    ));
    commands.insert_resource(Trail(trail_mesh));
    commands.insert_resource(TriggerArt {
        circle: meshes.add(Circle::new(1.)),
        black: materials.add(Color::BLACK),
        white: materials.add(Color::WHITE),
        fills: [
            Color::srgb(0.1, 0.5, 1.),
            Color::srgb(0.2, 0.8, 0.25),
            Color::srgb(0.8, 0.15, 0.8),
            Color::srgb(1., 0.55, 0.05),
        ]
        .map(|c| materials.add(c)),
    });
    commands.insert_resource(PlayerImages([
        server.load("cube/Cube001.png"),
        server.load("cube/Cube002.png"),
        server.load("ship/Ship001.png"),
        server.load("wave/Wave001.png"),
        server.load("ball/Ball001.png"),
        server.load("spider/Spider001.png"),
        server.load("cube/Cube001_small.png"),
        server.load("cube/Cube002_small.png"),
        server.load("checkpoint.png"),
        server.load("robot/Robot001.png"),
    ]));
    let frames = |directory: &str, state: &str, count| {
        (1..=count)
            .map(|i| server.load(format!("{directory}/{state}_{i:03}.png")))
            .collect()
    };
    commands.insert_resource(ModeAnimations {
        robot_run: frames("robot", "Robot_run", 16),
        robot_jump: frames("robot", "Robot_jump_loop", 5),
        robot_fall: frames("robot", "Robot_fall_loop", 5),
        spider_run: frames("spider", "Spider_run", 7),
        spider_fall: frames("spider", "Spider_fall_loop", 5),
    });
    commands.spawn((
        Camera2d,
        MainCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: constants::VIEWPORT_HEIGHT as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
        camera_transform(),
    ));
}
fn camera_transform() -> Transform {
    Transform::from_xyz(1280., constants::VIEWPORT_HEIGHT as f32 / 2., 0.)
}

pub fn texture(object: &LevelObject) -> Option<String> {
    Some(match object.kind.as_str() {
        "block" => format!(
            "blocks/{}.png",
            object.texture.as_deref().unwrap_or("block1")
        ),
        "platform" => "platforms/platform.png".into(),
        "spike" => "spikes/spike.png".into(),
        "minispike" => "spikes/minispike.png".into(),
        "special_object" => "special_objects/specialobj1.png".into(),
        "jump_pad" => "jump_pads/jump_pad.png".into(),
        "yellow_orb" => "orbs/yellow_orb.png".into(),
        "cube_portal" => "portals/CubePortalback.png".into(),
        "ship_portal" => "portals/ShipPortalback.png".into(),
        "wave_portal" => "portals/WavePortalback.png".into(),
        "ball_portal" => "portals/BallPortalback.png".into(),
        "spider_portal" => "portals/SpiderPortalback.png".into(),
        "robot_portal" => "portals/RobotPortalback.png".into(),
        "start_pos" => "native_ui/edit_eStartPosBtn_001.png".into(),
        _ => return None,
    })
}
fn rebuild(
    mut commands: Commands,
    mut game: ResMut<Game>,
    server: Res<AssetServer>,
    trigger_art: Res<TriggerArt>,
    visuals: Query<Entity, With<LevelVisual>>,
) {
    if !game.scene_dirty {
        return;
    }
    game.scene_dirty = false;
    for e in &visuals {
        commands.entity(e).despawn();
    }
    if matches!(game.screen, Screen::Menu | Screen::Levels | Screen::Icons) {
        for index in -4..=4 {
            if game.screen == Screen::Levels && index != 0 {
                continue;
            }
            commands.spawn((
                LevelVisual,
                MenuVisual::Tile(index),
                Sprite {
                    image: server.load(if game.screen == Screen::Levels {
                        "native_ui/gradient.png"
                    } else {
                        "native_ui/background.png"
                    }),
                    custom_size: Some(Vec2::new(
                        if game.screen == Screen::Levels {
                            10000.
                        } else {
                            MENU_TILE_WIDTH
                        },
                        if game.screen == Screen::Levels {
                            constants::VIEWPORT_HEIGHT as f32
                        } else {
                            MENU_TILE_WIDTH
                        },
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    0.,
                    if game.screen == Screen::Levels {
                        constants::VIEWPORT_HEIGHT as f32 / 2.
                    } else {
                        MENU_TILE_WIDTH / 2.
                    },
                    depth::BACKGROUND,
                ),
            ));
        }
        if game.screen == Screen::Levels {
            return;
        }
        commands.spawn((
            LevelVisual,
            MenuVisual::Floor,
            Sprite::from_color(Color::srgb(0.1, 0.2, 0.8), Vec2::new(12000., 420.)),
            Transform::from_xyz(0., 30., depth::GROUND),
        ));
        commands.spawn((
            LevelVisual,
            MenuVisual::Pattern,
            Sprite {
                image: server.load("native_ui/ground.png"),
                custom_size: Some(Vec2::new(GROUND_STRIP_WIDTH, MENU_GROUND_TILE_WIDTH)),
                image_mode: SpriteImageMode::Tiled {
                    tile_x: true,
                    tile_y: false,
                    stretch_value: constants::GD_SCALE as f32 / 4.,
                },
                color: Color::WHITE,
                ..default()
            },
            Transform::from_xyz(
                0.,
                MENU_FLOOR_Y - MENU_GROUND_TILE_WIDTH / 2.,
                depth::PATTERN,
            ),
        ));
        commands.spawn((
            LevelVisual,
            MenuVisual::Shade,
            Sprite {
                image: server.load("ground/blackout.png"),
                custom_size: Some(Vec2::new(GROUND_STRIP_WIDTH, 240.)),
                ..default()
            },
            Transform::from_xyz(0., 120., depth::SHADE),
        ));
        commands.spawn((
            LevelVisual,
            MenuVisual::Line,
            Sprite::from_color(Color::WHITE, Vec2::new(12000., 5.)),
            Transform::from_xyz(0., MENU_FLOOR_Y, depth::LINE),
        ));
        commands.spawn((
            LevelVisual,
            MenuVisual::Runner,
            Sprite {
                image: server.load(if game.profile.cube_icon == 1 {
                    "cube/Cube002.png"
                } else {
                    "cube/Cube001.png"
                }),
                custom_size: Some(Vec2::splat(constants::CUBE_SIZE as f32)),
                ..default()
            },
            Transform::from_xyz(
                0.,
                MENU_FLOOR_Y + constants::CUBE_SIZE as f32 / 2.,
                depth::PLAYER,
            ),
        ));
        return;
    }
    let editing = game.screen == Screen::Editor;
    let level = if editing {
        game.editor.as_ref().map(|e| &e.level)
    } else {
        game.simulation.as_ref().map(|s| &s.level)
    };
    let show = editing
        || matches!(
            game.screen,
            Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
        );
    let background = level
        .map(|l| l.background.as_str())
        .unwrap_or("assets/background/stereomadness_bw.png");
    let native_editor_bg = editing && background == "assets/background/stereomadness_bw.png";
    for index in -4..=4 {
        commands.spawn((
            LevelVisual,
            Background(index),
            Sprite {
                image: server.load(if native_editor_bg {
                    "native_ui/background.png".to_owned()
                } else {
                    crate::level::asset_path(background)
                }),
                custom_size: Some(Vec2::new(
                    MENU_TILE_WIDTH,
                    constants::VIEWPORT_HEIGHT as f32,
                )),
                ..default()
            },
            Transform::from_xyz(
                0.,
                constants::VIEWPORT_HEIGHT as f32 / 2.,
                depth::BACKGROUND,
            ),
        ));
    }
    for (layer, path, height, z) in [
        (Floor::Base, None, 800., depth::GROUND),
        (
            Floor::Shade,
            Some("ground/blackout.png"),
            800.,
            depth::SHADE,
        ),
        (
            Floor::Pattern,
            Some("ground/blacksquare.png"),
            250.,
            depth::PATTERN,
        ),
        (Floor::Line, None, 6., depth::LINE),
    ] {
        let mut sprite = Sprite::from_color(Color::WHITE, Vec2::new(GROUND_STRIP_WIDTH, height));
        if let Some(path) = path {
            sprite.image = server.load(path);
        }
        if matches!(layer, Floor::Pattern) {
            sprite.image_mode = SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: 1.,
            };
            sprite.color = Color::srgba(1., 1., 1., 0.22);
        }
        commands.spawn((LevelVisual, layer, sprite, Transform::from_xyz(0., 0., z)));
    }
    commands.spawn((
        LevelVisual,
        Ceiling,
        Sprite::from_color(Color::srgb(0.04, 0.1, 0.5), Vec2::new(10000., 300.)),
        Transform::from_xyz(0., CEILING as f32 + 150., 80.),
    ));
    if !show {
        return;
    }
    let Some(level) = level else {
        return;
    };
    for object in &level.objects {
        if editing
            && game
                .editor
                .as_ref()
                .is_some_and(|e| !e.object_visible(object))
        {
            continue;
        }
        if !editing && matches!(object.kind.as_str(), "trigger" | "end_level" | "start_pos") {
            continue;
        }
        if editing && object.editor_marker() {
            spawn_trigger(&mut commands, object, &trigger_art);
            continue;
        }
        let center = object.rect().center;
        let size = object.size();
        let path = texture(object);
        let mut sprite = if let Some(path) = &path {
            Sprite::from_image(server.load(path.to_owned()))
        } else {
            Sprite::from_color(
                if object.supported() {
                    Color::srgb(0.5, 1., 0.8)
                } else {
                    Color::srgb(1., 0.1, 0.1)
                },
                Vec2::ONE,
            )
        };
        sprite.flip_x = object
            .extra
            .get("flip_x")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        sprite.flip_y = object
            .extra
            .get("flip_y")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let artwork_size = path
            .as_ref()
            .and_then(|p| NATIVE_SIZES.get(p))
            .map(|&[w, h]| {
                Vec2::new(w * object.scale().x as f32, h * object.scale().y as f32)
                    * constants::GD_SCALE as f32
            });
        sprite.custom_size = Some(if object.kind == "start_pos" {
            Vec2::new(80., 50.)
        } else if object.kind.ends_with("portal") {
            artwork_size.unwrap_or(Vec2::new(size.x as f32, size.y as f32))
        } else {
            Vec2::new(size.x as f32, size.y as f32)
        });
        if object.kind == "trigger" {
            sprite.color = Color::srgba(1., 0.5, 0.1, 0.7);
        }
        commands.spawn((
            LevelVisual,
            sprite,
            Transform {
                translation: Vec3::new(
                    center.x as f32,
                    center.y as f32,
                    if object.kind.ends_with("portal") {
                        0.
                    } else {
                        depth::OBJECT
                    },
                ),
                rotation: Quat::from_rotation_z((-object.rotation as f32).to_radians()),
                ..default()
            },
        ));
        if let Some(path) = path.filter(|p| p.ends_with("Portalback.png")) {
            commands.spawn((
                LevelVisual,
                Sprite {
                    image: server.load(path.replace("back", "front")),
                    custom_size: artwork_size,
                    flip_x: object
                        .extra
                        .get("flip_x")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    flip_y: object
                        .extra
                        .get("flip_y")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(center.x as f32, center.y as f32, depth::PORTAL_FRONT),
                    rotation: Quat::from_rotation_z((-object.rotation as f32).to_radians()),
                    ..default()
                },
            ));
        }
    }
    if !editing {
        let player = commands
            .spawn((
                LevelVisual,
                PlayerVisual,
                Sprite::default(),
                Transform::from_xyz(0., 0., depth::PLAYER),
            ))
            .id();
        let rider = commands
            .spawn((
                ShipRider,
                Sprite {
                    custom_size: Some(Vec2::splat(40.)),
                    ..default()
                },
                Transform::from_xyz(0., 31., 1.),
                Visibility::Hidden,
            ))
            .id();
        commands.entity(player).add_child(rider);
    }
}

fn finish_loading(
    mut game: ResMut<Game>,
    server: Res<AssetServer>,
    sprites: Query<&Sprite>,
    audio: Query<&AudioPlayer>,
    images: Res<PlayerImages>,
    animations: Res<ModeAnimations>,
) {
    if game.screen != Screen::Loading {
        return;
    }
    let music = game
        .simulation
        .as_ref()
        .map(|s| s.level.music.as_str())
        .unwrap_or("");
    let music_handle =
        (!music.is_empty()).then(|| server.load::<AudioSource>(super::audio::audio_path(music)));
    let ids = sprites
        .iter()
        .filter(|s| s.image != Handle::default())
        .map(|s| s.image.id().untyped())
        .chain(images.0.iter().map(|h| h.id().untyped()))
        .chain(
            animations
                .robot_run
                .iter()
                .chain(&animations.robot_jump)
                .chain(&animations.robot_fall)
                .chain(&animations.spider_run)
                .chain(&animations.spider_fall)
                .map(|h| h.id().untyped()),
        )
        .chain(audio.iter().map(|p| p.0.id().untyped()))
        .chain(music_handle.iter().map(|h| h.id().untyped()));
    let mut pending = false;
    for id in ids {
        if let Some(bevy::asset::LoadState::Failed(error)) = server.get_load_state(id) {
            game.status = format!("Asset loading failed: {error}");
            game.screen(Screen::Levels);
            return;
        }
        if !server.is_loaded_with_dependencies(id) {
            pending = true;
        }
    }
    if !pending {
        game.attempt.enter(super::attempt::Phase::Spawning);
        game.screen(Screen::Playing);
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn animate(
    mut game: ResMut<Game>,
    time: Res<Time<Fixed>>,
    windows: Query<&Window>,
    images: Res<PlayerImages>,
    animations: Option<Res<ModeAnimations>>,
    textures: Res<Assets<Image>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    mut players: Query<
        (&mut Transform, &mut Sprite, &mut Visibility),
        (With<PlayerVisual>, Without<MainCamera>),
    >,
    mut scenery: Query<
        (
            &mut Transform,
            &mut Sprite,
            Option<&Background>,
            Option<&Floor>,
            Option<&Ceiling>,
        ),
        (
            Without<MainCamera>,
            Without<PlayerVisual>,
            Or<(With<Background>, With<Floor>, With<Ceiling>)>,
        ),
    >,
    mut clear: ResMut<ClearColor>,
) {
    if game.clear_trail {
        game.clear_trail = false;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut camera, mut projection)) = camera.single_mut() else {
        return;
    };
    let editing = game.screen == Screen::Editor;
    let mut zoom = 1.;
    let mut camera_pos = Vec2::new(1280., constants::VIEWPORT_HEIGHT as f32 / 2.);
    let mut floor = GROUND as f32;
    let mut ground_color = Color::srgb(0.02, 0.07, 0.4);
    let mut bg_color = Color::srgb(0.1, 0.2, 0.7);
    let mut show_ceiling = false;
    let mut ceiling_y = CEILING as f32;
    let mut level_elapsed = 0.;
    if editing {
        if let Some(e) = &game.editor {
            zoom = e.zoom as f32;
            camera_pos = Vec2::new(e.camera.x as f32, e.camera.y as f32);
            bg_color = rgba(
                e.level
                    .initial_color(false, [40. / 255., 125. / 255., 1., 1.]),
            );
            ground_color = rgba(e.level.initial_color(true, [0., 0.4, 1., 1.]));
        }
    } else if matches!(
        game.screen,
        Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
    ) && let Some(sim) = &game.simulation
    {
        let p = &sim.state.player;
        let alpha = if game.screen == Screen::Playing
            && sim.status == Status::Running
            && game.attempt.playing()
            && !game.block_fixed_frame
            && game.exit_dialog.is_none()
        {
            time.overstep_fraction_f64()
        } else {
            1.
        };
        let pos = p.previous + (p.position - p.previous) * alpha;
        level_elapsed = if sim.status == Status::Running && sim.state.tick > 0 {
            (sim.state.elapsed - crate::constants::DT * (1. - alpha)).max(0.)
        } else {
            sim.state.elapsed
        };
        let width = constants::VIEWPORT_HEIGHT as f32 * window.width() / window.height();
        camera_pos = Vec2::new(
            pos.x as f32 + width / 2. - constants::PLAYER_SCREEN_X as f32,
            constants::GAME_CAMERA_Y as f32 + sim.state.camera_y as f32,
        );
        floor = sim.state.floor as f32;
        ceiling_y = sim.state.ceiling as f32;
        ground_color = rgba(sim.state.ground.current);
        bg_color = rgba(sim.state.background.current);
        show_ceiling = matches!(
            p.mode,
            PlayerMode::Ship | PlayerMode::Wave | PlayerMode::Ball | PlayerMode::Spider
        ) || p.gravity > 0.;
        for (mut transform, mut sprite, mut visibility) in &mut players {
            let angle = p.rotation;
            transform.rotation = Quat::from_rotation_z(angle.to_radians() as f32);
            let offset = if p.mode == PlayerMode::Ship {
                transform.rotation * Vec3::new(0., -11., 0.)
            } else {
                Vec3::ZERO
            };
            transform.translation = Vec3::new(pos.x as f32, pos.y as f32, depth::PLAYER) + offset;
            sprite.image = match p.mode {
                PlayerMode::Cube => images.0[game.profile.cube_icon.min(1)].clone(),
                PlayerMode::Ship => images.0[2].clone(),
                PlayerMode::Wave => images.0[3].clone(),
                PlayerMode::Ball => images.0[4].clone(),
                PlayerMode::Spider => images.0[5].clone(),
                PlayerMode::Robot => images.0[9].clone(),
            };
            if let Some(animations) = &animations {
                let frames = match (p.mode, p.grounded) {
                    (PlayerMode::Robot, true) => &animations.robot_run,
                    (PlayerMode::Robot, false) if p.velocity.y * p.gravity < 0. => {
                        &animations.robot_jump
                    }
                    (PlayerMode::Robot, false) => &animations.robot_fall,
                    (PlayerMode::Spider, true) => &animations.spider_run,
                    (PlayerMode::Spider, false) => &animations.spider_fall,
                    _ => &animations.robot_run[..0],
                };
                if !frames.is_empty() {
                    sprite.image =
                        frames[(sim.state.elapsed * 30.) as usize % frames.len()].clone();
                }
            }
            // UHD artwork uses four pixels per native unit. Its transparent
            // margins and pivot belong to the sprite, not the collision box.
            sprite.custom_size = Some(
                textures
                    .get(&sprite.image)
                    .map(|image| {
                        Vec2::new(image.width() as f32, image.height() as f32)
                            * (constants::GD_SCALE as f32 / 4.)
                    })
                    .unwrap_or(Vec2::new(p.mode.size().x as f32, p.mode.size().y as f32)),
            );
            sprite.color = Color::WHITE;
            sprite.flip_y = p.gravity > 0.
                && matches!(
                    p.mode,
                    PlayerMode::Ship | PlayerMode::Robot | PlayerMode::Spider
                );
            *visibility = if matches!(sim.status, Status::Dead(_)) {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale = 1. / zoom;
    }
    camera.translation.x = camera_pos.x;
    camera.translation.y = camera_pos.y;
    clear.0 = bg_color;
    for (mut transform, mut sprite, bg, ground, ceiling) in &mut scenery {
        if let Some(bg) = bg {
            let native_editor_bg = editing
                && game.editor.as_ref().is_some_and(|e| {
                    e.level.background == "assets/background/stereomadness_bw.png"
                });
            let height = if native_editor_bg {
                MENU_TILE_WIDTH * (1. / zoom).max(1.)
            } else {
                constants::VIEWPORT_HEIGHT as f32 * (1. / zoom).max(1.)
            };
            let width = textures
                .get(&sprite.image)
                .map(|i| height * i.width() as f32 / i.height() as f32)
                .unwrap_or(MENU_TILE_WIDTH);
            sprite.custom_size = Some(Vec2::new(width, height));
            transform.translation.x = camera_pos.x + bg.0 as f32 * width
                - scroll_phase(level_elapsed, crate::constants::BACKGROUND_SPEED, width);
            transform.translation.y = if native_editor_bg {
                camera_pos.y - constants::VIEWPORT_HEIGHT as f32 / (2. * zoom) + height / 2.
            } else {
                camera_pos.y
            };
            sprite.color = bg_color;
        }
        if let Some(layer) = ground {
            transform.translation.x = camera_pos.x;
            match layer {
                Floor::Base => {
                    transform.translation.y = floor - 400.;
                    sprite.color = ground_color;
                }
                Floor::Shade => {
                    transform.translation.y = floor - 400.;
                }
                Floor::Pattern => {
                    let distance = if editing {
                        camera_pos.x as f64
                    } else {
                        level_elapsed * crate::constants::SPEED
                    };
                    transform.translation.x -= distance.rem_euclid(GROUND_TILE_WIDTH as f64) as f32;
                    transform.translation.y = floor - 125.;
                }
                Floor::Line => {
                    transform.translation.y = floor - 3.;
                }
            }
        }
        if ceiling.is_some() {
            transform.translation.x = camera_pos.x;
            transform.translation.y = if show_ceiling {
                ceiling_y + 150.
            } else {
                100000.
            };
            sprite.color = ground_color;
        }
    }
}
fn update_wave_trail(
    game: Res<Game>,
    time: Res<Time<Fixed>>,
    trail: Res<Trail>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut visuals: Query<&mut Visibility, With<WaveTrailVisual>>,
) {
    let visible = matches!(
        game.screen,
        Screen::Playing | Screen::Paused | Screen::Complete
    ) && game
        .simulation
        .as_ref()
        .is_some_and(|s| s.state.player.mode == PlayerMode::Wave);
    for mut v in &mut visuals {
        *v = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !visible {
        return;
    }
    let sim = game.simulation.as_ref().unwrap();
    let p = &sim.state.player;
    let interpolate = game.screen == Screen::Playing
        && sim.status == Status::Running
        && game.attempt.playing()
        && !game.block_fixed_frame
        && game.exit_dialog.is_none();
    let alpha = if interpolate {
        time.overstep_fraction_f64()
    } else {
        1.
    };
    let head = p.previous + (p.position - p.previous) * alpha;
    let mut points: Vec<Vec2> = sim
        .wave_trail
        .iter()
        .take_while(|p| p.x < head.x - 1e-6)
        .copied()
        .map(point)
        .collect();
    points.push(point(head));
    if let Some(mut mesh) = meshes.get_mut(&trail.0) {
        *mesh = super::wave::ribbon(&points, 6. * constants::GD_SCALE as f32);
    }
}
fn animate_menu(
    game: Res<Game>,
    time: Res<Time>,
    windows: Query<&Window>,
    camera: Query<&Transform, (With<MainCamera>, Without<MenuVisual>)>,
    mut objects: Query<(&mut Transform, &mut Sprite, &MenuVisual), Without<MainCamera>>,
    mut clear: ResMut<ClearColor>,
) {
    if !matches!(game.screen, Screen::Menu | Screen::Levels | Screen::Icons) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };
    let t = time.elapsed_secs();
    let color = if game.screen == Screen::Levels {
        match game
            .catalog
            .get(game.selected)
            .map(|e| e.level.name.as_str())
        {
            Some("Back On Track") => Color::srgb(0.75, 0.03, 0.6),
            Some("Polargeist") => Color::srgb(0.08, 0.65, 0.2),
            _ => Color::srgb(0., 0., 1.),
        }
    } else {
        Color::hsl((216. + t * 0.5).rem_euclid(360.), 1., 0.5)
    };
    clear.0 = color;
    let width = constants::VIEWPORT_HEIGHT as f32 * window.width() / window.height();
    for (mut transform, mut sprite, kind) in &mut objects {
        match kind {
            MenuVisual::Tile(index) => {
                transform.translation.x = camera.translation.x + *index as f32 * MENU_TILE_WIDTH
                    - scroll_phase(
                        t as f64,
                        crate::constants::BACKGROUND_SPEED,
                        MENU_TILE_WIDTH,
                    );
                if game.screen == Screen::Levels {
                    transform.translation.x = camera.translation.x;
                }
                sprite.color = color;
            }
            MenuVisual::Floor => {
                transform.translation.x = camera.translation.x;
                sprite.color = color;
            }
            MenuVisual::Pattern => {
                sprite.color = color;
                transform.translation.x = camera.translation.x
                    - scroll_phase(
                        t as f64,
                        crate::constants::MENU_GROUND_SPEED,
                        MENU_GROUND_TILE_WIDTH,
                    );
            }
            MenuVisual::Shade => {
                transform.translation.x = camera.translation.x;
            }
            MenuVisual::Line => {
                transform.translation.x = camera.translation.x;
            }
            MenuVisual::Runner => {
                transform.translation.x = camera.translation.x - width / 2. - 100.
                    + (t as f64 * crate::constants::SPEED).rem_euclid(width as f64 + 200.) as f32;
                let (height, rotation) = menu_jump(t as f64);
                transform.translation.y = MENU_FLOOR_Y + constants::CUBE_SIZE as f32 / 2. + height;
                transform.rotation = Quat::from_rotation_z(rotation);
            }
        }
    }
}

fn scroll_phase(elapsed: f64, speed: f64, period: f32) -> f32 {
    (elapsed * speed).rem_euclid(period as f64) as f32
}
fn rgba(c: [f32; 4]) -> Color {
    Color::srgba(c[0], c[1], c[2], c[3])
}
fn point(p: Point) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}
fn draw_helpers(game: Res<Game>, mut gizmos: Gizmos, windows: Query<&Window>) {
    if game.screen == Screen::Editor
        && let Some(editor) = &game.editor
    {
        if editor.grid {
            let Ok(window) = windows.single() else {
                return;
            };
            let half_h = constants::VIEWPORT_HEIGHT / 2. / editor.zoom;
            let half_w = half_h * window.width() as f64 / window.height() as f64;
            let spacing = crate::level::GRID * if editor.zoom < 0.3 { 5. } else { 1. };
            let left = ((editor.camera.x - half_w) / spacing).floor() * spacing;
            let bottom = ((editor.camera.y - half_h) / spacing).floor() * spacing;
            let color = Color::srgba(0., 0., 0., 0.45);
            for i in 0..=((half_w * 2. / spacing).ceil() as usize + 1) {
                let x = (left + i as f64 * spacing) as f32;
                gizmos.line_2d(
                    Vec2::new(x, (editor.camera.y - half_h) as f32),
                    Vec2::new(x, (editor.camera.y + half_h) as f32),
                    color,
                );
            }
            for i in 0..=((half_h * 2. / spacing).ceil() as usize + 1) {
                let y = (bottom + i as f64 * spacing) as f32;
                gizmos.line_2d(
                    Vec2::new((editor.camera.x - half_w) as f32, y),
                    Vec2::new((editor.camera.x + half_w) as f32, y),
                    color,
                );
            }
        }
        for o in editor
            .selection()
            .into_iter()
            .filter_map(|i| editor.level.objects.get(i))
        {
            let r = o.editor_rect();
            if o.editor_marker() {
                gizmos.circle_2d(point(r.center), 47., Color::srgb(0.2, 1., 0.2));
                gizmos.line_2d(
                    point(r.center - Point::new(0., 47.)),
                    Vec2::new(r.center.x as f32, GROUND as f32),
                    Color::srgba(0.2, 1., 0.2, 0.4),
                );
            } else {
                let [a, b] = r.axes();
                let pts = [
                    r.center - a * r.half.x - b * r.half.y,
                    r.center + a * r.half.x - b * r.half.y,
                    r.center + a * r.half.x + b * r.half.y,
                    r.center - a * r.half.x + b * r.half.y,
                ];
                for i in 0..4 {
                    gizmos.line_2d(
                        point(pts[i]),
                        point(pts[(i + 1) % 4]),
                        Color::srgb(0.2, 1., 0.2),
                    );
                }
            }
        }
    }
    if matches!(
        game.screen,
        Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
    ) && let Some(sim) = &game.simulation
    {
        let p = &sim.state.player;
        if game.attempt.phase == super::attempt::Phase::Dying {
            let t = game.attempt.elapsed as f32;
            if !game.profile.settings.no_death_effect && t < 0.45 {
                let radius = 35. + t * 350.;
                let color = Color::srgba(1., 0.75, 0.15, 1. - t / 0.45);
                gizmos.circle_2d(point(p.position), radius, color);
                for i in 0..16 {
                    let a = i as f32 * std::f32::consts::TAU / 16.;
                    let dir = Vec2::new(a.cos(), a.sin());
                    gizmos.line_2d(
                        point(p.position) + dir * radius * 0.6,
                        point(p.position) + dir * radius,
                        color,
                    );
                }
            }
        }
    }
}

fn spawn_trigger(commands: &mut Commands, object: &LevelObject, art: &TriggerArt) {
    let center = object.editor_rect().center;
    let (channel, label) = if object.kind == "end_level" {
        (3, "END")
    } else {
        match object.trigger_type.as_deref() {
            Some("ground_color_change") => (1, "G"),
            Some("color_change") => (2, "C"),
            _ => (0, "BG"),
        }
    };
    for (radius, material, z) in [
        (44., art.black.clone(), 12.),
        (41., art.white.clone(), 13.),
        (38., art.fills[channel].clone(), 14.),
    ] {
        commands.spawn((
            LevelVisual,
            Mesh2d(art.circle.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(center.x as f32, center.y as f32, z)
                .with_scale(Vec3::new(radius, radius, 1.)),
        ));
    }
    commands.spawn((
        LevelVisual,
        Text2d::new(label),
        TextFont {
            font_size: FontSize::Px(27.),
            ..default()
        },
        TextColor(Color::WHITE),
        Text2dShadow::default(),
        Transform::from_xyz(center.x as f32, center.y as f32, 15.),
    ));
}
fn update_ship_rider(
    game: Res<Game>,
    images: Res<PlayerImages>,
    mut riders: Query<(&mut Sprite, &mut Visibility), With<ShipRider>>,
) {
    let ship = matches!(
        game.screen,
        Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
    ) && game
        .simulation
        .as_ref()
        .is_some_and(|s| s.state.player.mode == PlayerMode::Ship);
    for (mut sprite, mut visibility) in &mut riders {
        sprite.image = images.0[6 + game.profile.cube_icon.min(1)].clone();
        *visibility = if ship {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
fn sync_checkpoints(
    mut commands: Commands,
    game: Res<Game>,
    images: Res<PlayerImages>,
    markers: Query<Entity, With<CheckpointVisual>>,
) {
    let checkpoints = game
        .simulation
        .as_ref()
        .filter(|s| {
            s.practice
                && matches!(
                    game.screen,
                    Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
                )
        })
        .map(|s| s.checkpoints.as_slice())
        .unwrap_or(&[]);
    if markers.iter().count() == checkpoints.len() {
        return;
    }
    for entity in &markers {
        commands.entity(entity).despawn();
    }
    for cp in checkpoints {
        commands.spawn((
            LevelVisual,
            CheckpointVisual,
            Sprite {
                image: images.0[8].clone(),
                custom_size: Some(Vec2::new(80. * 72. / 135., 80.)),
                ..default()
            },
            Transform::from_xyz(
                cp.player.position.x as f32,
                cp.player.position.y as f32,
                depth::CHECKPOINT,
            ),
        ));
    }
}
fn menu_jump(elapsed: f64) -> (f32, f32) {
    let gravity = constants::quantize_velocity(constants::GRAVITY * constants::DT) / constants::DT;
    let flight = 2. * crate::constants::JUMP / gravity - crate::constants::DT;
    let period = flight + constants::MENU_JUMP_PAUSE;
    let phase = elapsed.rem_euclid(period);
    let cycle = (elapsed / period).floor();
    let height = if phase < flight {
        crate::constants::JUMP * phase - gravity * phase * (phase + crate::constants::DT) / 2.
    } else {
        0.
    };
    let air_spin = constants::CUBE_ROTATION_SPEED * flight;
    let landing_spin = (air_spin / constants::ROTATION_ALIGNMENT_STEP).round()
        * constants::ROTATION_ALIGNMENT_STEP;
    let spin = if phase < flight {
        constants::CUBE_ROTATION_SPEED * phase
    } else {
        landing_spin
            + (air_spin - landing_spin)
                * (-constants::CUBE_ROTATION_LERP_SPEED * (phase - flight)).exp()
    };
    (
        height.max(0.) as f32,
        (-(cycle * landing_spin + spin)).to_radians() as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_cube_uses_the_gameplay_jump_timing() {
        let mut sim =
            crate::simulation::Simulation::new(crate::level::Level::empty("Menu jump")).unwrap();
        let floor = sim.state.player.position.y;
        for tick in 1..=110 {
            sim.tick(crate::simulation::TickInput {
                pressed: tick == 1,
                ..default()
            });
            let (height, rotation) = menu_jump(tick as f64 * crate::constants::DT);
            assert!((height as f64 - (sim.state.player.position.y - floor)).abs() < 0.001);
            assert!(rotation.is_finite());
        }
        assert!(sim.state.player.grounded);
        assert_eq!(menu_jump(0.5).0, 0.);
    }

    #[test]
    fn mode_sprites_ship_rider_and_checkpoint_use_their_cached_art() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        let images: [Handle<Image>; 10] = std::array::from_fn(|i| {
            let (width, height) = match i {
                2 => (149, 90),
                3 => (102, 82),
                _ => (120, 120),
            };
            let mut image = Image::default();
            image.resize(bevy::render::render_resource::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            });
            app.world_mut().resource_mut::<Assets<Image>>().add(image)
        });
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.screen = Screen::Playing;
        game.simulation =
            Some(crate::simulation::Simulation::new(crate::level::Level::empty("Art")).unwrap());
        app.insert_resource(game)
            .insert_resource(PlayerImages(images.clone()))
            .insert_resource(Time::<Fixed>::from_hz(240.))
            .insert_resource(ClearColor::default())
            .insert_resource(Trail::default())
            .add_systems(
                Update,
                (animate, update_ship_rider, sync_checkpoints).chain(),
            );
        app.world_mut().spawn((
            MainCamera,
            camera_transform(),
            Projection::Orthographic(OrthographicProjection::default_2d()),
        ));
        app.world_mut().spawn(Window::default());
        let player = app
            .world_mut()
            .spawn((
                PlayerVisual,
                Sprite::default(),
                Transform::default(),
                Visibility::Visible,
            ))
            .id();
        let rider = app
            .world_mut()
            .spawn((
                ShipRider,
                Sprite::default(),
                Transform::from_xyz(0., 31., 1.),
                Visibility::Hidden,
            ))
            .id();
        app.world_mut().entity_mut(player).add_child(rider);
        for (mode, image) in [
            (PlayerMode::Ship, 2),
            (PlayerMode::Wave, 3),
            (PlayerMode::Ball, 4),
            (PlayerMode::Spider, 5),
            (PlayerMode::Cube, 0),
        ] {
            app.world_mut()
                .resource_mut::<Game>()
                .simulation
                .as_mut()
                .unwrap()
                .state
                .player
                .mode = mode;
            app.update();
            assert_eq!(
                app.world().get::<Sprite>(player).unwrap().image,
                images[image]
            );
            assert_eq!(
                *app.world().get::<Visibility>(rider).unwrap(),
                if mode == PlayerMode::Ship {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                }
            );
            assert_eq!(app.world().get::<Sprite>(rider).unwrap().image, images[6]);
            if mode == PlayerMode::Wave {
                let sprite = app.world().get::<Sprite>(player).unwrap();
                let size = sprite.custom_size.unwrap();
                assert!((size.x / size.y - 102. / 82.).abs() < 1e-6);
                let direction = app.world().get::<Transform>(player).unwrap().rotation * Vec3::X;
                assert!(
                    direction.x > 0.99,
                    "the native source sprite must point right at zero heading"
                );
            }
        }
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.profile.cube_icon = 1;
            let sim = game.simulation.as_mut().unwrap();
            sim.state.player.mode = PlayerMode::Ship;
            sim.practice = true;
            sim.checkpoint();
        }
        app.update();
        assert_eq!(app.world().get::<Sprite>(rider).unwrap().image, images[7]);
        assert_eq!(
            app.world().get::<Sprite>(player).unwrap().custom_size,
            Some(Vec2::new(149., 90.) * (constants::GD_SCALE as f32 / 4.))
        );
        let mut checkpoints = app
            .world_mut()
            .query_filtered::<(&Sprite, &Transform), With<CheckpointVisual>>();
        let (sprite, transform) = checkpoints.single(app.world()).unwrap();
        assert_eq!(sprite.image, images[8]);
        assert_eq!(sprite.custom_size, Some(Vec2::new(80. * 72. / 135., 80.)));
        assert_eq!(transform.translation.z, depth::CHECKPOINT);
        app.world_mut()
            .resource_mut::<Game>()
            .simulation
            .as_mut()
            .unwrap()
            .remove_checkpoint();
        app.update();
        assert_eq!(checkpoints.iter(app.world()).count(), 0);
    }
    #[test]
    fn presentation_system_has_disjoint_component_access() {
        let mut app = App::new();
        app.insert_resource(Game::new(std::path::PathBuf::from(env!(
            "CARGO_MANIFEST_DIR"
        ))))
        .insert_resource(Time::<Fixed>::from_hz(240.))
        .insert_resource(Time::<()>::default())
        .insert_resource(PlayerImages(Default::default()))
        .init_resource::<Assets<Image>>()
        .insert_resource(ClearColor::default())
        .insert_resource(Trail::default())
        .add_systems(Update, (animate, animate_menu).chain());
        // Initializing the real system validates its ECS query access without a GPU/window.
        app.update();
    }
    #[test]
    fn scenery_is_visible_and_scrolls_in_menu_and_gameplay() {
        let mut app = App::new();
        app.insert_resource(Game::new(std::path::PathBuf::from(env!(
            "CARGO_MANIFEST_DIR"
        ))))
        .insert_resource(Time::<Fixed>::from_hz(240.))
        .insert_resource(Time::<()>::default())
        .insert_resource(PlayerImages(Default::default()))
        .init_resource::<Assets<Image>>()
        .insert_resource(ClearColor::default())
        .insert_resource(Trail::default())
        .add_systems(Update, (animate, animate_menu).chain());
        let camera = app
            .world_mut()
            .spawn((
                MainCamera,
                camera_transform(),
                Projection::Orthographic(OrthographicProjection::default_2d()),
            ))
            .id();
        app.world_mut().spawn(Window::default());
        let menu = app
            .world_mut()
            .spawn((
                MenuVisual::Tile(0),
                Sprite::default(),
                Transform::from_xyz(0., 720., depth::BACKGROUND),
            ))
            .id();
        let background = app
            .world_mut()
            .spawn((
                Background(0),
                Sprite::default(),
                Transform::from_xyz(0., 720., depth::BACKGROUND),
            ))
            .id();
        let floor = app
            .world_mut()
            .spawn((
                Floor::Pattern,
                Sprite::default(),
                Transform::from_xyz(0., 295., depth::PATTERN),
            ))
            .id();
        app.update();
        let menu_start = app.world().get::<Transform>(menu).unwrap().translation.x;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.update();
        assert!(
            (app.world().get::<Transform>(menu).unwrap().translation.x - menu_start + 50.).abs()
                < 0.01
        );
        let projection = app.world().get::<Projection>(camera).unwrap();
        let Projection::Orthographic(projection) = projection else {
            panic!("Expected orthographic camera")
        };
        for entity in [menu, background, floor] {
            let depth = app.world().get::<Transform>(camera).unwrap().translation.z
                - app.world().get::<Transform>(entity).unwrap().translation.z;
            assert!(
                depth >= projection.near && depth <= projection.far,
                "scenery clipped at {depth}"
            );
        }
        let background_z = app
            .world()
            .get::<Transform>(background)
            .unwrap()
            .translation
            .z;
        assert!(app.world().get::<Transform>(floor).unwrap().translation.z > background_z);
        assert!(
            [depth::GROUND, depth::SHADE, depth::PATTERN, depth::LINE]
                .into_iter()
                .all(|z| z > background_z)
        );
        let mut level = crate::level::Level::empty("Scroll");
        level
            .objects
            .push(crate::level::LevelObject::new("block", 100000., 420.));
        let mut sim = crate::simulation::Simulation::new(level).unwrap();
        for _ in 0..24 {
            sim.tick(Default::default());
        }
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.simulation = Some(sim);
            game.screen = Screen::Playing;
        }
        app.update();
        let camera_start = app.world().get::<Transform>(camera).unwrap().translation.x;
        let bg_start = app
            .world()
            .get::<Transform>(background)
            .unwrap()
            .translation
            .x
            - camera_start;
        let floor_start = app.world().get::<Transform>(floor).unwrap().translation.x - camera_start;
        for _ in 0..24 {
            app.world_mut()
                .resource_mut::<Game>()
                .simulation
                .as_mut()
                .unwrap()
                .tick(Default::default());
        }
        app.update();
        assert!(
            (app.world()
                .get::<Transform>(background)
                .unwrap()
                .translation
                .x
                - app.world().get::<Transform>(camera).unwrap().translation.x
                - bg_start
                + 5.)
                .abs()
                < 0.01
        );
        assert!(
            (app.world().get::<Transform>(floor).unwrap().translation.x
                - app.world().get::<Transform>(camera).unwrap().translation.x
                - floor_start
                + (constants::SPEED * 0.1) as f32)
                .abs()
                < 0.01
        );
        app.world_mut().resource_mut::<Game>().screen = Screen::Paused;
        app.update();
        let paused = app
            .world()
            .get::<Transform>(background)
            .unwrap()
            .translation;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(5));
        app.update();
        assert_eq!(
            app.world()
                .get::<Transform>(background)
                .unwrap()
                .translation,
            paused
        );
    }
}
