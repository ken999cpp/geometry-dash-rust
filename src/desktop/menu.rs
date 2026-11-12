//! Geometry Dash menu composition. Uses the original project's button artwork.
use super::{
    Game, Screen,
    ui::{Action, UiRoot},
};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub(super) struct MenuArt(HashMap<&'static str, Handle<Image>>);
impl MenuArt {
    fn get(&self, path: &'static str) -> Handle<Image> {
        self.0.get(path).cloned().unwrap_or_default()
    }
}
#[derive(Component)]
pub(super) struct ImageButton {
    pub background: Color,
}
#[derive(Component)]
pub(super) struct MirroredButton;
impl Default for ImageButton {
    fn default() -> Self {
        Self {
            background: Color::NONE,
        }
    }
}

pub(super) fn load_art(mut commands: Commands, server: Res<AssetServer>) {
    let paths = [
        "ui/star.png",
        "menubuttons/m_m_settings_btn.png",
        "menubuttons/mm_iconsbtn.png",
        "menubuttons/mm_playbtn.png",
        "menubuttons/mm_editorbtn.png",
        "menubuttons/pm_editorbtn.png",
        "menubuttons/pm_practicebtn.png",
        "menubuttons/pm_unpracticebtn.png",
        "menubuttons/pm_playbtn.png",
        "menubuttons/pm_menubtn.png",
        "menubuttons/pm_restartbtn.png",
        "cube/Cube001.png",
        "cube/Cube002.png",
        "ship/Ship001.png",
        "wave/Wave001.png",
        "ball/Ball001.png",
        "spider/Spider001.png",
        "normal_level_complete_text.png",
        "practice_level_complete_text.png",
    ];
    commands.insert_resource(MenuArt(
        paths.into_iter().map(|p| (p, server.load(p))).collect(),
    ));
}
fn child(commands: &mut Commands, parent: Entity, bundle: impl Bundle) -> Entity {
    let entity = commands.spawn(bundle).id();
    commands.entity(parent).add_child(entity);
    entity
}
fn container(commands: &mut Commands, parent: Entity, node: Node) -> Entity {
    child(commands, parent, node)
}
fn text(
    commands: &mut Commands,
    parent: Entity,
    value: impl Into<String>,
    size: f32,
    color: Color,
) -> Entity {
    child(
        commands,
        parent,
        (
            Text::new(value),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            TextShadow {
                offset: Vec2::new(2., 3.),
                color: Color::BLACK,
            },
            TextLayout::justify(Justify::Center),
        ),
    )
}
fn image(
    commands: &mut Commands,
    parent: Entity,
    art: &MenuArt,
    path: &'static str,
    node: Node,
) -> Entity {
    child(commands, parent, (node, ImageNode::new(art.get(path))))
}
fn image_button(
    commands: &mut Commands,
    parent: Entity,
    art: &MenuArt,
    path: &'static str,
    action: Action,
    size: f32,
) -> Entity {
    child(
        commands,
        parent,
        (
            Button,
            action,
            ImageButton::default(),
            UiTransform::default(),
            BackgroundColor(Color::NONE),
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.,
                ..default()
            },
            ImageNode::new(art.get(path)),
        ),
    )
}
fn row(commands: &mut Commands, parent: Entity, top: f32, gap: f32, height: f32) -> Entity {
    container(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: percent(0),
            top: percent(top),
            width: percent(100),
            height: px(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(gap),
            ..default()
        },
    )
}
fn centered_text(
    commands: &mut Commands,
    parent: Entity,
    value: impl Into<String>,
    top: f32,
    size: f32,
    color: Color,
) {
    let node = row(commands, parent, top, 0., size * 1.5);
    text(commands, node, value, size, color);
}
fn arrow(commands: &mut Commands, parent: Entity, action: Action, left: bool, node: Node) {
    let button = child(
        commands,
        parent,
        (
            Button,
            action,
            ImageButton::default(),
            BackgroundColor(Color::NONE),
            UiTransform::default(),
            node,
        ),
    );
    for (top, angle) in [
        (24., if left { 45. } else { -45. }),
        (48., if left { -45. } else { 45. }),
    ] {
        child(
            commands,
            button,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(26),
                    top: px(top),
                    width: px(14),
                    height: px(48),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                UiTransform::from_rotation(Rot2::degrees(angle)),
            ),
        );
    }
}
fn back(commands: &mut Commands, parent: Entity) {
    arrow(
        commands,
        parent,
        Action::Menu,
        true,
        Node {
            position_type: PositionType::Absolute,
            left: px(14),
            top: px(10),
            width: px(80),
            height: px(100),
            ..default()
        },
    );
}
fn progress(
    commands: &mut Commands,
    parent: Entity,
    title: &str,
    value: u32,
    top: f32,
    color: Color,
) {
    centered_text(commands, parent, title, top, 26., Color::WHITE);
    let bar = child(
        commands,
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: percent(25),
                top: percent(top + 7.),
                width: percent(50),
                height: px(32),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(18)),
                overflow: Overflow::clip(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.55)),
            BorderColor::all(Color::BLACK),
        ),
    );
    child(
        commands,
        bar,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                height: percent(100),
                width: percent(value.min(100) as f32),
                border_radius: BorderRadius::all(px(14)),
                ..default()
            },
            BackgroundColor(color),
        ),
    );
    text(
        commands,
        bar,
        format!("{}%", value.min(100)),
        22.,
        Color::WHITE,
    );
}
fn glyph(c: char) -> [u8; 7] {
    match c {
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        _ => [0; 7],
    }
}
fn logo(commands: &mut Commands, parent: Entity) {
    let logo = container(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: percent(13),
            top: percent(19),
            width: percent(74),
            aspect_ratio: Some(11.),
            ..default()
        },
    );
    // Draw all outlines first so adjacent pixels merge into a single letter contour.
    for outline in [true, false] {
        for (i, c) in "GEOMETRY DASH".chars().enumerate() {
            for (y, bits) in glyph(c).iter().enumerate() {
                for x in 0..5 {
                    if bits & (1 << (4 - x)) == 0 {
                        continue;
                    }
                    let margin = if outline { 0.32 } else { 0. };
                    child(
                        commands,
                        logo,
                        (
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent((i as f32 * 6. + x as f32 - margin) / 77. * 100.),
                                top: percent((y as f32 - margin) / 7. * 100.),
                                width: percent((1. + 2. * margin) / 77. * 100.),
                                height: percent((1. + 2. * margin) / 7. * 100.),
                                ..default()
                            },
                            BackgroundColor(if outline {
                                Color::BLACK
                            } else if y < 3 {
                                Color::srgb(0.5, 1., 0.05)
                            } else {
                                Color::srgb(0.87, 1., 0.12)
                            }),
                        ),
                    );
                }
            }
        }
    }
}
fn difficulty(commands: &mut Commands, parent: Entity) {
    let face = child(
        commands,
        parent,
        (
            Node {
                width: px(72),
                height: px(72),
                border: UiRect::all(px(4)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.7, 1.)),
            BorderColor::all(Color::BLACK),
        ),
    );
    for left in [17., 43.] {
        child(
            commands,
            face,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(19),
                    width: px(8),
                    height: px(12),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
            ),
        );
    }
    child(
        commands,
        face,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                top: px(37),
                width: px(34),
                height: px(12),
                border: UiRect {
                    bottom: px(4),
                    ..default()
                },
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
        ),
    );
}

