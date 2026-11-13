use super::{
    Game, Screen,
    native_menu::{self as native, FontKind, NativeArt, add, button, center, image, n, text},
    settings::{MusicSlider, VolumeVisual, long_button, sliced},
    ui::{Action, UiRoot},
};
use crate::{
    editor::{Editor, EditorMode, TOOLS},
    storage::MusicChannel,
};
use bevy::{
    prelude::*,
    ui::{InteractionDisabled, RelativeCursorPosition},
};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub(super) struct EditorArt(HashMap<String, Handle<Image>>);
#[derive(Component)]
pub(super) struct Tooltip(String);
#[derive(Component)]
pub(super) struct TooltipText;
#[derive(Component)]
pub(super) struct EditorScrub;
#[derive(Component)]
pub(super) struct ScrubThumb;
#[derive(Component)]
pub(super) struct EditorTimeSlider;
#[derive(Component)]
pub(super) struct SpeedKnob;
#[derive(Component)]
pub(super) struct SpeedLabel;
#[derive(Component)]
pub(super) struct SpeedFill;

pub(super) fn load_art(mut commands: Commands, server: Res<AssetServer>) {
    let mut art = HashMap::new();
    for i in 1..=11 {
        art.insert(
            format!("block{i}"),
            server.load(format!("blocks/block{i}.png")),
        );
    }
    for kind in TOOLS {
        let object = crate::level::LevelObject::new(kind, 0., 0.);
        if let Some(path) = super::render::texture(&object) {
            art.insert(kind.into(), server.load(path));
        }
    }
    commands.insert_resource(EditorArt(art));
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
fn bottom_left(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: n(x - w / 2.),
        bottom: n(y - h / 2.),
        width: n(w),
        height: n(h),
        ..default()
    }
}
fn bottom_right(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: n(x - w / 2.),
        bottom: n(y - h / 2.),
        width: n(w),
        height: n(h),
        ..default()
    }
}
fn bottom_center(x: f32, y: f32, w: f32, h: f32) -> Node {
    let mut node = center(x, 0., w, h);
    node.top = Val::Auto;
    node.bottom = n(y - h / 2.);
    node
}
fn tip(commands: &mut Commands, e: Entity, description: &str, enabled: bool) {
    commands.entity(e).insert((
        Tooltip(description.into()),
        Name::new(description.to_owned()),
    ));
    if !enabled {
        commands.entity(e).insert(InteractionDisabled);
    }
}
#[allow(clippy::too_many_arguments)]
fn art_button(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    name: &str,
    action: Action,
    node: Node,
    enabled: bool,
    description: &str,
) -> Entity {
    let e = button(commands, root, art, name, action, node);
    if !enabled {
        commands
            .entity(e)
            .insert(ImageNode::new(art.image(name)).with_color(Color::srgba(1., 1., 1., 0.35)));
    }
    tip(commands, e, description, enabled);
    e
}
#[allow(clippy::too_many_arguments)]
fn tile(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    icon: &str,
    action: Action,
    node: Node,
    active: bool,
    enabled: bool,
    description: &str,
) -> Entity {
    let e = add(
        commands,
        root,
        (
            Button,
            action,
            super::menu::ImageButton::default(),
            UiTransform::default(),
            BackgroundColor(Color::NONE),
            node,
        ),
    );
    let mode_button = matches!(
        action,
        Action::EditorMode(..)
            | Action::EditorView
            | Action::EditorMoveStep(..)
            | Action::EditorCustomStep
            | Action::EditorDeleteFilter(..)
            | Action::EditorDeleteAll(..)
            | Action::Delete
    );
    let skin = if mode_button {
        if active {
            "GJ_button_02.png"
        } else {
            "GJ_button_04.png"
        }
    } else {
        "GJ_button_01.png"
    };
    let bg = sliced(
        commands,
        e,
        art,
        skin,
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        6.,
    );
    let icon_entity = image(
        commands,
        e,
        art,
        icon,
        Node {
            position_type: PositionType::Absolute,
            left: percent(8),
            top: percent(8),
            width: percent(84),
            height: percent(84),
            ..default()
        },
    );
    if !enabled || (!mode_button && !active) {
        let alpha = if enabled { 0.65 } else { 0.3 };
        commands.entity(bg).insert(
            ImageNode::new(art.image(skin))
                .with_color(Color::srgba(1., 1., 1., alpha))
                .with_mode(NodeImageMode::Sliced(bevy::sprite::TextureSlicer {
                    border: bevy::sprite::BorderRect::all(24.),
                    max_corner_scale: 720. / 1280.,
                    ..default()
                })),
        );
        commands
            .entity(icon_entity)
            .insert(ImageNode::new(art.image(icon)).with_color(Color::srgba(1., 1., 1., alpha)));
    }
    tip(commands, e, description, enabled);
    e
}
fn label(
    commands: &mut Commands,
    root: Entity,
    value: impl Into<String>,
    kind: FontKind,
    scale: f32,
    node: Node,
) {
    text(commands, root, value, kind, scale, node);
}
fn root(commands: &mut Commands, z: i32, shade: f32) -> Entity {
    commands
        .spawn((
            UiRoot,
            GlobalZIndex(z),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., shade)),
        ))
        .id()
}
pub(super) fn build(commands: &mut Commands, game: &Game, objects: &EditorArt, art: &NativeArt) {
    let Some(editor) = &game.editor else {
        return;
    };
    let root = root(commands, 0, 0.);
    if editor.view_only {
        art_button(
            commands,
            root,
            art,
            "sideBtn_hide_001.png",
            Action::EditorView,
            native::left(25., 20., 28., 28.),
            true,
            "Show editor controls",
        );
        return;
    }
    for (name, action, x, enabled, description) in [
        (
            "GJ_undoBtn_001.png",
            Action::Undo,
            25.,
            editor.can_undo(),
            "Undo (Ctrl+Z)",
        ),
        (
            "GJ_redoBtn_001.png",
            Action::Redo,
            70.,
            editor.can_redo(),
            "Redo (Ctrl+Shift+Z)",
        ),
        (
            "GJ_resetBtn_001.png",
            Action::Delete,
            115.,
            editor.selected.is_some(),
            "Delete selected objects",
        ),
    ] {
        art_button(
            commands,
            root,
            art,
            name,
            action,
            native::left(x, 20., 38., 38.),
            enabled,
            description,
        );
    }
    art_button(
        commands,
        root,
        art,
        "sideBtn_hide_001.png",
        Action::EditorView,
        native::left(158., 20., 27., 27.),
        true,
        "Hide editor controls",
    );
    for (name, action, y, size, description) in [
        (
            "GJ_playMusicBtn_001.png",
            Action::Test,
            65.,
            38.,
            "Playtest with music (F5)",
        ),
        (
            "GJ_playEditorBtn_001.png",
            Action::Test,
            110.,
            38.,
            "Playtest from Start Pos",
        ),
        ("GJ_zoomInBtn_001.png", Action::ZoomIn, 155., 30., "Zoom in"),
        (
            "GJ_zoomOutBtn_001.png",
            Action::ZoomOut,
            190.,
            30.,
            "Zoom out",
        ),
    ] {
        art_button(
            commands,
            root,
            art,
            name,
            action,
            native::left(25., y, size, size),
            true,
            description,
        );
    }
    let scrub = add(
        commands,
        root,
        (
            Button,
            EditorScrub,
            RelativeCursorPosition::default(),
            center(5., 20., 155., 8.),
            BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
            BorderColor::all(Color::WHITE),
        ),
    );
    commands.entity(scrub).insert(Node {
        border: UiRect::all(n(1.)),
        border_radius: BorderRadius::all(n(4.)),
        ..center(5., 20., 155., 8.)
    });
    let knob = add(
        commands,
        scrub,
        (
            ScrubThumb,
            Node {
                position_type: PositionType::Absolute,
                left: percent(0),
                top: n(-10.),
                margin: UiRect::left(n(-14.)),
                width: n(28.),
                height: n(28.),
                ..default()
            },
        ),
    );
    image(
        commands,
        knob,
        art,
        "GJ_plainBtn_001.png",
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
    );
    image(
        commands,
        knob,
        art,
        "edit_flipXBtn_001.png",
        Node {
            position_type: PositionType::Absolute,
            left: percent(20),
            top: percent(20),
            width: percent(60),
            height: percent(60),
            ..default()
        },
    );
    for (name, action, x, size, description) in [
        (
            "GJ_plainBtn_001.png",
            Action::Grid,
            142.,
            27.,
            "Show grid (G)",
        ),
        (
            "GJ_viewLevelsBtn_001.png",
            Action::EditorFiles,
            106.,
            27.,
            "Level and file settings",
        ),
        (
            "GJ_optionsBtn02_001.png",
            Action::EditorSettings,
            65.,
            38.,
            "Level settings",
        ),
        (
            "GJ_pauseEditorBtn_001.png",
            Action::EditorPause,
            20.,
            38.,
            "Editor pause (Esc)",
        ),
    ] {
        let e = art_button(
            commands,
            root,
            art,
            name,
            action,
            right(x, 20., size, size),
            true,
            description,
        );
        if matches!(action, Action::Grid) {
            image(
                commands,
                e,
                art,
                "edit_gridBtn_001.png",
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(15),
                    top: percent(15),
                    width: percent(70),
                    height: percent(70),
                    ..default()
                },
            );
        }
    }
    let selected = editor.selected.is_some();
    for (name, action, col, row, enabled, description) in [
        (
            "GJ_copyBtn2_001.png",
            Action::EditorCopy,
            0,
            0,
            selected,
            "Copy (Ctrl+C)",
        ),
        (
            "GJ_pasteBtn2_001.png",
            Action::EditorPaste,
            1,
            0,
            editor.can_paste(),
            "Paste (Ctrl+V)",
        ),
        (
            "GJ_duplicateObjectBtn2_001.png",
            Action::Duplicate,
            2,
            0,
            selected,
            "Copy and paste (Ctrl+D)",
        ),
        (
            "GJ_editObjBtn4_001.png",
            Action::EditorSelectionSettings,
            0,
            1,
            selected,
            "Edit selected trigger or Start Pos",
        ),
        (
            "GJ_groupIDBtn2_001.png",
            Action::ObjectGroups,
            1,
            1,
            selected,
            "Edit object groups",
        ),
        (
            "GJ_editObjBtn3_001.png",
            Action::Rotation,
            2,
            1,
            selected,
            "Edit object",
        ),
        (
            "GJ_copyStateBtn_001.png",
            Action::EditorCopyState,
            0,
            2,
            selected,
            "Copy object properties",
        ),
        (
            "GJ_pasteStateBtn_001.png",
            Action::EditorPasteState(false),
            1,
            2,
            selected && editor.can_paste_state(),
            "Paste object properties",
        ),
        (
            "GJ_pasteColorBtn_001.png",
            Action::EditorPasteState(true),
            2,
            2,
            selected && editor.can_paste_state(),
            "Paste trigger color",
        ),
        (
            "GJ_editHSVBtn2_001.png",
            Action::Trigger,
            0,
            3,
            selected,
            "Edit trigger color",
        ),
        (
            "GJ_goToLayerBtn_001.png",
            Action::EditorGoToLayer,
            1,
            3,
            selected,
            "Go to selected object's layer",
        ),
        (
            "GJ_deSelBtn2_001.png",
            Action::EditorDeselect,
            2,
            3,
            selected,
            "Deselect objects",
        ),
    ] {
        art_button(
            commands,
            root,
            art,
            name,
            action,
            right(98. - col as f32 * 38., 61. + row as f32 * 35., 32., 32.),
            enabled,
            description,
        );
    }
    let layer_button = add(
        commands,
        root,
        (
            Button,
            Action::ObjectLayer,
            super::menu::ImageButton::default(),
            UiTransform::default(),
            BackgroundColor(Color::NONE),
            right(45., 198., 25., 25.),
        ),
    );
    tip(
        commands,
        layer_button,
        "Set selected object's editor layer",
        true,
    );
    label(
        commands,
        layer_button,
        if editor.show_all_layers {
            "All".into()
        } else {
            editor.layer.to_string()
        },
        FontKind::Big,
        0.5,
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
    );
    for (x, flip) in [(75., false), (15., true)] {
        let e = art_button(
            commands,
            root,
            art,
            "GJ_arrow_03_001.png",
            Action::EditorLayer(if flip { 1 } else { -1 }),
            right(x, 198., 20., 25.),
            true,
            "Change editor layer",
        );
        if flip {
            commands.entity(e).insert((
                super::menu::MirroredButton,
                UiTransform {
                    scale: Vec2::new(-1., 1.),
                    ..default()
                },
            ));
        }
    }
    art_button(
        commands,
        root,
        art,
        "GJ_arrow_02_001.png",
        Action::EditorAllLayers,
        right(102., 198., 20., 20.),
        true,
        "Toggle all editor layers",
    );
    add(
        commands,
        root,
        (
            Node {
                position_type: PositionType::Absolute,
                left: n(0.),
                bottom: n(0.),
                width: percent(100),
                height: n(90.),
                border: UiRect::top(n(1.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.035, 0.08, 0.7)),
            BorderColor::all(Color::srgba(1., 1., 1., 0.3)),
        ),
    );
    for (icon, action, x, y, active, description) in [
        (
            "edit_buildBtn_001.png",
            Action::EditorMode(EditorMode::Build),
            25.,
            66.,
            editor.mode == EditorMode::Build,
            "Build objects",
        ),
        (
            "edit_editBtn_001.png",
            Action::EditorMode(EditorMode::Edit),
            69.,
            66.,
            editor.mode == EditorMode::Edit,
            "Select and move objects",
        ),
        (
            "edit_deleteBtn_001.png",
            Action::EditorMode(EditorMode::Delete),
            25.,
            23.,
            editor.mode == EditorMode::Delete,
            "Delete objects",
        ),
        (
            "edit_viewBtn_001.png",
            Action::EditorView,
            69.,
            23.,
            false,
            "View level without editor controls",
        ),
    ] {
        tile(
            commands,
            root,
            art,
            icon,
            action,
            bottom_left(x, y, 40., 40.),
            active,
            true,
            description,
        );
    }
    for (icon, action, x, y, active, description) in [
        (
            "edit_swipeBtn_001.png",
            Action::EditorSwipe,
            69.,
            66.,
            editor.swipe,
            "Swipe build or box selection",
        ),
        (
            "edit_enableRotateBtn_001.png",
            Action::EditorRotateObjects,
            25.,
            66.,
            editor.rotate_objects,
            "Rotate around selection center",
        ),
        (
            "edit_freeMoveBtn_001.png",
            Action::Snap,
            69.,
            23.,
            !editor.snap_to_grid,
            "Free move",
        ),
        (
            "edit_snapBtn_001.png",
            Action::Snap,
            25.,
            23.,
            editor.snap_to_grid,
            "Snap to grid",
        ),
    ] {
        tile(
            commands,
            root,
            art,
            icon,
            action,
            bottom_right(x, y, 40., 40.),
            active,
            true,
            description,
        );
    }
    if editor.mode == EditorMode::Build {
        build_palette(commands, root, editor, objects, art);
    } else if editor.mode == EditorMode::Edit {
        build_transform_tools(commands, root, art, editor);
    } else {
        build_delete_tools(commands, root, art, editor);
    }
    if editor.settings_open {
        build_level_settings(commands, game, art);
    }
    if editor.start_settings_open {
        build_start_settings(commands, game, art);
    }
    if editor.pause_open {
        build_pause(commands, game, art);
    }
    if editor.files_open {
        build_file_settings(commands, game, art);
    }
    add(
        commands,
        root,
        (
            TooltipText,
            Text::new(&game.status),
            TextFont {
                font_size: FontSize::Vh(1.5),
                ..default()
            },
            TextColor(Color::WHITE),
            TextShadow::default(),
            Node {
                position_type: PositionType::Absolute,
                left: n(50.),
                bottom: n(112.),
                ..default()
            },
            bevy::ui::FocusPolicy::Pass,
        ),
    );
}
fn build_palette(
    commands: &mut Commands,
    root: Entity,
    editor: &Editor,
    objects: &EditorArt,
    art: &NativeArt,
) {
    let categories = add(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: n(48.),
            right: n(48.),
            bottom: n(90.),
            height: n(20.),
            ..default()
        },
    );
    for (i, (icon, category)) in [
        ("blocks/block1.png", Some(0)),
        ("blocks/block2.png", None),
        ("blocks/block3.png", None),
        ("spikes/spike.png", Some(1)),
        ("platforms/platform.png", None),
        ("orbs/yellow_orb.png", Some(2)),
        ("special_objects/specialobj1.png", None),
        ("blocks/block4.png", None),
        ("blocks/block5.png", None),
        ("blocks/block6.png", None),
        ("special_objects/specialobj1.png", None),
        ("ball/Ball001.png", None),
        ("native_ui/GJ_editHSVBtn2_001.png", Some(3)),
        ("native_ui/edit_eStartPosBtn_001.png", Some(3)),
    ]
    .into_iter()
    .enumerate()
    {
        let tab = add(
            commands,
            categories,
            (
                Button,
                Action::EditorCategory(category.unwrap_or(0)),
                super::menu::ImageButton::default(),
                UiTransform::default(),
                BackgroundColor(Color::NONE),
                Node {
                    width: percent(100. / 14.),
                    height: percent(100),
                    ..default()
                },
            ),
        );
        image(
            commands,
            tab,
            art,
            if category == Some(editor.category) {
                "GJ_tabOn_001.png"
            } else {
                "GJ_tabOff_001.png"
            },
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        );
        let handle = if let Some(name) = icon.strip_prefix("native_ui/") {
            art.image(name)
        } else if let Some((_, file)) = icon.rsplit_once('/') {
            objects
                .0
                .get(file.trim_end_matches(".png"))
                .cloned()
                .unwrap_or_default()
        } else {
            Handle::default()
        };
        let handle = match i {
            0 => objects.0.get("block1"),
            1 => objects.0.get("block2"),
            2 => objects.0.get("block3"),
            3 => objects.0.get("spike"),
            4 => objects.0.get("platform"),
            5 => objects.0.get("yellow_orb"),
            6 | 10 => objects.0.get("special_object"),
            7 => objects.0.get("block4"),
            8 => objects.0.get("block5"),
            9 => objects.0.get("block6"),
            _ => None,
        }
        .cloned()
        .unwrap_or(handle);
        add(
            commands,
            tab,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(25),
                    top: percent(10),
                    width: percent(50),
                    height: percent(80),
                    ..default()
                },
                ImageNode::new(handle).with_color(Color::srgba(
                    1.,
                    1.,
                    1.,
                    if category.is_some() { 1. } else { 0.35 },
                )),
            ),
        );
        if category.is_none() {
            commands.entity(tab).insert(InteractionDisabled);
        }
    }
    let mut items = Vec::new();
    if editor.category == 0 {
        for i in 1..=11 {
            items.push((
                format!("block{i}"),
                Action::BlockTexture(i),
                editor.tool == 1 && editor.texture == i,
            ));
        }
        items.push(("platform".into(), Action::Tool(2), editor.tool == 2));
    } else {
        let range = match editor.category {
            1 => 3..6,
            2 => 6..14,
            _ => 14..19,
        };
        for i in range {
            items.push((TOOLS[i].into(), Action::Tool(i), editor.tool == i));
        }
    }
    for (i, (name, action, active)) in items.iter().enumerate() {
        let x = -102.5 + (i % 6) as f32 * 41.;
        let y = 71. - (i / 6) as f32 * 41.;
        let e = add(
            commands,
            root,
            (
                Button,
                *action,
                super::menu::ImageButton::default(),
                UiTransform::default(),
                BackgroundColor(Color::NONE),
                bottom_center(x, y, 38., 38.),
            ),
        );
        sliced(
            commands,
            e,
            art,
            if *active {
                "GJ_button_04.png"
            } else {
                "GJ_button_05.png"
            },
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            5.,
        );
        let handle = objects.0.get(name).cloned().unwrap_or_else(|| {
            art.image(match name.as_str() {
                "end_level" => "GJ_starsIcon_001.png",
                _ => "GJ_editHSVBtn2_001.png",
            })
        });
        let (w, h) = if name.ends_with("portal") {
            (15., 32.)
        } else if name == "platform" {
            (30., 12.)
        } else if name == "start_pos" {
            (30., 19.)
        } else {
            (30., 30.)
        };
        add(
            commands,
            e,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    top: percent(50),
                    margin: UiRect {
                        left: n(-w / 2.),
                        top: n(-h / 2.),
                        ..default()
                    },
                    width: n(w),
                    height: n(h),
                    ..default()
                },
                ImageNode::new(handle),
            ),
        );
        tip(commands, e, &name.replace('_', " "), true);
    }
    for (x, flip) in [(-166., false), (172., true)] {
        let e = art_button(
            commands,
            root,
            art,
            "GJ_arrow_02_001.png",
            Action::EditorPalettePage(if flip { 1 } else { -1 }),
            bottom_center(x, 50., 20., 25.),
            false,
            "Object palette page",
        );
        if flip {
            commands.entity(e).insert((
                super::menu::MirroredButton,
                UiTransform {
                    scale: Vec2::new(-1., 1.),
                    ..default()
                },
            ));
        }
    }
}
fn build_transform_tools(commands: &mut Commands, root: Entity, art: &NativeArt, editor: &Editor) {
    let selected = editor.selected.is_some();
    let grid = crate::level::GRID;
    for (i, step) in [
        grid * 5.,
        grid,
        0.,
        grid / 2.,
        grid / 4.,
        grid / 8.,
        crate::constants::GD_SCALE * 2.,
        crate::constants::GD_SCALE,
        crate::constants::GD_SCALE / 2.,
    ]
    .into_iter()
    .enumerate()
    {
        let icon = if i == 0 {
            "edit_upBtn3_001.png"
        } else if i == 1 {
            "edit_upBtn2_001.png"
        } else if (3..6).contains(&i) {
            "edit_upBtn5_001.png"
        } else {
            "edit_upBtn_001.png"
        };
        let e = tile(
            commands,
            root,
            art,
            icon,
            if i == 2 {
                Action::EditorCustomStep
            } else {
                Action::EditorMoveStep(step)
            },
            bottom_center(
                -51. + (i % 3) as f32 * 34.,
                73. - (i / 3) as f32 * 28.,
                30.,
                23.,
            ),
            (step - editor.move_step).abs() < 1e-8,
            true,
            "Choose movement step",
        );
        if i == 2 || i >= 6 {
            let tint = if i == 2 {
                Color::srgb(1., 0.5, 0.9)
            } else {
                Color::srgb(1., 0.8, 0.2)
            };
            let colored = image(
                commands,
                e,
                art,
                icon,
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(8),
                    top: percent(8),
                    width: percent(84),
                    height: percent(84),
                    ..default()
                },
            );
            commands
                .entity(colored)
                .insert(ImageNode::new(art.image(icon)).with_color(tint));
        }
    }
    for (icon, action, x, y, description) in [
        (
            "edit_flipXBtn_001.png",
            Action::EditorFlip(true),
            62.,
            73.,
            "Flip horizontally",
        ),
        (
            "edit_flipYBtn_001.png",
            Action::EditorFlip(false),
            96.,
            73.,
            "Flip vertically",
        ),
        (
            "edit_cwBtn_001.png",
            Action::Rotate(90.),
            130.,
            73.,
            "Rotate clockwise",
        ),
        (
            "edit_ccwBtn_001.png",
            Action::Rotate(-90.),
            164.,
            73.,
            "Rotate counterclockwise",
        ),
        (
            "edit_rotate45rBtn_001.png",
            Action::Rotate(45.),
            62.,
            45.,
            "Rotate 45 degrees",
        ),
        (
            "edit_rotate45lBtn_001.png",
            Action::Rotate(-45.),
            96.,
            45.,
            "Rotate minus 45 degrees",
        ),
        (
            "edit_freeRotateBtn_001.png",
            Action::Rotation,
            130.,
            45.,
            "Set exact rotation",
        ),
        (
            "edit_rotateSnapBtn_001.png",
            Action::Rotate(90.),
            164.,
            45.,
            "Rotate with snapping",
        ),
        (
            "edit_scaleBtn_001.png",
            Action::ObjectScale,
            62.,
            17.,
            "Scale object",
        ),
        (
            "edit_scaleXYBtn_001.png",
            Action::ObjectScale,
            96.,
            17.,
            "Scale X and Y",
        ),
        (
            "edit_warpBtn_001.png",
            Action::EditorSelectionSettings,
            130.,
            17.,
            "Warp is not supported",
        ),
    ] {
        tile(
            commands,
            root,
            art,
            icon,
            action,
            bottom_center(x, y, 30., 23.),
            true,
            selected && icon != "edit_warpBtn_001.png",
            description,
        );
    }
    for (icon, action, x, y) in [
        (
            "edit_leftBtn_001.png",
            Action::EditorNudge(-1., 0.),
            -164.,
            45.,
        ),
        (
            "edit_upBtn_001.png",
            Action::EditorNudge(0., 1.),
            -130.,
            72.,
        ),
        (
            "edit_rightBtn_001.png",
            Action::EditorNudge(1., 0.),
            -96.,
            45.,
        ),
        (
            "edit_downBtn_001.png",
            Action::EditorNudge(0., -1.),
            -130.,
            18.,
        ),
    ] {
        let e = art_button(
            commands,
            root,
            art,
            icon,
            action,
            bottom_center(x, y, 28., 28.),
            selected,
            "Move selection",
        );
        commands
            .entity(e)
            .insert(ImageNode::new(art.image(icon)).with_color(if selected {
                Color::srgb(0.6, 0.9, 0.1)
            } else {
                Color::srgb(0.35, 0.5, 0.1)
            }));
    }
    let step = if (editor.move_step - grid / 2.).abs() < 1e-8 {
        "1/2".into()
    } else if (editor.move_step - grid / 4.).abs() < 1e-8 {
        "1/4".into()
    } else if (editor.move_step - grid / 8.).abs() < 1e-8 {
        "1/8".into()
    } else if editor.move_step >= grid {
        format!("{}", editor.move_step / grid)
    } else {
        format!("{}", editor.move_step / crate::constants::GD_SCALE)
    };
    label(
        commands,
        root,
        step,
        FontKind::Big,
        0.55,
        bottom_center(-130., 45., 40., 20.),
    );
}
fn build_delete_tools(commands: &mut Commands, root: Entity, art: &NativeArt, editor: &Editor) {
    for (icon, action, x, y, description) in [
        (
            "edit_delBtn_001.png",
            Action::Delete,
            -122.,
            66.,
            "Delete selection",
        ),
        (
            "GJ_deleteAllIcon_001.png",
            Action::EditorDeleteAll(false),
            -122.,
            26.,
            "Delete matching objects in this layer; Ctrl+Z to undo",
        ),
        (
            "edit_eStartPosBtn_001.png",
            Action::EditorDeleteAll(true),
            -82.,
            66.,
            "Delete Start Pos markers in this layer",
        ),
        (
            "edit_findBtn_001.png",
            Action::EditorDeleteFilter(2),
            -6.,
            66.,
            "Filter by selected object type",
        ),
        (
            "edit_delBtn_001.png",
            Action::Delete,
            -6.,
            26.,
            "Delete selected objects",
        ),
    ] {
        tile(
            commands,
            root,
            art,
            icon,
            action,
            bottom_center(x, y, 35., 35.),
            false,
            if matches!(action, Action::Delete | Action::EditorDeleteFilter(2)) {
                editor.selected.is_some()
            } else {
                true
            },
            description,
        );
    }
    for (icon, filter, x, y, enabled) in [
        ("deleteFilter_none_001.png", 0, 74., 66., true),
        ("deleteFilter_static_001.png", 1, 114., 66., true),
        ("deleteFilter_details_001.png", 3, 74., 26., false),
        (
            "deleteFilter_custom_001.png",
            2,
            114.,
            26.,
            editor.selected.is_some(),
        ),
    ] {
        let active = match filter {
            0 => editor.delete_filter.is_none(),
            1 => editor.delete_filter.as_deref() == Some("solid"),
            2 => editor.delete_filter.as_ref().is_some_and(|s| s != "solid"),
            _ => false,
        };
        tile(
            commands,
            root,
            art,
            icon,
            Action::EditorDeleteFilter(filter),
            bottom_center(x, y, 35., 35.),
            active,
            enabled,
            "Choose deletion filter",
        );
    }
    for y in [66., 26.] {
        let e = add(
            commands,
            root,
            (
                Button,
                Action::EditorDeleteFilter(0),
                bottom_center(34., y, 35., 35.),
                InteractionDisabled,
            ),
        );
        sliced(
            commands,
            e,
            art,
            "GJ_button_04.png",
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            6.,
        );
        label(
            commands,
            e,
            "0",
            FontKind::Big,
            0.8,
            Node {
                position_type: PositionType::Absolute,
                left: n(0.),
                top: n(0.),
                width: percent(100),
                height: percent(100),
                ..default()
            },
        );
    }
}

