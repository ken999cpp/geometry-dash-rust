//! UI in GD's 320-point viewport, using the installed reference's sprite frames
//! and bitmap-font metrics. `vh` keeps artwork proportional at every resolution.
use super::{
    Game, Screen,
    menu::ImageButton,
    ui::{Action, UiRoot},
};
use bevy::{prelude::*, ui::RelativeCursorPosition};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Copy, Default)]
pub(super) enum FontKind {
    #[default]
    Big,
    Gold,
}
#[derive(Deserialize)]
struct Glyph {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    xoffset: f32,
    yoffset: f32,
    xadvance: f32,
}
#[derive(Deserialize)]
struct BitmapFont {
    glyphs: HashMap<u32, Glyph>,
    kerning: HashMap<String, f32>,
}
#[derive(Resource)]
pub(super) struct NativeArt {
    images: HashMap<String, Handle<Image>>,
    sizes: HashMap<String, [f32; 2]>,
    fonts: HashMap<String, BitmapFont>,
}
impl Default for NativeArt {
    fn default() -> Self {
        Self {
            images: HashMap::new(),
            sizes: serde_json::from_str(include_str!("../../assets/native_ui/sprites.json"))
                .expect("sprite manifest"),
            fonts: serde_json::from_str(include_str!("../../assets/native_ui/fonts.json"))
                .expect("bitmap fonts"),
        }
    }
}
impl NativeArt {
    fn image(&self, name: &str) -> Handle<Image> {
        self.images.get(name).cloned().unwrap_or_default()
    }
    fn font(&self, kind: FontKind) -> &BitmapFont {
        &self.fonts[match kind {
            FontKind::Big => "bigFont",
            FontKind::Gold => "goldFont",
        }]
    }
}
pub(super) fn load(mut commands: Commands, server: Res<AssetServer>) {
    let mut art = NativeArt::default();
    for name in art.sizes.keys().map(String::as_str).chain([
        "bigFont.png",
        "goldFont.png",
        "slidergroove.png",
        "sliderthumb.png",
        "sliderBar.png",
    ]) {
        art.images
            .insert(name.into(), server.load(format!("native_ui/{name}")));
    }
    commands.insert_resource(art);
}
pub(super) fn n(value: f32) -> Val {
    vh(value / 3.2)
}
fn add(commands: &mut Commands, parent: Entity, bundle: impl Bundle) -> Entity {
    let e = commands.spawn(bundle).id();
    commands.entity(parent).add_child(e);
    e
}
pub(super) fn center(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: n(y - h / 2.),
        margin: UiRect::left(n(x - w / 2.)),
        width: n(w),
        height: n(h),
        ..default()
    }
}
fn left(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: n(x - w / 2.),
        top: n(y - h / 2.),
        width: n(w),
        height: n(h),
        ..default()
    }
}
fn right(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: n(x - w / 2.),
        top: n(y - h / 2.),
        width: n(w),
        height: n(h),
        ..default()
    }
}
fn image(
    commands: &mut Commands,
    parent: Entity,
    art: &NativeArt,
    name: &str,
    node: Node,
) -> Entity {
    add(commands, parent, (node, ImageNode::new(art.image(name))))
}
fn button(
    commands: &mut Commands,
    parent: Entity,
    art: &NativeArt,
    name: &str,
    action: Action,
    node: Node,
) -> Entity {
    add(
        commands,
        parent,
        (
            Button,
            action,
            Name::new(name.to_owned()),
            ImageButton::default(),
            UiTransform::default(),
            BackgroundColor(Color::NONE),
            node,
            ImageNode::new(art.image(name)),
        ),
    )
}
#[derive(Component)]
pub(super) struct BitmapText {
    pub value: String,
    kind: FontKind,
    scale: f32,
}
pub(super) fn text(
    commands: &mut Commands,
    parent: Entity,
    value: impl Into<String>,
    kind: FontKind,
    scale: f32,
    mut node: Node,
) -> Entity {
    node.justify_content = JustifyContent::Center;
    node.align_items = AlignItems::Center;
    let wrapper = add(commands, parent, node);
    add(
        commands,
        wrapper,
        (
            BitmapText {
                value: value.into(),
                kind,
                scale,
            },
            Node::default(),
        ),
    )
}
pub(super) fn refresh_text(
    mut commands: Commands,
    art: Res<NativeArt>,
    mut labels: Query<(Entity, &BitmapText, &mut Node), Changed<BitmapText>>,
) {
    for (entity, label, mut node) in &mut labels {
        commands.entity(entity).despawn_children();
        let font = art.font(label.kind);
        let mut pen = 0.;
        let mut previous = 0;
        let mut glyphs = Vec::new();
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for ch in label.value.chars() {
            let id = ch as u32;
            let Some(g) = font.glyphs.get(&id).or_else(|| font.glyphs.get(&63)) else {
                continue;
            };
            pen += font
                .kerning
                .get(&format!("{previous}:{id}"))
                .copied()
                .unwrap_or(0.);
            if g.width > 0. && g.height > 0. {
                let x = pen + g.xoffset;
                let y = g.yoffset;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x + g.width);
                max_y = max_y.max(y + g.height);
                glyphs.push((g, x, y));
            }
            pen += g.xadvance;
            previous = id;
        }
        if glyphs.is_empty() {
            node.width = n(0.);
            node.height = n(0.);
            continue;
        }
        let scale = label.scale / 4.;
        node.width = n((max_x - min_x) * scale);
        node.height = n((max_y - min_y) * scale);
        let atlas = art.image(match label.kind {
            FontKind::Big => "bigFont.png",
            FontKind::Gold => "goldFont.png",
        });
        for (g, x, y) in glyphs {
            add(
                &mut commands,
                entity,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: n((x - min_x) * scale),
                        top: n((y - min_y) * scale),
                        width: n(g.width * scale),
                        height: n(g.height * scale),
                        ..default()
                    },
                    ImageNode {
                        image: atlas.clone(),
                        rect: Some(Rect::new(g.x, g.y, g.x + g.width, g.y + g.height)),
                        ..default()
                    },
                ),
            );
        }
    }
}
fn caption(
    commands: &mut Commands,
    root: Entity,
    value: impl Into<String>,
    y: f32,
    scale: f32,
) -> Entity {
    text(
        commands,
        root,
        value,
        FontKind::Big,
        scale,
        center(0., y, 360., 35.),
    )
}
fn coins(commands: &mut Commands, root: Entity, art: &NativeArt, y: f32, small: bool) {
    let size = if small { 24. } else { 26. };
    for dx in [-45., 0., 45.] {
        let e = image(
            commands,
            root,
            art,
            "gjCoinUI_01_off_001.png",
            center(dx, y, size, size),
        );
        commands.entity(e).insert(ImageNode {
            image: art.image("gjCoinUI_01_off_001.png"),
            color: Color::WHITE,
            ..default()
        });
    }
}
fn progress(
    commands: &mut Commands,
    root: Entity,
    value: u32,
    y: f32,
    color: Color,
    best: Option<u32>,
) {
    let bar = add(
        commands,
        root,
        (
            center(0., y, 340., 20.),
            BackgroundColor(Color::srgba(0., 0., 0., 0.6)),
        ),
    );
    commands.entity(bar).insert(Node {
        border_radius: BorderRadius::MAX,
        overflow: Overflow::clip(),
        ..center(0., y, 340., 20.)
    });
    if let Some(best) = best.filter(|&best| best > value) {
        add(
            commands,
            bar,
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(best.min(100) as f32),
                    height: percent(100),
                    border_radius: BorderRadius::all(n(10.)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
            ),
        );
    }
    add(
        commands,
        bar,
        (
            Node {
                position_type: PositionType::Absolute,
                width: percent(value.min(100) as f32),
                height: percent(100),
                border_radius: BorderRadius::all(n(10.)),
                ..default()
            },
            BackgroundColor(color),
        ),
    );
    text(
        commands,
        bar,
        format!("{}%", value.min(100)),
        FontKind::Big,
        0.5,
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
    );
}
fn decoration(commands: &mut Commands, root: Entity, art: &NativeArt) {
    image(
        commands,
        root,
        art,
        "GJ_topBar_001.png",
        center(0., 11., 306.5, 36.),
    );
    for flip in [false, true] {
        let e = image(
            commands,
            root,
            art,
            "GJ_sideArt_001.png",
            Node {
                position_type: PositionType::Absolute,
                left: if flip { Val::Auto } else { n(0.) },
                right: if flip { n(0.) } else { Val::Auto },
                bottom: n(0.),
                width: n(71.5),
                height: n(70.5),
                ..default()
            },
        );
        if flip {
            commands
                .entity(e)
                .insert(UiTransform::from_scale(Vec2::new(-1., 1.)));
        }
    }
}
pub(super) fn build(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let root = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(if game.screen == Screen::Paused {
                Color::srgba(0., 0., 0., 0.6)
            } else {
                Color::NONE
            }),
        ))
        .id();
    match game.screen {
        Screen::Menu => {
            image(
                commands,
                root,
                art,
                "GJ_logo_001.png",
                center(0., 50., 429., 52.),
            );
            button(
                commands,
                root,
                art,
                "GJ_closeBtn_001.png",
                Action::ExitApplication,
                left(18., 18., 32.2, 33.25),
            );
            for (name, action, dx, w, h) in [
                ("GJ_garageBtn_001.png", Action::Icons, -110., 70., 70.),
                ("GJ_playBtn_001.png", Action::Levels, 0., 109.5, 109.5),
                ("GJ_creatorBtn_001.png", Action::New, 110., 70., 70.),
            ] {
                button(commands, root, art, name, action, center(dx, 150., w, h));
            }
            for (name, action, dx) in [
                ("GJ_achBtn_001.png", Action::Summary("Achievements"), -84.),
                ("GJ_optionsBtn_001.png", Action::Settings, -28.),
                ("GJ_statsBtn_001.png", Action::Summary("Stats"), 28.),
                (
                    "GJ_ngBtn_001.png",
                    Action::External("https://www.newgrounds.com"),
                    84.,
                ),
            ] {
                button(
                    commands,
                    root,
                    art,
                    name,
                    action,
                    center(dx, 275., 48., 51.),
                );
            }
            for (name, url, x, y, w, h) in [
                (
                    "gj_fbIcon_001.png",
                    "https://www.facebook.com/geometrydash",
                    22.,
                    265.,
                    24.,
                    24.,
                ),
                (
                    "gj_twIcon_001.png",
                    "https://twitter.com/RobTopGames",
                    51.,
                    265.,
                    24.,
                    24.,
                ),
                (
                    "gj_ytIcon_001.png",
                    "https://www.youtube.com/user/RobTopGames",
                    80.,
                    265.,
                    24.,
                    24.,
                ),
                (
                    "gj_twitchIcon_001.png",
                    "https://www.twitch.tv/robtopgames",
                    109.,
                    265.,
                    24.,
                    24.,
                ),
                (
                    "gj_discordIcon_001.png",
                    "https://discord.com/invite/geometrydash",
                    109.,
                    294.,
                    24.,
                    24.,
                ),
                (
                    "robtoplogo_small.png",
                    "https://www.robtopgames.com",
                    50.,
                    294.,
                    80.,
                    22.4,
                ),
            ] {
                button(
                    commands,
                    root,
                    art,
                    name,
                    Action::External(url),
                    left(x, y, w, h),
                );
            }
            button(
                commands,
                root,
                art,
                "GJ_dailyRewardBtn_001.png",
                Action::Summary("Daily rewards"),
                right(40., 143., 48.5, 46.),
            );
            button(
                commands,
                root,
                art,
                "GJ_newsBtn_001.png",
                Action::External("https://www.robtopgames.com"),
                right(40., 270., 65., 55.8),
            );
        }
        Screen::Levels => {
            decoration(commands, root, art);
            button(
                commands,
                root,
                art,
                "GJ_arrow_01_001.png",
                Action::Menu,
                left(25., 23., 32.5, 40.5),
            );
            button(
                commands,
                root,
                art,
                "GJ_infoIcon_001.png",
                Action::Summary("Level info"),
                right(20., 20., 23.25, 23.),
            );
            button(
                commands,
                root,
                art,
                "sideBtn_death_001.png",
                Action::Summary("Stats"),
                right(20., 60., 27.6, 28.5),
            );
            if let Some(entry) = game.catalog.get(game.selected) {
                let p = game
                    .profile
                    .levels
                    .get(&entry.level.name)
                    .cloned()
                    .unwrap_or_default();
                let title = add(
                    commands,
                    root,
                    (
                        Button,
                        Action::Play,
                        ImageButton {
                            background: Color::srgba(0., 0., 0., 0.5),
                        },
                        BackgroundColor(Color::srgba(0., 0., 0., 0.5)),
                        Node {
                            border_radius: BorderRadius::all(n(10.)),
                            ..center(0., 100., 340., 95.)
                        },
                    ),
                );
                let difficulty = match entry.level.stars() {
                    1 => "diffIcon_01_btn_001.png",
                    2 => "diffIcon_02_btn_001.png",
                    _ => "diffIcon_03_btn_001.png",
                };
                image(commands, title, art, difficulty, left(35., 45., 35., 35.));
                text(
                    commands,
                    title,
                    &entry.level.name,
                    FontKind::Big,
                    0.9,
                    Node {
                        position_type: PositionType::Absolute,
                        left: n(62.),
                        right: n(15.),
                        top: n(26.),
                        height: n(40.),
                        ..default()
                    },
                );
                text(
                    commands,
                    title,
                    entry.level.stars().to_string(),
                    FontKind::Big,
                    0.5,
                    right(32., 13., 24., 20.),
                );
                image(
                    commands,
                    title,
                    art,
                    "GJ_starsIcon_001.png",
                    right(15., 13., 16.5, 17.),
                );
                for i in 0..3 {
                    let e = image(
                        commands,
                        title,
                        art,
                        "gjCoinUI_01_off_001.png",
                        right(16. + i as f32 * 26., 80., 24., 24.),
                    );
                    commands.entity(e).insert(ImageNode {
                        image: art.image("gjCoinUI_01_off_001.png"),
                        color: Color::WHITE,
                        ..default()
                    });
                }
                text(
                    commands,
                    title,
                    "0/50",
                    FontKind::Big,
                    0.4,
                    left(22., 84., 40., 20.),
                );
                image(
                    commands,
                    title,
                    art,
                    "currencyOrbIcon_001.png",
                    left(49., 84., 15., 13.9),
                );
                caption(commands, root, "Normal Mode", 171., 0.55);
                progress(
                    commands,
                    root,
                    p.normal,
                    190.,
                    Color::srgb(0., 1., 0.),
                    None,
                );
                caption(commands, root, "Practice Mode", 221., 0.55);
                progress(
                    commands,
                    root,
                    p.practice,
                    240.,
                    Color::srgb(0., 0.9, 1.),
                    None,
                );
                caption(commands, root, "Download the Soundtrack", 286., 0.5);
                let previous = button(
                    commands,
                    root,
                    art,
                    "navArrowBtn_001.png",
                    Action::PreviousLevel,
                    left(25., 160., 27.5, 54.),
                );
                commands
                    .entity(previous)
                    .insert(super::menu::MirroredButton);
                button(
                    commands,
                    root,
                    art,
                    "navArrowBtn_001.png",
                    Action::NextLevel,
                    right(25., 160., 27.5, 54.),
                );
                let dots = add(
                    commands,
                    root,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        bottom: n(10.),
                        width: percent(100),
                        height: n(10.),
                        justify_content: JustifyContent::Center,
                        column_gap: n(8.),
                        ..default()
                    },
                );
                for i in 0..game.catalog.len() {
                    add(
                        commands,
                        dots,
                        (
                            Button,
                            Action::Select(i),
                            ImageButton {
                                background: if i == game.selected {
                                    Color::WHITE
                                } else {
                                    Color::srgb(0.5, 0.5, 0.5)
                                },
                            },
                            Node {
                                width: n(8.),
                                height: n(8.),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(if i == game.selected {
                                Color::WHITE
                            } else {
                                Color::srgb(0.5, 0.5, 0.5)
                            }),
                        ),
                    );
                }
            }
        }
        Screen::Paused => {
            let panel = add(
                commands,
                root,
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: n(10.),
                        right: n(10.),
                        top: n(10.),
                        bottom: n(10.),
                        border_radius: BorderRadius::all(n(30.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0., 0., 0., 0.95)),
                ),
            );
            let Some(sim) = &game.simulation else {
                return;
            };
            let p = game
                .profile
                .levels
                .get(&sim.level.name)
                .cloned()
                .unwrap_or_default();
            caption(commands, panel, &sim.level.name, 22., 1.0);
            text(
                commands,
                panel,
                format!(
                    "Best: {}%",
                    if sim.practice { p.practice } else { p.normal }
                ),
                FontKind::Gold,
                0.5,
                center(0., 57., 300., 24.),
            );
            caption(
                commands,
                panel,
                if sim.practice {
                    "Practice Mode"
                } else {
                    "Normal Mode"
                },
                75.,
                0.55,
            );
            progress(
                commands,
                panel,
                sim.progress(),
                95.,
                if sim.practice {
                    Color::srgb(0., 0.9, 1.)
                } else {
                    Color::srgb(0., 1., 0.)
                },
                Some(if sim.practice { p.practice } else { p.normal }),
            );
            let entries = [
                (
                    "GJ_practiceBtn_001.png",
                    Action::Practice,
                    -123.,
                    61.5,
                    64.5,
                ),
                ("GJ_playBtn2_001.png", Action::Resume, -38., 78.5, 81.5),
                ("GJ_menuBtn_001.png", Action::BackLevels, 47., 61.5, 64.5),
                ("GJ_replayBtn_001.png", Action::Restart, 123., 61.5, 64.5),
            ];
            for (mut name, action, dx, w, h) in entries {
                if matches!(action, Action::Practice) && sim.practice {
                    name = "GJ_normalBtn_001.png";
                }
                button(commands, panel, art, name, action, center(dx, 180., w, h));
            }
            for (name, action, y) in [
                ("sideBtn_death_001.png", Action::Summary("Stats"), 26.),
                ("GJ_deleteBtn_001.png", Action::BackLevels, 60.),
                (
                    if game.testing {
                        "GJ_editBtn_001.png"
                    } else {
                        "sideBtn_quests_001.png"
                    },
                    if game.testing {
                        Action::Edit
                    } else {
                        Action::Summary("Level info")
                    },
                    94.,
                ),
            ] {
                button(commands, panel, art, name, action, left(26., y, 27.6, 28.5));
            }
            button(
                commands,
                panel,
                art,
                "GJ_optionsBtn_001.png",
                Action::Settings,
                right(26., 26., 30., 32.),
            );
            button(
                commands,
                panel,
                art,
                "sideBtn_hide_001.png",
                Action::HidePause,
                right(26., 60., 27.6, 28.5),
            );
            pause_slider(
                commands,
                panel,
                art,
                crate::storage::MusicChannel::Level,
                game.profile.settings.level_music_volume,
                -99.,
            );
            pause_slider(
                commands,
                panel,
                art,
                crate::storage::MusicChannel::Sfx,
                game.profile.settings.sfx_volume,
                101.,
            );
            coins(commands, panel, art, 277., true);
        }
        _ => {}
    }
}