pub(super) fn build(commands: &mut Commands, game: &Game, art: &MenuArt) {
    let overlay = matches!(game.screen, Screen::Paused | Screen::Complete);
    let root = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(if overlay {
                Color::srgba(0., 0., 0., 0.65)
            } else {
                Color::NONE
            }),
        ))
        .id();
    if matches!(game.screen, Screen::Menu | Screen::Levels | Screen::Paused) {
        let settings = image_button(
            commands,
            root,
            art,
            "menubuttons/m_m_settings_btn.png",
            Action::Settings,
            58.,
        );
        commands.entity(settings).insert(Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(24),
            width: px(58),
            height: px(58),
            ..default()
        });
    }
    if matches!(
        game.screen,
        Screen::Menu | Screen::Levels | Screen::Icons | Screen::Paused | Screen::Complete
    ) {
        let counter = container(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(20),
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
        );
        commands.entity(counter).insert(Name::new("Star counter"));
        text(
            commands,
            counter,
            game.profile.total_stars().to_string(),
            30.,
            Color::srgb(1., 0.9, 0.2),
        );
        image(
            commands,
            counter,
            art,
            "ui/star.png",
            Node {
                width: px(38),
                height: px(38),
                ..default()
            },
        );
    }
    match game.screen {
        Screen::Menu => {
            logo(commands, root);
            let buttons = row(commands, root, 43., 52., 210.);
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/mm_iconsbtn.png",
                Action::Icons,
                145.,
            );
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/pm_playbtn.png",
                Action::Levels,
                215.,
            );
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/mm_editorbtn.png",
                Action::New,
                145.,
            );
        }
        Screen::Levels => {
            back(commands, root);
            if let Some(entry) = game.catalog.get(game.selected) {
                let p = game
                    .profile
                    .levels
                    .get(&entry.level.name)
                    .cloned()
                    .unwrap_or_default();
                let title = child(
                    commands,
                    root,
                    (
                        Button,
                        Action::Play,
                        ImageButton {
                            background: Color::srgba(0., 0., 0., 0.2),
                        },
                        UiTransform::default(),
                        BackgroundColor(Color::srgba(0., 0., 0., 0.2)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(18),
                            top: percent(20),
                            width: percent(64),
                            height: percent(22),
                            border_radius: BorderRadius::all(px(16)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: px(26),
                            ..default()
                        },
                    ),
                );
                difficulty(commands, title);
                text(commands, title, &entry.level.name, 42., Color::WHITE);
                let stars = entry.level.stars();
                if stars > 0 {
                    let rating = container(
                        commands,
                        title,
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(6),
                            ..default()
                        },
                    );
                    text(
                        commands,
                        rating,
                        stars.to_string(),
                        26.,
                        Color::srgb(1., 0.9, 0.2),
                    );
                    image(
                        commands,
                        rating,
                        art,
                        "ui/star.png",
                        Node {
                            width: px(36),
                            height: px(36),
                            ..default()
                        },
                    );
                } else {
                    text(commands, title, "CUSTOM", 20., Color::srgb(1., 0.9, 0.2));
                }
                progress(
                    commands,
                    root,
                    "NORMAL MODE",
                    p.normal,
                    47.,
                    Color::srgb(0.35, 0.95, 0.05),
                );
                progress(
                    commands,
                    root,
                    "PRACTICE MODE",
                    p.practice,
                    64.,
                    Color::srgb(0., 0.9, 1.),
                );
                centered_text(
                    commands,
                    root,
                    format!("{} attempts", p.attempts),
                    80.,
                    18.,
                    Color::WHITE,
                );
                arrow(
                    commands,
                    root,
                    Action::PreviousLevel,
                    true,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(4),
                        top: percent(35),
                        width: px(80),
                        height: px(110),
                        ..default()
                    },
                );
                arrow(
                    commands,
                    root,
                    Action::NextLevel,
                    false,
                    Node {
                        position_type: PositionType::Absolute,
                        right: percent(4),
                        top: percent(35),
                        width: px(80),
                        height: px(110),
                        ..default()
                    },
                );
                let edit = container(
                    commands,
                    root,
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(25),
                        bottom: px(25),
                        ..default()
                    },
                );
                image_button(
                    commands,
                    edit,
                    art,
                    "menubuttons/pm_editorbtn.png",
                    Action::Edit,
                    62.,
                );
                let dots = row(commands, root, 90., 12., 16.);
                for i in 0..game.catalog.len() {
                    child(
                        commands,
                        dots,
                        (
                            Button,
                            Action::Select(i),
                            ImageButton {
                                background: if i == game.selected {
                                    Color::WHITE
                                } else {
                                    Color::srgba(1., 1., 1., 0.35)
                                },
                            },
                            Node {
                                width: px(12),
                                height: px(12),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(if i == game.selected {
                                Color::WHITE
                            } else {
                                Color::srgba(1., 1., 1., 0.35)
                            }),
                        ),
                    );
                }
            } else {
                centered_text(commands, root, "NO LEVELS", 40., 40., Color::WHITE);
            }
        }
        Screen::Paused | Screen::Complete => {
            let practice = game.simulation.as_ref().is_some_and(|s| s.practice);
            let name = game
                .simulation
                .as_ref()
                .map(|s| s.level.name.as_str())
                .unwrap_or("");
            centered_text(commands, root, name, 9., 42., Color::WHITE);
            if game.screen == Screen::Complete {
                image(
                    commands,
                    root,
                    art,
                    if practice {
                        "practice_level_complete_text.png"
                    } else {
                        "normal_level_complete_text.png"
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(19),
                        top: percent(24),
                        width: percent(62),
                        height: percent(12),
                        ..default()
                    },
                );
            } else if practice {
                centered_text(
                    commands,
                    root,
                    "PRACTICE MODE",
                    19.,
                    20.,
                    Color::srgb(0., 1., 0.9),
                );
            }
            let p = game.profile.levels.get(name).cloned().unwrap_or_default();
            progress(
                commands,
                root,
                "NORMAL MODE",
                p.normal,
                36.,
                Color::srgb(0.35, 0.95, 0.05),
            );
            progress(
                commands,
                root,
                "PRACTICE MODE",
                p.practice,
                52.,
                Color::srgb(0., 0.9, 1.),
            );
            let buttons = row(commands, root, 73., 25., 125.);
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/pm_editorbtn.png",
                Action::Edit,
                90.,
            );
            image_button(
                commands,
                buttons,
                art,
                if practice {
                    "menubuttons/pm_unpracticebtn.png"
                } else {
                    "menubuttons/pm_practicebtn.png"
                },
                Action::Practice,
                90.,
            );
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/pm_playbtn.png",
                if game.screen == Screen::Complete {
                    Action::Restart
                } else {
                    Action::Resume
                },
                125.,
            );
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/pm_menubtn.png",
                Action::BackLevels,
                90.,
            );
            image_button(
                commands,
                buttons,
                art,
                "menubuttons/pm_restartbtn.png",
                Action::Restart,
                90.,
            );
        }
        Screen::Icons => {
            back(commands, root);
            centered_text(commands, root, "ICON KIT", 12., 42., Color::WHITE);
            let preview = row(commands, root, 28., 0., 100.);
            image(
                commands,
                preview,
                art,
                if game.profile.cube_icon == 1 {
                    "cube/Cube002.png"
                } else {
                    "cube/Cube001.png"
                },
                Node {
                    width: px(100),
                    height: px(100),
                    ..default()
                },
            );
            let choices = row(commands, root, 52., 30., 90.);
            for (i, path) in ["cube/Cube001.png", "cube/Cube002.png"]
                .into_iter()
                .enumerate()
            {
                let choice = image_button(commands, choices, art, path, Action::Icon(i), 70.);
                if game.profile.cube_icon == i {
                    commands.entity(choice).insert((
                        BorderColor::all(Color::srgb(0.5, 1., 0.1)),
                        Node {
                            width: px(82),
                            height: px(82),
                            padding: UiRect::all(px(5)),
                            border: UiRect::all(px(3)),
                            ..default()
                        },
                    ));
                }
            }
            let modes = row(commands, root, 75., 35., 50.);
            image(
                commands,
                modes,
                art,
                "ship/Ship001.png",
                Node {
                    width: px(60),
                    height: px(50),
                    ..default()
                },
            );
            image(
                commands,
                modes,
                art,
                "wave/Wave001.png",
                Node {
                    width: px(45),
                    height: px(50),
                    ..default()
                },
            );
            for path in ["ball/Ball001.png", "spider/Spider001.png"] {
                image(
                    commands,
                    modes,
                    art,
                    path,
                    Node {
                        width: px(50),
                        height: px(50),
                        ..default()
                    },
                );
            }
        }
        Screen::Loading => centered_text(commands, root, "LOADING…", 42., 40., Color::WHITE),
        _ => {}
    }
    if !game.status.is_empty() {
        centered_text(commands, root, &game.status, 96., 14., Color::WHITE);
    }
}