fn modal(commands: &mut Commands, art: &NativeArt, title: &str) -> Entity {
    let root = root(commands, 1400, 0.8);
    sliced(
        commands,
        root,
        art,
        "GJ_square_01.png",
        center(0., 160., 450., 295.),
        10.,
    );
    label(
        commands,
        root,
        title,
        FontKind::Gold,
        0.9,
        center(0., 30., 350., 25.),
    );
    root
}
fn build_level_settings(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let Some(editor) = &game.editor else {
        return;
    };
    let root = root(commands, 1400, 0.65);
    sliced(
        commands,
        root,
        art,
        "GJ_square_01.png",
        center(0., 160., 450., 295.),
        10.,
    );
    label(
        commands,
        root,
        "Select Color:",
        FontKind::Gold,
        0.9,
        center(0., 27., 300., 25.),
    );
    label(
        commands,
        root,
        "Speed:",
        FontKind::Gold,
        0.65,
        center(-185., 27., 60., 20.),
    );
    art_button(
        commands,
        root,
        art,
        "boost_01_001.png",
        Action::EditorSettings,
        center(-185., 59., 18., 32.),
        false,
        "Only normal level speed is supported",
    );
    for (i, name) in ["BG:", "C:", "C2:", "Line:", "MC:", "MC2:"]
        .iter()
        .enumerate()
    {
        let x = -137. + i as f32 * 46.;
        label(
            commands,
            root,
            *name,
            FontKind::Big,
            0.52,
            center(x, 45., 43., 16.),
        );
        let e = art_button(
            commands,
            root,
            art,
            "GJ_colorBtn_001.png",
            Action::LevelColor(i == 1),
            center(x, 67., 27., 27.),
            i < 2,
            if i == 0 {
                "Background color"
            } else if i == 1 {
                "Ground color"
            } else {
                "Additional color channels are not available"
            },
        );
        let color = match i {
            0 => rgba(
                editor
                    .level
                    .initial_color(false, [40. / 255., 125. / 255., 1., 1.]),
            ),
            1 => rgba(editor.level.initial_color(true, [0., 0.4, 1., 1.])),
            3 => Color::WHITE,
            _ => Color::srgb(0.15, 0.45, 1.),
        };
        commands
            .entity(e)
            .insert(ImageNode::new(art.image("GJ_colorBtn_001.png")).with_color(color));
    }
    label(
        commands,
        root,
        "More:",
        FontKind::Big,
        0.52,
        center(137., 45., 45., 16.),
    );
    let more = long_button(
        commands,
        root,
        art,
        "+",
        Action::EditorSettings,
        (137., 67., 27.),
        "GJ_button_04.png",
    );
    tip(
        commands,
        more,
        "Additional color channels are not available",
        false,
    );
    label(
        commands,
        root,
        "Mode:",
        FontKind::Gold,
        0.65,
        center(-185., 106., 65., 20.),
    );
    art_button(
        commands,
        root,
        art,
        "edit_eStartPosBtn_001.png",
        Action::EditorSettings,
        center(-185., 138., 30., 30.),
        false,
        "Use Start Pos to choose a playtest mode",
    );
    label(
        commands,
        root,
        "Game Type:",
        FontKind::Gold,
        0.9,
        center(0., 102., 250., 28.),
    );
    for (caption, x, skin, description) in [
        ("Classic", -65., "GJ_button_01.png", "Classic mode"),
        (
            "Platformer",
            65.,
            "GJ_button_04.png",
            "Platformer mode is not supported",
        ),
    ] {
        let e = long_button(
            commands,
            root,
            art,
            caption,
            Action::EditorSettings,
            (x, 128., 110.),
            skin,
        );
        commands.entity(e).insert(center(x, 128., 110., 23.));
        tip(commands, e, description, false);
    }
    label(
        commands,
        root,
        "Select Song:",
        FontKind::Gold,
        0.85,
        center(-65., 168., 150., 25.),
    );
    for (caption, x, action, skin) in [
        ("Normal", 40., Action::EditorSong(1), "GJ_button_01.png"),
        ("Custom", 102., Action::Music, "GJ_button_04.png"),
    ] {
        let e = long_button(commands, root, art, caption, action, (x, 168., 53.), skin);
        commands.entity(e).insert(center(x, 168., 53., 18.));
    }
    add(
        commands,
        root,
        (
            Node {
                border_radius: BorderRadius::all(n(10.)),
                ..center(0., 225., 300., 90.)
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.25)),
        ),
    );
    let song = if editor.level.music.contains("BackOnTrack") {
        "02: Back On Track".to_owned()
    } else if editor.level.music.contains("Polargeist") {
        "03: Polargeist".into()
    } else if editor.level.music.contains("StereoMadness") {
        "01: Stereo Madness".into()
    } else if editor.level.music.is_empty() {
        "No song selected".into()
    } else {
        std::path::Path::new(&editor.level.music)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    };
    let scale = (205. / art.text_width(&song, FontKind::Big)).min(0.9);
    label(
        commands,
        root,
        song,
        FontKind::Big,
        scale,
        center(0., 226., 220., 28.),
    );
    for (x, delta) in [(-118., -1), (118., 1)] {
        let e = art_button(
            commands,
            root,
            art,
            "navArrowBtn_001.png",
            Action::EditorSong(delta),
            center(x, 226., 16., 24.),
            true,
            "Select song",
        );
        if delta < 0 {
            commands.entity(e).insert((
                super::menu::MirroredButton,
                UiTransform {
                    scale: Vec2::new(-1., 1.),
                    ..default()
                },
            ));
        }
    }
    label(
        commands,
        root,
        "Options:",
        FontKind::Gold,
        0.65,
        center(-185., 187., 70., 20.),
    );
    art_button(
        commands,
        root,
        art,
        "GJ_optionsBtn_001.png",
        Action::EditorFiles,
        center(-185., 219., 27., 27.),
        true,
        "Level files and name",
    );
    for (caption, y, texture, enabled) in [
        ("BG:", 56., "background.png", true),
        ("G:", 121., "ground.png", false),
        ("MG:", 186., "GJ_button_04.png", false),
    ] {
        label(
            commands,
            root,
            caption,
            FontKind::Gold,
            0.65,
            center(185., y - 28., 50., 20.),
        );
        let e = art_button(
            commands,
            root,
            art,
            texture,
            Action::Background,
            center(185., y, 40., 40.),
            enabled,
            "Choose background image",
        );
        if enabled {
            commands
                .entity(e)
                .insert(ImageNode::new(art.image(texture)).with_color(Color::srgb(0.7, 0.7, 0.7)));
        }
    }
    let font = long_button(
        commands,
        root,
        art,
        "Font",
        Action::EditorSettings,
        (185., 232., 44.),
        "GJ_button_04.png",
    );
    tip(commands, font, "Text objects are not available", false);
    long_button(
        commands,
        root,
        art,
        "OK",
        Action::EditorSettings,
        (0., 289., 54.),
        "GJ_button_01.png",
    );
}
fn rgba(c: [f32; 4]) -> Color {
    Color::srgba(c[0], c[1], c[2], c[3])
}
fn build_file_settings(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let Some(editor) = &game.editor else {
        return;
    };
    let root = modal(commands, art, "Level Files");
    label(
        commands,
        root,
        &editor.level.name,
        FontKind::Big,
        0.7,
        center(0., 65., 350., 28.),
    );
    long_button(
        commands,
        root,
        art,
        "Name",
        Action::Name,
        (-95., 100., 120.),
        "GJ_button_01.png",
    );
    long_button(
        commands,
        root,
        art,
        "Background",
        Action::Background,
        (95., 100., 120.),
        "GJ_button_01.png",
    );
    label(
        commands,
        root,
        "Select Song",
        FontKind::Gold,
        0.85,
        center(0., 140., 350., 28.),
    );
    let song = if editor.level.music.contains("BackOnTrack") {
        "02: Back On Track"
    } else if editor.level.music.contains("Polargeist") {
        "03: Polargeist"
    } else if editor.level.music.is_empty() {
        "No song selected"
    } else {
        "01: Stereo Madness"
    };
    label(
        commands,
        root,
        song,
        FontKind::Big,
        0.7,
        center(0., 177., 350., 25.),
    );
    for (x, delta) in [(-170., -1), (170., 1)] {
        let e = art_button(
            commands,
            root,
            art,
            "navArrowBtn_001.png",
            Action::EditorSong(delta),
            center(x, 177., 15., 22.),
            true,
            "Select song",
        );
        if delta < 0 {
            commands.entity(e).insert((
                super::menu::MirroredButton,
                UiTransform {
                    scale: Vec2::new(-1., 1.),
                    ..default()
                },
            ));
        }
    }
    for (label, action, x) in [
        ("New", Action::New, -150.),
        ("Open", Action::Open, -50.),
        ("Save As", Action::SaveAs, 50.),
        ("Music", Action::Music, 150.),
    ] {
        long_button(
            commands,
            root,
            art,
            label,
            action,
            (x, 225., 85.),
            "GJ_button_04.png",
        );
    }
    long_button(
        commands,
        root,
        art,
        "OK",
        Action::EditorFiles,
        (0., 280., 70.),
        "GJ_button_01.png",
    );
    status(commands, root, &game.status, 257.);
}
fn build_start_settings(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let Some(editor) = &game.editor else {
        return;
    };
    let Some(object) = editor.selected.and_then(|i| editor.level.objects.get(i)) else {
        return;
    };
    let root = modal(commands, art, "Start Pos");
    for (key, action, y) in [
        ("mode", Action::StartMode, 115.),
        ("gravity", Action::StartGravity, 185.),
    ] {
        let value = object
            .extra
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("auto");
        label(
            commands,
            root,
            format!("{key}: {value}"),
            FontKind::Big,
            0.7,
            center(0., y - 25., 350., 22.),
        );
        long_button(
            commands,
            root,
            art,
            "Change",
            action,
            (0., y + 8., 130.),
            "GJ_button_01.png",
        );
    }
    long_button(
        commands,
        root,
        art,
        "OK",
        Action::EditorSelectionSettings,
        (0., 280., 70.),
        "GJ_button_01.png",
    );
}
fn volume(
    commands: &mut Commands,
    root: Entity,
    art: &NativeArt,
    channel: MusicChannel,
    value: u8,
    x: f32,
) {
    label(
        commands,
        root,
        match channel {
            MusicChannel::Level => "Music",
            MusicChannel::EditorSfx => "Editor SFX",
            _ => "SFX",
        },
        FontKind::Big,
        0.5,
        center(x, 278., 120., 20.),
    );
    let slider = add(
        commands,
        root,
        (
            Button,
            MusicSlider(channel),
            RelativeCursorPosition::default(),
            center(x, 299., 105., 16.),
        ),
    );
    image(
        commands,
        slider,
        art,
        "slidergroove.png",
        Node {
            position_type: PositionType::Absolute,
            left: n(-6.),
            top: n(4.),
            width: n(117.),
            height: n(8.),
            ..default()
        },
    );
    let fill = image(
        commands,
        slider,
        art,
        "sliderBar.png",
        Node {
            position_type: PositionType::Absolute,
            left: n(0.),
            top: n(6.5),
            width: percent(value as f32),
            height: n(3.),
            ..default()
        },
    );
    commands.entity(fill).insert(VolumeVisual {
        channel,
        value,
        knob: false,
    });
    let knob = image(
        commands,
        slider,
        art,
        "sliderthumb.png",
        Node {
            position_type: PositionType::Absolute,
            left: percent(value as f32),
            top: n(-2.),
            width: n(20.),
            height: n(20.),
            margin: UiRect::left(n(-10.)),
            ..default()
        },
    );
    commands.entity(knob).insert(VolumeVisual {
        channel,
        value,
        knob: true,
    });
}
fn build_pause(commands: &mut Commands, game: &Game, art: &NativeArt) {
    let Some(editor) = &game.editor else {
        return;
    };
    let root = root(commands, 1500, 0.8);
    label(
        commands,
        root,
        format!("Objects: {}/40000", editor.level.objects.len()),
        FontKind::Gold,
        0.6,
        native::left(80., 13., 150., 15.),
    );
    label(
        commands,
        root,
        format!(
            "Length: {:.0} seconds ({})",
            editor.level.length() / crate::constants::SPEED,
            match editor.level.length() / crate::constants::SPEED {
                x if x < 10. => "Tiny",
                x if x < 30. => "Short",
                x if x < 60. => "Medium",
                x if x < 120. => "Long",
                _ => "XL",
            }
        ),
        FontKind::Gold,
        0.6,
        native::left(90., 33., 170., 15.),
    );
    for (label, action, y) in [
        ("Resume", Action::EditorPause, 76.),
        ("Save and Play", Action::EditorSavePlay, 118.),
        ("Save and Exit", Action::EditorSaveExit, 160.),
        ("Save", Action::Save, 202.),
        ("Exit", Action::ExitEditor, 244.),
    ] {
        let e = long_button(
            commands,
            root,
            art,
            label,
            action,
            (0., y, 195.),
            "editor_green.png",
        );
        commands.entity(e).insert(center(0., y, 195., 28.));
    }
    for (label, action, y) in [
        ("Settings", Action::Settings, 20.),
        ("Export", Action::SaveAs, 52.),
        (
            "Editor SFX",
            Action::Setting(super::settings::Toggle::EditorSfx),
            84.,
        ),
        (
            "Auto-Save",
            Action::Setting(super::settings::Toggle::EditorAutoSave),
            116.,
        ),
        ("Death Stats", Action::EditorHelp, 148.),
        ("Get Audio", Action::Music, 180.),
        ("Help", Action::EditorHelp, 212.),
    ] {
        let e = long_button(
            commands,
            root,
            art,
            "",
            action,
            (0., y, 80.),
            "editor_gray.png",
        );
        commands.entity(e).insert(right(50., y, 80., 26.));
        commands.entity(e).insert(Name::new(label));
        text(
            commands,
            e,
            label,
            FontKind::Big,
            0.55_f32.min(58. / art.text_width(label, FontKind::Big)),
            Node {
                position_type: PositionType::Absolute,
                left: n(4.),
                top: n(0.),
                width: n(57.),
                height: percent(100),
                ..default()
            },
        );
        let icon = match label {
            "Settings" => "GJ_sSettingsIcon_001.png",
            "Export" => "GJ_sExportIcon_001.png",
            "Editor SFX" => "GJ_sSFXIcon_001.png",
            "Auto-Save" => "GJ_sSaveIcon_001.png",
            "Death Stats" => "GJ_deleteIcon_001.png",
            "Get Audio" => "GJ_sAudioDLIcon_001.png",
            _ => "GJ_sHelpIcon_001.png",
        };
        image(
            commands,
            e,
            art,
            icon,
            Node {
                position_type: PositionType::Absolute,
                right: n(5.),
                top: n(5.5),
                width: n(15.),
                height: n(15.),
                ..default()
            },
        );
        if y == 148. {
            tip(commands, e, "Death statistics are not available", false);
        }
        if y == 116. || y == 84. {
            let enabled = if y == 116. {
                game.profile.settings.editor_auto_save
            } else {
                game.profile.settings.editor_sfx_volume > 0
            };
            tip(
                commands,
                e,
                if enabled { "Enabled" } else { "Disabled" },
                true,
            );
        }
    }
    volume(
        commands,
        root,
        art,
        MusicChannel::Level,
        game.profile.settings.level_music_volume,
        -160.,
    );
    volume(
        commands,
        root,
        art,
        MusicChannel::Sfx,
        game.profile.settings.sfx_volume,
        0.,
    );
    volume(
        commands,
        root,
        art,
        MusicChannel::EditorSfx,
        game.profile.settings.editor_sfx_volume,
        160.,
    );
    sliced(
        commands,
        root,
        art,
        "editor_time.png",
        native::left(40., 160., 35., 145.),
        5.,
    );
    label(
        commands,
        root,
        "Time",
        FontKind::Gold,
        0.38,
        native::left(40., 100., 40., 15.),
    );
    let slider = add(
        commands,
        root,
        (
            Button,
            EditorTimeSlider,
            RelativeCursorPosition::default(),
            native::left(40., 146., 8., 75.),
        ),
    );
    let groove = image(
        commands,
        slider,
        art,
        "slidergroove.png",
        Node {
            position_type: PositionType::Absolute,
            left: n(-33.5),
            top: n(33.5),
            width: n(75.),
            height: n(8.),
            ..default()
        },
    );
    commands.entity(groove).insert(UiTransform {
        rotation: Rot2::radians(std::f32::consts::FRAC_PI_2),
        ..default()
    });
    add(
        commands,
        slider,
        (
            SpeedFill,
            Node {
                position_type: PositionType::Absolute,
                left: n(3.),
                bottom: n(0.),
                width: n(2.),
                height: percent(((editor.test_speed - 0.25) / 1.5 * 100.) as f32),
                ..default()
            },
            BackgroundColor(Color::srgb(0., 0.75, 1.)),
        ),
    );
    let knob = image(
        commands,
        slider,
        art,
        "sliderthumb.png",
        Node {
            position_type: PositionType::Absolute,
            left: n(-6.),
            top: percent(((1.75 - editor.test_speed) / 1.5 * 100.) as f32),
            width: n(20.),
            height: n(20.),
            margin: UiRect::top(n(-10.)),
            ..default()
        },
    );
    commands.entity(knob).insert(SpeedKnob);
    let caption = text(
        commands,
        root,
        format!("{:.1}x", editor.test_speed),
        FontKind::Big,
        0.38,
        native::left(40., 196., 40., 15.),
    );
    commands.entity(caption).insert(SpeedLabel);
    art_button(
        commands,
        root,
        art,
        "GJ_musicOnBtn_001.png",
        Action::Music,
        native::left(40., 219., 25., 25.),
        true,
        "Choose level audio",
    );
    status(commands, root, &game.status, 264.);
}
fn status(commands: &mut Commands, root: Entity, value: &str, y: f32) {
    if !value.is_empty() {
        add(
            commands,
            root,
            (
                Text::new(value),
                TextFont {
                    font_size: FontSize::Vh(1.65),
                    ..default()
                },
                TextColor(Color::srgb(1., 0.85, 0.4)),
                TextLayout::justify(Justify::Center),
                center(0., y, 350., 15.),
            ),
        );
    }
}
pub(super) fn sync_test_rate(game: Res<Game>, mut fixed: ResMut<Time<Fixed>>) {
    let speed = if game.testing {
        game.editor.as_ref().map_or(1., |e| e.test_speed)
    } else {
        1.
    };
    let hz = crate::constants::HZ * speed.clamp(0.25, 1.75);
    if (fixed.timestep().as_secs_f64() - 1. / hz).abs() > 1e-9 {
        fixed.set_timestep_hz(hz);
    }
}
pub(super) fn tooltip(
    game: Res<Game>,
    buttons: Query<(&Interaction, &Tooltip)>,
    mut labels: Query<&mut Text, With<TooltipText>>,
) {
    if game.screen != Screen::Editor {
        return;
    }
    let value = buttons
        .iter()
        .find(|(i, _)| **i != Interaction::None)
        .map(|(_, t)| t.0.as_str())
        .unwrap_or(&game.status);
    for mut label in &mut labels {
        if label.0 != value {
            label.0 = value.to_owned();
        }
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn controls(
    mut game: ResMut<Game>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursors: Query<&RelativeCursorPosition, With<EditorScrub>>,
    mut knobs: Query<&mut Node, With<ScrubThumb>>,
    speed_cursors: Query<&RelativeCursorPosition, With<EditorTimeSlider>>,
    mut speed_knobs: Query<&mut Node, (With<SpeedKnob>, Without<ScrubThumb>)>,
    mut speed_fill: Query<&mut Node, (With<SpeedFill>, Without<ScrubThumb>, Without<SpeedKnob>)>,
    mut speed_labels: Query<&mut native::BitmapText, With<SpeedLabel>>,
    time: Res<Time>,
    mut auto_save_elapsed: Local<f32>,
) {
    if game.screen != Screen::Editor
        || game.exit_dialog.is_some()
        || game.settings_open
        || game.prompt.is_some()
    {
        return;
    }
    if game.profile.settings.editor_auto_save && game.editor.as_ref().is_some_and(|e| e.dirty) {
        *auto_save_elapsed += time.delta_secs();
        if *auto_save_elapsed >= 30. {
            game.save_editor(false);
            *auto_save_elapsed = 0.;
        }
    } else {
        *auto_save_elapsed = 0.;
    }
    let Some(editor) = game.editor.as_mut() else {
        return;
    };
    if editor.pause_open {
        if mouse.pressed(MouseButton::Left)
            && let Some(cursor) = speed_cursors.iter().find(|c| c.cursor_over())
            && let Some(pos) = cursor.normalized
        {
            editor.test_speed = (1.75 - (pos.y as f64 + 0.5).clamp(0., 1.) * 1.5).clamp(0.25, 1.75);
        }
        for mut knob in &mut speed_knobs {
            knob.top = percent(((1.75 - editor.test_speed) / 1.5 * 100.) as f32);
        }
        for mut fill in &mut speed_fill {
            fill.height = percent(((editor.test_speed - 0.25) / 1.5 * 100.) as f32);
        }
        for mut label in &mut speed_labels {
            label.value = format!("{:.1}x", editor.test_speed);
        }
        return;
    }
    if editor.settings_open || editor.start_settings_open || editor.files_open {
        return;
    }
    let length = editor.level.length().max(3200.);
    if mouse.pressed(MouseButton::Left)
        && let Some(cursor) = cursors.iter().find(|c| c.cursor_over())
        && let Some(pos) = cursor.normalized
    {
        editor.camera.x = ((pos.x + 0.5).clamp(0., 1.) as f64) * length;
    }
    let percent_value = (editor.camera.x / length * 100.).clamp(0., 100.) as f32;
    for mut knob in &mut knobs {
        knob.left = percent(percent_value);
    }
}
/// Bounds are in the same 320-point coordinate system as the visible HUD.
pub(super) fn canvas_contains(position: Vec2, size: Vec2, modal: bool) -> bool {
    if modal {
        return false;
    }
    let scale = size.y / 320.;
    let p = position / scale;
    let width = size.x / scale;
    p.x >= 45. && p.x < width - 120. && p.y >= 42. && p.y < 210.
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_speed_changes_only_playtest_timing_and_controls_have_disjoint_access() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.edit();
        game.editor.as_mut().unwrap().test_speed = 0.5;
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<Time>()
            .init_resource::<Time<Fixed>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(PreUpdate, sync_test_rate)
            .add_systems(Update, controls);
        app.update();
        assert!(
            (app.world()
                .resource::<Time<Fixed>>()
                .timestep()
                .as_secs_f64()
                - 1. / 240.)
                .abs()
                < 1e-9
        );
        app.world_mut().resource_mut::<Game>().testing = true;
        app.update();
        assert!(
            (app.world()
                .resource::<Time<Fixed>>()
                .timestep()
                .as_secs_f64()
                - 1. / 120.)
                .abs()
                < 1e-9
        );
        app.world_mut().resource_mut::<Game>().testing = false;
        app.update();
        assert!(
            (app.world()
                .resource::<Time<Fixed>>()
                .timestep()
                .as_secs_f64()
                - 1. / 240.)
                .abs()
                < 1e-9
        );
    }
    #[test]
    fn editor_controls_are_outside_the_world_input_region() {
        for size in [
            Vec2::new(960., 640.),
            Vec2::new(1280., 720.),
            Vec2::new(1920., 1080.),
        ] {
            let s = size.y / 320.;
            assert!(canvas_contains(Vec2::new(150., 100.) * s, size, false));
            for p in [
                Vec2::new(25., 65.) * s,
                Vec2::new(size.x - 25. * s, 61. * s),
                Vec2::new(size.x / 2., size.y - 50. * s),
                Vec2::new(size.x / 2., 20. * s),
            ] {
                assert!(!canvas_contains(p, size, false));
            }
            assert!(!canvas_contains(size / 2., size, true));
        }
    }
    #[test]
    fn native_palette_and_toolbars_build_for_supported_categories() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.edit();
        let mut app = App::new();
        app.insert_resource(game);
        for category in 0..4 {
            app.world_mut()
                .resource_mut::<Game>()
                .editor
                .as_mut()
                .unwrap()
                .category = category;
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let mut commands = Commands::new(&mut queue, app.world());
            build(
                &mut commands,
                app.world().resource::<Game>(),
                &EditorArt::default(),
                &NativeArt::default(),
            );
            queue.apply(app.world_mut());
        }
        assert!(app.world_mut().query::<&Button>().iter(app.world()).count() > 100);
    }
}