fn pause_slider(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    channel: crate::storage::MusicChannel,
    value: u8,
    x: f32,
) {
    image(
        commands,
        root,
        art,
        if channel == crate::storage::MusicChannel::Sfx {
            "GJ_sfxIcon_001.png"
        } else {
            "GJ_musicIcon_001.png"
        },
        center(x - 90., 240., 17., 17.),
    );
    let slider = add(
        commands,
        root,
        (
            Button,
            super::settings::MusicSlider(channel),
            RelativeCursorPosition::default(),
            center(x, 240., 145., 16.),
        ),
    );
    image(
        commands,
        slider,
        art,
        "slidergroove.png",
        Node {
            position_type: PositionType::Absolute,
            left: n(-12.),
            width: n(169.),
            height: n(12.),
            top: n(2.),
            ..default()
        },
    );
    let knob = image(
        commands,
        slider,
        art,
        "sliderthumb.png",
        Node {
            position_type: PositionType::Absolute,
            left: percent(value as f32),
            margin: UiRect::left(n(-12.)),
            top: n(-4.),
            width: n(24.),
            height: n(24.),
            ..default()
        },
    );
    commands.entity(knob).insert(super::settings::VolumeVisual {
        channel,
        value,
        knob: true,
    });
}

#[derive(Component)]
pub(super) struct HudProgress;
#[derive(Component)]
pub(super) struct HudFps;
pub(super) fn build_hud(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let root = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .id();
    let label = text(
        commands,
        root,
        "0%",
        FontKind::Big,
        0.5,
        center(0., 9., 100., 20.),
    );
    commands.entity(label).insert(HudProgress);
    let fps = text(
        commands,
        root,
        "",
        FontKind::Gold,
        0.3,
        left(25., 8., 50., 15.),
    );
    commands.entity(fps).insert(HudFps);
    button(
        commands,
        root,
        art,
        "GJ_pauseBtn_001.png",
        Action::Pause,
        right(12., 12., 22.8, 23.7),
    );
    if game.simulation.as_ref().is_some_and(|s| s.practice)
        && !game.profile.settings.hide_checkpoint_buttons
    {
        button(
            commands,
            root,
            art,
            "GJ_checkpointBtn_001.png",
            Action::AddCheckpoint,
            Node {
                position_type: PositionType::Absolute,
                left: n(8.),
                bottom: n(8.),
                width: n(44.),
                height: n(29.),
                ..default()
            },
        );
        button(
            commands,
            root,
            art,
            "GJ_removeCheckBtn_001.png",
            Action::RemoveCheckpoint,
            Node {
                position_type: PositionType::Absolute,
                left: n(58.),
                bottom: n(8.),
                width: n(28.),
                height: n(29.),
                ..default()
            },
        );
    }
}

