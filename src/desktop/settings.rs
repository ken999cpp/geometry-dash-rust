use super::{
    Game,
    native_menu::{self as native, FontKind, NativeArt, add, button, center, image, n, text},
    ui::{Action, UiRoot},
};
use crate::storage::{FPS_LIMITS, MusicChannel, Settings};
use bevy::{
    prelude::*,
    ui::{InteractionDisabled, RelativeCursorPosition},
    window::{MonitorSelection, PrimaryWindow, WindowMode},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Page {
    #[default]
    Main,
    Options(Tab),
    Graphics,
    Advanced,
    Info(&'static str),
    OptionInfo(Tab, &'static str),
}
impl Page {
    pub fn parent(self) -> Self {
        match self {
            Self::Advanced => Self::Graphics,
            Self::OptionInfo(tab, _) => Self::Options(tab),
            _ => Self::Main,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tab {
    Gameplay,
    Visual,
    Practice,
    Editor,
    Audio,
    Other,
    Keys,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Toggle {
    Cursor,
    FlipPause,
    Fps,
    Percentage,
    Progress,
    Time,
    MenuMusic,
    Vsync,
    AutoRetry,
    FasterReset,
    LockCursor,
    EditorAutoSave,
    EditorSfx,
}
impl Toggle {
    pub fn flip(self, s: &mut Settings) {
        if matches!(self, Self::EditorSfx) {
            s.editor_sfx_volume = if s.editor_sfx_volume == 0 { 100 } else { 0 };
            return;
        }
        let v = match self {
            Self::Cursor => &mut s.show_cursor,
            Self::FlipPause => &mut s.flip_pause_button,
            Self::Fps => &mut s.show_fps,
            Self::Percentage => &mut s.show_percentage,
            Self::Progress => &mut s.show_progress_bar,
            Self::Time => &mut s.show_time,
            Self::MenuMusic => &mut s.menu_music_enabled,
            Self::Vsync => &mut s.vsync,
            Self::AutoRetry => &mut s.auto_retry,
            Self::FasterReset => &mut s.faster_reset,
            Self::LockCursor => &mut s.lock_cursor,
            Self::EditorAutoSave => &mut s.editor_auto_save,
            Self::EditorSfx => unreachable!(),
        };
        *v = !*v;
    }
}
pub(super) const RESOLUTIONS: [(u32, u32); 4] =
    [(960, 640), (1280, 720), (1600, 900), (1920, 1080)];
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct VideoOptions {
    pub fullscreen: bool,
    pub borderless: bool,
    pub resolution: usize,
}
impl From<&Settings> for VideoOptions {
    fn from(s: &Settings) -> Self {
        Self {
            fullscreen: s.fullscreen,
            borderless: s.borderless,
            resolution: s.resolution.min(RESOLUTIONS.len() - 1),
        }
    }
}
#[derive(Component)]
pub(super) struct MusicSlider(pub(super) MusicChannel);
#[derive(Component)]
pub(super) struct VolumeVisual {
    pub(super) channel: MusicChannel,
    pub(super) value: u8,
    pub(super) knob: bool,
}

// Keep UHD corner pixels at the same native size as the rest of the UI.
#[derive(Component)]
pub(super) struct NativeSlice;
pub(super) fn sliced(
    commands: &mut Commands,
    parent: Entity,
    art: &NativeArt,
    name: &str,
    node: Node,
    border: f32,
) -> Entity {
    add(
        commands,
        parent,
        (
            node,
            NativeSlice,
            ImageNode::new(art.image(name)).with_mode(NodeImageMode::Sliced(
                bevy::sprite::TextureSlicer {
                    border: bevy::sprite::BorderRect::all(border * 4.),
                    max_corner_scale: 720. / 1280.,
                    ..default()
                },
            )),
        ),
    )
}
pub(super) fn scale_slices(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: Query<&mut ImageNode, With<NativeSlice>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.height() / 1280.;
    for mut image in &mut images {
        if let NodeImageMode::Sliced(slicer) = &mut image.image_mode
            && (slicer.max_corner_scale - scale).abs() > f32::EPSILON
        {
            slicer.max_corner_scale = scale;
        }
    }
}

fn caption(
    commands: &mut Commands,
    root: Entity,
    value: &str,
    kind: FontKind,
    scale: f32,
    (x, y, w): (f32, f32, f32),
) {
    text(commands, root, value, kind, scale, center(x, y, w, 22.));
}
pub(super) fn long_button(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    label: &str,
    action: Action,
    (x, y, w): (f32, f32, f32),
    skin: &str,
) -> Entity {
    let e = add(
        commands,
        root,
        (
            Button,
            action,
            Name::new(label.to_owned()),
            super::menu::ImageButton::default(),
            UiTransform::default(),
            BackgroundColor(Color::NONE),
            center(x, y, w, 30.),
        ),
    );
    sliced(
        commands,
        e,
        art,
        skin,
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        7.,
    );
    let white_label = matches!(label, "Classic" | "Platformer" | "Normal" | "Custom");
    let kind = if skin == "editor_gray.png"
        || white_label
        || (w < 70.
            && matches!(action, Action::SettingsPage(Page::Options(tab)) if tab != Tab::Keys))
    {
        FontKind::Big
    } else {
        FontKind::Gold
    };
    text(
        commands,
        e,
        label,
        kind,
        (if skin == "editor_green.png" || white_label {
            0.8_f32
        } else if skin == "editor_gray.png" {
            0.55_f32
        } else if w < 70. && matches!(action, Action::SettingsPage(Page::Options(..))) {
            0.48_f32
        } else {
            1.0_f32
        })
        .min((w - 8.) / art.text_width(label, kind)),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
    );
    e
}
fn check(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    label: &str,
    action: Action,
    enabled: bool,
    (x, y, scale): (f32, f32, f32),
) -> Entity {
    let checkbox = button(
        commands,
        root,
        art,
        if enabled {
            "GJ_checkOn_001.png"
        } else {
            "GJ_checkOff_001.png"
        },
        action,
        center(x, y, 24., 24.),
    );
    text(
        commands,
        root,
        label,
        FontKind::Big,
        scale,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            margin: UiRect::left(n(x + 20.)),
            top: n(y - 14.),
            width: n(138.),
            height: n(28.),
            justify_content: JustifyContent::Start,
            ..default()
        },
    );
    checkbox
}
fn option(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    label: &'static str,
    action: Option<Action>,
    enabled: bool,
    context: (f32, f32, Tab),
) {
    let (x, y, tab) = context;
    let e = check(
        commands,
        root,
        art,
        label,
        action.unwrap_or(Action::SettingsBack),
        enabled,
        (x, y, 0.35),
    );
    if action.is_none() {
        commands.entity(e).insert(InteractionDisabled);
    }
    button(
        commands,
        root,
        art,
        "GJ_infoIcon_001.png",
        Action::SettingsPage(Page::OptionInfo(tab, label)),
        center(x - 17., y - 15., 12., 12.),
    );
}

fn slider(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    channel: MusicChannel,
    value: u8,
    (x, y, w): (f32, f32, f32),
) {
    let slider = add(
        commands,
        root,
        (
            Button,
            MusicSlider(channel),
            RelativeCursorPosition::default(),
            center(x, y, w, 20.),
        ),
    );
    image(
        commands,
        slider,
        art,
        "slidergroove.png",
        Node {
            position_type: PositionType::Absolute,
            left: n(-8.),
            top: n(2.),
            width: n(w + 16.),
            height: n(16.),
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
            top: n(-7.),
            margin: UiRect::left(n(-17.)),
            width: n(34.),
            height: n(34.),
            ..default()
        },
    );
    commands.entity(knob).insert(VolumeVisual {
        channel,
        value,
        knob: true,
    });
}
fn frame(commands: &mut Commands, root: Entity, art: &NativeArt) {
    add(
        commands,
        root,
        (
            center(0., 160., 370., 240.),
            BackgroundColor(Color::srgba(0., 0., 0., 0.9)),
        ),
    );
    for x in [-185., 185.] {
        image(
            commands,
            root,
            art,
            "GJ_table_side_001.png",
            center(x, 160., 29.75, 240.),
        );
        image(
            commands,
            root,
            art,
            "chain_01_001.png",
            center(x * 0.84, 7.5, 9.75, 34.75),
        );
    }
    image(
        commands,
        root,
        art,
        "GJ_table_top_001.png",
        center(0., 41., 394., 53.75),
    );
    image(
        commands,
        root,
        art,
        "GJ_table_bottom_001.png",
        center(0., 286., 389., 51.5),
    );
    caption(
        commands,
        root,
        "Settings",
        FontKind::Big,
        0.8,
        (0., 39., 200.),
    );
    button(
        commands,
        root,
        art,
        "GJ_arrow_03_001.png",
        Action::CloseSettings,
        native::left(25., 23., 32.5, 40.),
    );
}
fn popup(commands: &mut Commands, root: Entity, art: &NativeArt, brown: bool) {
    add(
        commands,
        root,
        (
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.55)),
        ),
    );
    if !brown {
        let mut panel = center(0., 160., 400., 280.);
        panel.border_radius = BorderRadius::all(n(8.));
        add(
            commands,
            root,
            (
                panel,
                BackgroundColor(Color::srgb(61. / 255., 66. / 255., 80. / 255.)),
            ),
        );
    }
    sliced(
        commands,
        root,
        art,
        if brown {
            "GJ_square_01.png"
        } else {
            "GJ_square_07.png"
        },
        center(
            0.,
            160.,
            if brown { 360. } else { 400. },
            if brown { 260. } else { 280. },
        ),
        10.,
    );
    button(
        commands,
        root,
        art,
        "GJ_closeBtn_001.png",
        Action::SettingsBack,
        center(
            if brown { -175. } else { -200. },
            if brown { 35. } else { 20. },
            if brown { 44. } else { 32. },
            if brown { 44. } else { 32. },
        ),
    );
}
pub(super) fn build(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let root = commands
        .spawn((
            UiRoot,
            GlobalZIndex(1500),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.5)),
        ))
        .id();
    frame(commands, root, art);
    let s = &game.profile.settings;
    for (label, action, x, y, w) in [
        (
            "Account",
            Action::SettingsPage(Page::Info("Account")),
            -79.,
            80.,
            145.,
        ),
        (
            "How to play",
            Action::SettingsPage(Page::Info("How to play")),
            79.,
            80.,
            145.,
        ),
        (
            "Options",
            Action::SettingsPage(Page::Options(Tab::Gameplay)),
            -79.,
            120.,
            145.,
        ),
        (
            "Graphics",
            Action::SettingsPage(Page::Graphics),
            79.,
            120.,
            145.,
        ),
        (
            "Rate",
            Action::SettingsPage(Page::Info("Rate")),
            -105.,
            160.,
            94.,
        ),
        (
            "Parent",
            Action::SettingsPage(Page::Info("Parent")),
            0.,
            160.,
            94.,
        ),
        (
            "Help",
            Action::SettingsPage(Page::Info("Help")),
            105.,
            160.,
            94.,
        ),
    ] {
        long_button(
            commands,
            root,
            art,
            label,
            action,
            (x, y, w),
            "GJ_button_01.png",
        );
    }
    caption(
        commands,
        root,
        "Music",
        FontKind::Big,
        0.7,
        (0., 195., 150.),
    );
    slider(
        commands,
        root,
        art,
        MusicChannel::Level,
        s.volume(MusicChannel::Level),
        (0., 214., 190.),
    );
    caption(commands, root, "SFX", FontKind::Big, 0.7, (0., 235., 150.));
    slider(
        commands,
        root,
        art,
        MusicChannel::Sfx,
        s.volume(MusicChannel::Sfx),
        (0., 254., 190.),
    );
    caption(
        commands,
        root,
        "Menu",
        FontKind::Big,
        0.4,
        (145., 200., 60.),
    );
    caption(
        commands,
        root,
        "Music",
        FontKind::Big,
        0.4,
        (145., 211., 60.),
    );
    button(
        commands,
        root,
        art,
        if s.menu_music_enabled {
            "GJ_checkOn_001.png"
        } else {
            "GJ_checkOff_001.png"
        },
        Action::Setting(Toggle::MenuMusic),
        center(145., 230., 24., 24.),
    );
    match game.settings_page {
        Page::Main => {}
        Page::Options(tab) => {
            popup(commands, root, art, false);
            for (i, (label, t)) in [
                ("Gameplay", Tab::Gameplay),
                ("Visual", Tab::Visual),
                ("Practice", Tab::Practice),
                ("Editor", Tab::Editor),
                ("Audio", Tab::Audio),
                ("Other", Tab::Other),
                ("Keys", Tab::Keys),
            ]
            .into_iter()
            .enumerate()
            {
                long_button(
                    commands,
                    root,
                    art,
                    label,
                    Action::SettingsPage(Page::Options(t)),
                    (-152. + i as f32 * 51., 39., 48.),
                    if t == tab {
                        "GJ_button_02.png"
                    } else {
                        "GJ_button_05.png"
                    },
                );
            }
            match tab {
                Tab::Gameplay => {
                    for (label, action, value, x, row) in [
                        (
                            "Auto-retry",
                            Some(Action::Setting(Toggle::AutoRetry)),
                            s.auto_retry,
                            -160.,
                            0,
                        ),
                        ("Click between steps", None, false, -160., 1),
                        (
                            "Lock cursor in-game",
                            Some(Action::Setting(Toggle::LockCursor)),
                            s.lock_cursor,
                            -160.,
                            2,
                        ),
                        ("Always limit\ncontrols", None, false, -160., 3),
                        ("Disable auto\ncreator controls", None, false, -160., 4),
                        (
                            "Percentage",
                            Some(Action::Setting(Toggle::Percentage)),
                            s.show_percentage,
                            -160.,
                            5,
                        ),
                        (
                            "Faster reset",
                            Some(Action::Setting(Toggle::FasterReset)),
                            s.faster_reset,
                            32.,
                            0,
                        ),
                        ("Click on steps", None, false, 32., 1),
                        ("Flip 2-player\ncontrols", None, false, 32., 2),
                        ("Disable thumbstick", None, false, 32., 3),
                        (
                            "Progress bar",
                            Some(Action::Setting(Toggle::Progress)),
                            s.show_progress_bar,
                            32.,
                            4,
                        ),
                        (
                            "Show time",
                            Some(Action::Setting(Toggle::Time)),
                            s.show_time,
                            32.,
                            5,
                        ),
                    ] {
                        option(
                            commands,
                            root,
                            art,
                            label,
                            action,
                            value,
                            (x, 85. + row as f32 * 37., tab),
                        );
                    }
                }
                Tab::Visual => {
                    for (label, action, value, x, row) in [
                        (
                            "Show cursor in-game",
                            Some(Action::Setting(Toggle::Cursor)),
                            s.show_cursor,
                            -160.,
                            0,
                        ),
                        (
                            "Flip pause button",
                            Some(Action::Setting(Toggle::FlipPause)),
                            s.flip_pause_button,
                            -160.,
                            1,
                        ),
                        ("Orb guide", None, false, -160., 2),
                        ("Disable trigger\norb scale", None, false, -160., 3),
                        (
                            "No death effect",
                            Some(Action::NoDeathEffect),
                            s.no_death_effect,
                            -160.,
                            4,
                        ),
                        ("Default mini icon", None, false, -160., 5),
                        ("Hide attempts", None, true, 32., 0),
                        ("Disable portal guide", None, false, 32., 1),
                        ("Disable orb scale", None, false, 32., 2),
                        ("Disable shake", None, false, 32., 3),
                        ("Disable gravity\neffect", None, false, 32., 4),
                        ("Switch spider\nteleport color", None, false, 32., 5),
                    ] {
                        option(
                            commands,
                            root,
                            art,
                            label,
                            action,
                            value,
                            (x, 85. + row as f32 * 37., tab),
                        );
                    }
                }
                Tab::Practice => {
                    for (label, action, value, x, row) in [
                        (
                            "Hide practice buttons",
                            Some(Action::HideCheckpointButtons),
                            s.hide_checkpoint_buttons,
                            -160.,
                            0,
                        ),
                        ("Auto-checkpoints", None, false, -160., 1),
                        ("Quick-checkpoints", None, false, -160., 2),
                        ("Skip death triggers", None, false, -160., 3),
                        ("Show hitboxes", None, false, -160., 4),
                        ("Show hitbox trail", None, false, -160., 5),
                        ("No auto-checkpoints\nfor Start Pos", None, true, 32., 1),
                        (
                            "Death effect",
                            Some(Action::NoDeathEffect),
                            !s.no_death_effect,
                            32.,
                            2,
                        ),
                        ("Normal music\nin editor", None, true, 32., 3),
                        ("Disable player hitbox", None, false, 32., 4),
                        ("Disable checkpoints", None, false, 32., 5),
                    ] {
                        option(
                            commands,
                            root,
                            art,
                            label,
                            action,
                            value,
                            (x, 85. + row as f32 * 37., tab),
                        );
                    }
                }
                Tab::Audio => {
                    for (channel, label, y) in [
                        (MusicChannel::Menu, "Menu music", 100.),
                        (MusicChannel::Level, "Music", 170.),
                        (MusicChannel::Sfx, "SFX", 240.),
                    ] {
                        caption(
                            commands,
                            root,
                            label,
                            FontKind::Big,
                            0.65,
                            (0., y - 20., 220.),
                        );
                        slider(
                            commands,
                            root,
                            art,
                            channel,
                            s.volume(channel),
                            (0., y, 240.),
                        );
                    }
                }
                Tab::Keys | Tab::Editor => {
                    for (i, line) in [
                        "Space / Mouse - Jump",
                        "P / Escape - Pause",
                        "R - Restart",
                        "E - Level editor",
                        "F5 - Test from Start Pos",
                        "M / V - Start Pos settings",
                        "Ctrl+S - Save level",
                    ]
                    .iter()
                    .enumerate()
                    {
                        caption(
                            commands,
                            root,
                            line,
                            FontKind::Big,
                            0.5,
                            (0., 83. + i as f32 * 28., 350.),
                        );
                    }
                }
                Tab::Other => {
                    caption(
                        commands,
                        root,
                        "Geometry Dash",
                        FontKind::Big,
                        0.85,
                        (0., 130., 300.),
                    );
                    caption(
                        commands,
                        root,
                        "Local levels and editor",
                        FontKind::Big,
                        0.55,
                        (0., 170., 320.),
                    );
                }
            }
        }
        Page::Graphics | Page::Advanced => {
            popup(commands, root, art, true);
            if game.settings_page == Page::Graphics {
                caption(
                    commands,
                    root,
                    "Video Options",
                    FontKind::Gold,
                    0.72,
                    (0., 47., 300.),
                );
                long_button(
                    commands,
                    root,
                    art,
                    "Advanced",
                    Action::SettingsPage(Page::Advanced),
                    (132., 49., 70.),
                    "GJ_button_04.png",
                );
                check(
                    commands,
                    root,
                    art,
                    "Fullscreen",
                    Action::VideoFullscreen,
                    game.settings_video.fullscreen,
                    (-65., 75., 0.48),
                );
                check(
                    commands,
                    root,
                    art,
                    "Borderless",
                    Action::VideoBorderless,
                    game.settings_video.borderless,
                    (-89., 112., 0.48),
                );
                caption(
                    commands,
                    root,
                    "Windowed resolution",
                    FontKind::Gold,
                    0.85,
                    (0., 145., 330.),
                );
                let (w, h) = RESOLUTIONS[game.settings_video.resolution];
                caption(
                    commands,
                    root,
                    &format!("{w}x{h}"),
                    FontKind::Big,
                    0.6,
                    (0., 170., 210.),
                );
                for (x, d) in [(-70., -1), (70., 1)] {
                    let e = button(
                        commands,
                        root,
                        art,
                        "navArrowBtn_001.png",
                        Action::VideoResolution(d),
                        center(x, 170., 15., 22.),
                    );
                    if d < 0 {
                        commands.entity(e).insert((
                            super::menu::MirroredButton,
                            UiTransform {
                                scale: Vec2::new(-1., 1.),
                                ..default()
                            },
                        ));
                    }
                }
                caption(
                    commands,
                    root,
                    "Texture quality",
                    FontKind::Gold,
                    0.85,
                    (0., 207., 330.),
                );
                caption(commands, root, "High", FontKind::Big, 0.8, (0., 231., 230.));
                long_button(
                    commands,
                    root,
                    art,
                    "Back",
                    Action::SettingsBack,
                    (-55., 267., 94.),
                    "GJ_button_01.png",
                );
                long_button(
                    commands,
                    root,
                    art,
                    "Apply",
                    Action::VideoApply,
                    (55., 267., 94.),
                    "GJ_button_01.png",
                );
            } else {
                caption(
                    commands,
                    root,
                    "Advanced video options",
                    FontKind::Big,
                    0.6,
                    (0., 47., 330.),
                );
                check(
                    commands,
                    root,
                    art,
                    "Vertical Sync",
                    Action::Setting(Toggle::Vsync),
                    s.vsync,
                    (-135., 100., 0.48),
                );
                check(
                    commands,
                    root,
                    art,
                    "Show FPS",
                    Action::Setting(Toggle::Fps),
                    s.show_fps,
                    (-135., 148., 0.48),
                );
                caption(
                    commands,
                    root,
                    "FPS",
                    FontKind::Big,
                    0.7,
                    (-105., 230., 110.),
                );
                for (i, fps) in FPS_LIMITS.into_iter().enumerate() {
                    let e = long_button(
                        commands,
                        root,
                        art,
                        &fps.to_string(),
                        Action::FpsLimit(fps),
                        (-105. + i as f32 * 70., 255., 60.),
                        if fps == game.display_rate.selected_fps(s.fps_limit()) {
                            "GJ_button_01.png"
                        } else {
                            "GJ_button_04.png"
                        },
                    );
                    if !game.display_rate.supports(fps) {
                        commands.entity(e).insert(InteractionDisabled);
                    }
                }
                let hz = game
                    .display_rate
                    .refresh_rate_millihertz
                    .map(|r| format!("Display: {:.2} Hz", r as f32 / 1000.))
                    .unwrap_or("Display: 60 FPS".into());
                caption(commands, root, &hz, FontKind::Big, 0.45, (0., 280., 300.));
            }
        }
        Page::Info(title) | Page::OptionInfo(_, title) => {
            popup(commands, root, art, true);
            caption(commands, root, title, FontKind::Gold, 1., (0., 55., 300.));
            let lines: &[&str] = match title {
                "Auto-retry" => &["Restart automatically after death"],
                "Percentage" => &["Show level completion percentage"],
                "Progress bar" => &["Show a progress bar at the top"],
                "Show time" => &["Show elapsed level time"],
                "Show cursor in-game" => &["Keep the cursor visible while playing"],
                "Flip pause button" => &["Place pause on the left side"],
                "No death effect" | "Death effect" => &["Control the death particles"],
                "Hide practice buttons" => &["Hide checkpoint buttons", "Z and X remain available"],
                "Faster reset" => &["Shorten the delay after death"],
                "Lock cursor in-game" => &["Keep the cursor inside the game window"],
                _ if matches!(game.settings_page, Page::OptionInfo(..)) => &[
                    "Options for other control modes",
                    "are unavailable in these levels",
                ],
                "How to play" => &[
                    "Hold to jump again on landing",
                    "Ship: hold to fly up",
                    "Wave: hold to move up",
                    "Ball / Spider: click to flip",
                    "P - Pause    R - Restart",
                    "E - Editor    F5 - Playtest",
                ],
                "Account" => &[
                    "Local profile",
                    "Progress is saved automatically",
                    "Online account is not connected",
                ],
                "Parent" => &["Local single-player game", "No chat or in-game purchases"],
                "Rate" => &["Thanks for playing!"],
                _ => &[
                    "Space / Mouse - Jump",
                    "P / Escape - Pause",
                    "E - Edit selected level",
                    "F5 - Test from Start Pos",
                ],
            };
            for (i, line) in lines.iter().enumerate() {
                caption(
                    commands,
                    root,
                    line,
                    FontKind::Big,
                    0.48,
                    (0., 105. + i as f32 * 25., 330.),
                );
            }
            long_button(
                commands,
                root,
                art,
                "OK",
                Action::SettingsBack,
                (0., 270., 85.),
                "GJ_button_01.png",
            );
        }
    }
}

pub(super) fn sync_window(
    game: Res<Game>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut applied: Local<Option<(bool, bool, usize, bool)>>,
) {
    let s = &game.profile.settings;
    let key = (
        s.fullscreen,
        s.borderless,
        s.resolution.min(RESOLUTIONS.len() - 1),
        s.vsync,
    );
    if *applied == Some(key) {
        return;
    }
    for mut window in &mut windows {
        window.mode = if s.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        window.decorations = !s.borderless;
        if !s.fullscreen {
            let (w, h) = RESOLUTIONS[key.2];
            window.resolution.set(w as f32, h as f32);
        }
        window.present_mode = if s.vsync {
            bevy::window::PresentMode::AutoVsync
        } else {
            bevy::window::PresentMode::AutoNoVsync
        };
        *applied = Some(key);
    }
}

pub(super) fn sliders(
    mut game: ResMut<Game>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursors: Query<(&MusicSlider, &Interaction, &RelativeCursorPosition)>,
    mut visuals: Query<(&mut VolumeVisual, &mut Node)>,
    mut dragging: Local<Option<MusicChannel>>,
) {
    let editor_pause =
        game.screen == super::Screen::Editor && game.editor.as_ref().is_some_and(|e| e.pause_open);
    if (!game.settings_open && game.screen != super::Screen::Paused && !editor_pause)
        || game.exit_dialog.is_some()
        || game.prompt.is_some()
        || game
            .editor
            .as_ref()
            .is_some_and(|e| e.files_open || e.settings_open || e.start_settings_open)
    {
        *dragging = None;
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = cursors
            .iter()
            .find(|(_, interaction, _)| **interaction == Interaction::Pressed)
            .map(|(slider, _, _)| slider.0);
    }
    if mouse.pressed(MouseButton::Left)
        && let Some(channel) = *dragging
        && let Some(position) = cursors
            .iter()
            .find(|(slider, _, _)| slider.0 == channel)
            .and_then(|(_, _, cursor)| cursor.normalized)
    {
        let volume = ((position.x + 0.5) * 100.).round().clamp(0., 100.) as u8;
        game.profile.settings.set_volume(channel, volume);
    }
    if !mouse.pressed(MouseButton::Left) && dragging.take().is_some() {
        game.persist();
    }
    for (mut visual, mut node) in &mut visuals {
        let value = game.profile.settings.volume(visual.channel);
        if visual.value != value {
            if visual.knob {
                node.left = percent(value as f32);
            } else {
                node.width = percent(value as f32);
            }
            visual.value = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn volume_slider_drags_clamps_and_persists_without_rebuilding_ui() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-slider-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = crate::storage::Profile::default();
        game.open_settings();
        game.settings_page = Page::Options(Tab::Audio);
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Startup, |mut commands: Commands, game: Res<Game>| {
                build(&mut commands, &game, &NativeArt::default())
            })
            .add_systems(Update, sliders);
        app.update();
        let slider = app
            .world_mut()
            .query::<(Entity, &MusicSlider)>()
            .iter(app.world())
            .find(|(_, s)| s.0 == MusicChannel::Menu)
            .unwrap()
            .0;
        *app.world_mut().get_mut::<Interaction>(slider).unwrap() = Interaction::Pressed;
        app.world_mut()
            .get_mut::<RelativeCursorPosition>(slider)
            .unwrap()
            .normalized = Some(Vec2::new(-0.25, 0.));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Game>()
                .profile
                .settings
                .menu_music_volume,
            25
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        for (x, expected) in [(0.8, 100), (-0.8, 0)] {
            app.world_mut()
                .get_mut::<RelativeCursorPosition>(slider)
                .unwrap()
                .normalized = Some(Vec2::new(x, 0.));
            app.update();
            assert_eq!(
                app.world()
                    .resource::<Game>()
                    .profile
                    .settings
                    .menu_music_volume,
                expected
            );
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        let saved = crate::storage::Profile::load(&root).unwrap();
        assert_eq!(saved.settings.menu_music_volume, 0);
        assert_eq!(saved.settings.level_music_volume, 100);
        assert!(app.world().get::<MusicSlider>(slider).is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
}
