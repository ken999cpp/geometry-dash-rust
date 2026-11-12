use super::{
    Game,
    editor_icons::{self, Icon},
    menu::ImageButton,
    ui::{Action, UiRoot},
};
use crate::editor::EditorMode;
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) const PANEL_HEIGHT: f32 = 210.;
#[derive(Resource, Default)]
pub(super) struct EditorArt(HashMap<String, Handle<Image>>);
#[derive(Component)]
pub(super) struct Tooltip(String);
#[derive(Component)]
pub(super) struct TooltipText;

pub(super) fn load_art(mut commands: Commands, server: Res<AssetServer>) {
    let mut art = HashMap::new();
    for n in 1..=11 {
        art.insert(
            format!("block{n}"),
            server.load(format!("blocks/block{n}.png")),
        );
    }
    for (key, path) in [
        ("platform", "platforms/platform.png"),
        ("spike", "spikes/spike.png"),
        ("minispike", "spikes/minispike.png"),
        ("special_object", "special_objects/specialobj1.png"),
        ("jump_pad", "jump_pads/jump_pad.png"),
        ("yellow_orb", "orbs/yellow_orb.png"),
        ("cube_portal", "portals/CubePortalfront.png"),
        ("ship_portal", "portals/ShipPortalfront.png"),
        ("wave_portal", "portals/WavePortalfront.png"),
        ("ball_portal", "portals/BallPortal.png"),
        ("spider_portal", "portals/SpiderPortal.png"),
        ("play", "menubuttons/pm_playbtn.png"),
        ("settings", "menubuttons/m_m_settings_btn.png"),
        ("start_pos", "native_ui/edit_eStartPosBtn_001.png"),
    ] {
        art.insert(key.into(), server.load(path));
    }
    commands.insert_resource(EditorArt(art));
}
fn child(commands: &mut Commands, parent: Entity, bundle: impl Bundle) -> Entity {
    let id = commands.spawn(bundle).id();
    commands.entity(parent).add_child(id);
    id
}
fn group(commands: &mut Commands, parent: Entity, node: Node) -> Entity {
    child(commands, parent, node)
}
fn icon_button(
    commands: &mut Commands,
    parent: Entity,
    action: Action,
    icon: Icon,
    tip: impl Into<String>,
    active: bool,
) -> Entity {
    let tip = tip.into();
    let color = if active {
        Color::srgb(0.22, 0.55, 0.05)
    } else {
        Color::srgb(0.08, 0.22, 0.42)
    };
    let button = child(
        commands,
        parent,
        (
            Button,
            action,
            Tooltip(tip.clone()),
            Name::new(tip),
            ImageButton { background: color },
            UiTransform::default(),
            Node {
                width: px(46),
                height: px(46),
                flex_shrink: 0.,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor::all(if active {
                Color::srgb(0.65, 1., 0.2)
            } else {
                Color::srgb(0.25, 0.55, 0.85)
            }),
        ),
    );
    editor_icons::draw(commands, button, icon);
    button
}
fn art_button(
    commands: &mut Commands,
    parent: Entity,
    art: &EditorArt,
    key: &str,
    action: Action,
    tip: impl Into<String>,
    active: bool,
) {
    let button = icon_button(commands, parent, action, Icon::Build, tip, active);
    // Keep the same accessible/hover label; artwork occupies the button itself.
    commands.entity(button).despawn_children();
    let size = if key.ends_with("portal") {
        Vec2::new(18., 38.)
    } else if key == "start_pos" {
        Vec2::new(38., 24.)
    } else if key == "platform" {
        Vec2::new(36., 14.)
    } else if key == "minispike" {
        Vec2::new(36., 18.)
    } else {
        Vec2::splat(34.)
    };
    child(
        commands,
        button,
        (
            Node {
                width: px(size.x),
                height: px(size.y),
                ..default()
            },
            ImageNode::new(art.0.get(key).cloned().unwrap_or_default()),
        ),
    );
}
fn caption(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    node: Node,
    size: f32,
) {
    child(
        commands,
        parent,
        (
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(Color::WHITE),
            TextShadow::default(),
            node,
        ),
    );
}
fn tool_icon(tool: usize) -> Icon {
    match tool {
        11 => Icon::Robot,
        12 => Icon::Spider,
        13 => Icon::Ball,
        14 => Icon::Background,
        15 => Icon::Ground,
        17 => Icon::Flag,
        _ => Icon::Color,
    }
}

pub(super) fn build(commands: &mut Commands, game: &Game, art: &EditorArt) {
    let Some(editor) = &game.editor else {
        return;
    };
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
    caption(
        commands,
        root,
        format!(
            "{}{}",
            editor.level.name,
            if editor.dirty { " *" } else { "" }
        ),
        Node {
            position_type: PositionType::Absolute,
            left: percent(22),
            top: px(17),
            ..default()
        },
        22.,
    );
    caption(
        commands,
        root,
        format!(
            "{} objects · {:.0}%",
            editor.level.objects.len(),
            editor.zoom * 100.
        ),
        Node {
            position_type: PositionType::Absolute,
            left: percent(22),
            top: px(46),
            ..default()
        },
        14.,
    );
    let left = group(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(15),
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            ..default()
        },
    );
    if let Some(object) = editor
        .selected
        .and_then(|id| editor.level.objects.get(id))
        .filter(|o| o.kind == "start_pos")
    {
        let settings = group(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(22),
                top: px(75),
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
        );
        icon_button(
            commands,
            settings,
            Action::StartMode,
            Icon::Robot,
            "Cycle start mode (M)",
            false,
        );
        icon_button(
            commands,
            settings,
            Action::StartGravity,
            Icon::Arrow(-90.),
            "Cycle start gravity (V)",
            false,
        );
        caption(
            commands,
            settings,
            format!(
                "Start: {} / {}",
                object
                    .extra
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auto"),
                object
                    .extra
                    .get("gravity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auto")
            ),
            Node::default(),
            16.,
        );
    }
    art_button(
        commands,
        left,
        art,
        "play",
        Action::Test,
        "Test level (F5)",
        false,
    );
    for (action, icon, tip) in [
        (Action::Undo, Icon::Undo, "Undo (Ctrl+Z)"),
        (Action::Redo, Icon::Redo, "Redo (Ctrl+Shift+Z / Ctrl+Y)"),
        (Action::ZoomIn, Icon::ZoomIn, "Zoom in"),
        (Action::ZoomOut, Icon::ZoomOut, "Zoom out"),
        (Action::ResetView, Icon::Reset, "Reset view"),
    ] {
        icon_button(commands, left, action, icon, tip, false);
    }
    let top = group(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(15),
            column_gap: px(9),
            ..default()
        },
    );
    icon_button(
        commands,
        top,
        Action::Save,
        Icon::Save,
        "Save (Ctrl+S)",
        false,
    );
    art_button(
        commands,
        top,
        art,
        "settings",
        Action::EditorSettings,
        "Level settings and files",
        editor.settings_open,
    );
    icon_button(
        commands,
        top,
        Action::ExitEditor,
        Icon::Exit,
        "Exit editor",
        false,
    );
    let right = group(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(91),
            width: px(152),
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(5),
            row_gap: px(7),
            ..default()
        },
    );
    for (action, icon, tip, active) in [
        (Action::Duplicate, Icon::Copy, "Duplicate (Ctrl+D)", false),
        (Action::Delete, Icon::Trash, "Delete selected", false),
        (Action::Rotation, Icon::Angle, "Set exact rotation", false),
        (
            Action::Rotate(-90.),
            Icon::RotateLeft,
            "Rotate left 90 degrees",
            false,
        ),
        (
            Action::Rotate(90.),
            Icon::RotateRight,
            "Rotate right 90 degrees",
            false,
        ),
        (Action::Trigger, Icon::Color, "Edit selected trigger", false),
        (Action::Grid, Icon::Grid, "Show grid (G)", editor.grid),
        (
            Action::Snap,
            Icon::Snap,
            "Snap to grid",
            editor.snap_to_grid,
        ),
        (Action::Move(0., 80.), Icon::Arrow(-90.), "Move up", false),
        (
            Action::Move(-80., 0.),
            Icon::Arrow(180.),
            "Move left",
            false,
        ),
        (Action::Move(0., -80.), Icon::Arrow(90.), "Move down", false),
        (Action::Move(80., 0.), Icon::Arrow(0.), "Move right", false),
    ] {
        icon_button(commands, right, action, icon, tip, active);
    }
    let panel = child(
        commands,
        root,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(0),
                width: percent(100),
                height: px(PANEL_HEIGHT),
                border: UiRect {
                    top: px(3),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.08, 0.19, 0.97)),
            BorderColor::all(Color::srgb(0.4, 0.7, 1.)),
        ),
    );
    let modes = group(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(14),
            top: px(24),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
    );
    for (mode, icon, tip) in [
        (EditorMode::Build, Icon::Build, "Build objects"),
        (EditorMode::Edit, Icon::Cursor, "Select and move objects"),
        (EditorMode::Delete, Icon::Trash, "Delete objects"),
    ] {
        icon_button(
            commands,
            modes,
            Action::EditorMode(mode),
            icon,
            tip,
            editor.mode == mode,
        );
    }
    let categories = group(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(96),
            top: px(9),
            column_gap: px(10),
            ..default()
        },
    );
    for (category, icon, tip) in [
        (0, Icon::Build, "Blocks and platforms"),
        (1, Icon::Hazard, "Hazards"),
        (2, Icon::Robot, "Gameplay objects and portals"),
        (3, Icon::Color, "Triggers"),
    ] {
        icon_button(
            commands,
            categories,
            Action::EditorCategory(category),
            icon,
            tip,
            editor.category == category,
        );
    }
    let palette = group(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(96),
            right: px(12),
            top: px(69),
            bottom: px(10),
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(9),
            row_gap: px(10),
            align_content: AlignContent::FlexStart,
            ..default()
        },
    );
    if editor.category == 0 {
        for n in 1..=11 {
            art_button(
                commands,
                palette,
                art,
                &format!("block{n}"),
                Action::BlockTexture(n),
                format!("Block {n}"),
                editor.mode == EditorMode::Build && editor.tool == 1 && editor.texture == n,
            );
        }
        art_button(
            commands,
            palette,
            art,
            "platform",
            Action::Tool(2),
            "Platform",
            editor.mode == EditorMode::Build && editor.tool == 2,
        );
    } else {
        let range = match editor.category {
            1 => 3..6,
            2 => 6..14,
            _ => 14..19,
        };
        for tool in range {
            let name = crate::editor::TOOLS[tool];
            let active = editor.mode == EditorMode::Build && editor.tool == tool;
            if art.0.contains_key(name) {
                art_button(
                    commands,
                    palette,
                    art,
                    name,
                    Action::Tool(tool),
                    name.replace('_', " "),
                    active,
                );
            } else {
                icon_button(
                    commands,
                    palette,
                    Action::Tool(tool),
                    tool_icon(tool),
                    name.replace('_', " "),
                    active,
                );
            }
        }
    }
    if editor.settings_open {
        let settings = child(
            commands,
            root,
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: px(14),
                    top: px(75),
                    width: px(210),
                    padding: UiRect::all(px(12)),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(12),
                    row_gap: px(12),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.16, 0.28)),
                BorderColor::all(Color::srgb(0.6, 0.9, 1.)),
                ZIndex(10),
            ),
        );
        for (action, icon, tip) in [
            (Action::New, Icon::Build, "New level"),
            (Action::Open, Icon::Folder, "Open level (Ctrl+O)"),
            (Action::SaveAs, Icon::SaveAs, "Save as (Ctrl+Shift+S)"),
            (Action::Name, Icon::Pencil, "Level name"),
            (Action::Music, Icon::Music, "Choose music"),
            (Action::Background, Icon::Picture, "Choose background"),
        ] {
            icon_button(commands, settings, action, icon, tip, false);
        }
    }
    let tip = child(
        commands,
        root,
        (
            TooltipText,
            Text::new(&game.status),
            TextFont {
                font_size: FontSize::Px(15.),
                ..default()
            },
            TextColor(Color::WHITE),
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(96),
                bottom: px(PANEL_HEIGHT + 12.),
                ..default()
            },
        ),
    );
    commands.entity(tip).insert(bevy::ui::FocusPolicy::Pass);
}
pub(super) fn tooltip(
    game: Res<Game>,
    buttons: Query<(&Interaction, &Tooltip)>,
    mut labels: Query<&mut Text, With<TooltipText>>,
) {
    if game.screen != super::Screen::Editor {
        return;
    }
    let value = if game.confirm_exit {
        game.status.as_str()
    } else {
        buttons
            .iter()
            .find(|(i, _)| **i != Interaction::None)
            .map(|(_, t)| t.0.as_str())
            .unwrap_or(&game.status)
    };
    for mut label in &mut labels {
        if label.0 != value {
            label.0 = value.to_owned();
        }
    }
}
/// Shared with world input: UI clicks must never place or move level objects.
pub(super) fn canvas_contains(position: Vec2, size: Vec2, settings_open: bool) -> bool {
    position.x >= 78.
        && position.x < size.x - 174.
        && position.y >= 78.
        && position.y < size.y - PANEL_HEIGHT
        && !(settings_open && position.x > size.x - 238. && position.y < 272.)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_and_toolbars_never_contain_button_text() {
        let mut app = App::new();
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.edit();
        app.insert_resource(game).init_resource::<EditorArt>();
        for category in 0..4 {
            let art = EditorArt::default();
            {
                let mut game = app.world_mut().resource_mut::<Game>();
                let editor = game.editor.as_mut().unwrap();
                editor.category = category;
                editor.settings_open = category == 3;
            }
            let mut state =
                bevy::ecs::system::SystemState::<(Commands, Res<Game>)>::new(app.world_mut());
            let (mut commands, game) = state.get_mut(app.world_mut()).unwrap();
            build(&mut commands, &game, &art);
            state.apply(app.world_mut());
        }
        let buttons = app
            .world_mut()
            .query_filtered::<Entity, With<Button>>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert!(buttons.len() > 100);
        for button in buttons {
            let mut pending = vec![button];
            while let Some(entity) = pending.pop() {
                assert!(
                    app.world().get::<Text>(entity).is_none(),
                    "Text inside an editor button"
                );
                if let Some(children) = app.world().get::<Children>(entity) {
                    pending.extend(children.iter());
                }
            }
        }
    }
    #[test]
    fn editor_controls_are_outside_the_world_input_region() {
        for size in [
            Vec2::new(960., 640.),
            Vec2::new(1280., 720.),
            Vec2::new(1920., 1080.),
        ] {
            assert!(canvas_contains(
                Vec2::new(size.x / 2., size.y / 2.),
                size,
                false
            ));
            for p in [
                Vec2::new(32., 120.),
                Vec2::new(size.x - 32., 120.),
                Vec2::new(200., size.y - 60.),
                Vec2::new(200., 30.),
            ] {
                assert!(!canvas_contains(p, size, false));
            }
            assert!(!canvas_contains(Vec2::new(size.x - 220., 120.), size, true));
        }
    }
}