pub(super) fn summary(commands: &mut Commands, game: &Game, art: &NativeArt, title: &str) {
    let overlay = commands
        .spawn((
            UiRoot,
            GlobalZIndex(1400),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
        ))
        .id();
    let panel = add(
        commands,
        overlay,
        (
            Node {
                border: UiRect::all(n(2.)),
                border_radius: BorderRadius::all(n(12.)),
                ..center(0., 160., 350., 220.)
            },
            BackgroundColor(Color::srgb(0.12, 0.25, 0.48)),
            BorderColor::all(Color::WHITE),
        ),
    );
    caption(commands, panel, title, 25., 0.8);
    let attempts: u32 = game.profile.levels.values().map(|p| p.attempts).sum();
    let completed = game
        .profile
        .levels
        .values()
        .filter(|p| p.normal == 100)
        .count();
    let lines = match title {
        "Stats" | "Achievements" => vec![
            format!("Stars: {}", game.profile.total_stars()),
            format!("Completed levels: {completed}"),
            format!("Total attempts: {attempts}"),
        ],
        "Level info" => game
            .catalog
            .get(game.selected)
            .map(|e| {
                vec![
                    e.level.name.clone(),
                    format!("Objects: {}", e.level.objects.len()),
                    "Press E to open the editor".into(),
                ]
            })
            .unwrap_or_default(),
        "Music" => vec![
            "Music Library".into(),
            "Stereo Madness - ForeverBound".into(),
            "Back On Track - DJVI".into(),
            "Polargeist - Step".into(),
        ],
        _ => vec!["Local game".into(), "Online rewards are unavailable".into()],
    };
    for (i, line) in lines.iter().enumerate() {
        caption(commands, panel, line, 70. + i as f32 * 30., 0.45);
    }
    button(
        commands,
        panel,
        art,
        "GJ_closeBtn_001.png",
        Action::CloseSummary,
        right(0., 0., 32.2, 33.25),
    );
}
pub(super) fn open_link(url: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .creation_flags(0x08000000)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}
