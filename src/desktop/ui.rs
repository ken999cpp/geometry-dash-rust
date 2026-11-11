use super::{ExitTarget, Field, Game, Screen};
use crate::{
    editor::{Editor, EditorMode},
    level::Level,
};
use bevy::prelude::*;

pub struct InterfacePlugin;
#[derive(Component)]
pub(super) struct UiRoot;
#[derive(Component)]
struct Hud;
#[derive(Resource, Default)]
struct DialogFont(Handle<Font>);

fn load_dialog_font(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(DialogFont(server.load("fonts/LiberationSans-Regular.ttf")));
}
#[derive(Component, Clone, Copy)]
pub(super) enum Action {
    Levels,
    Select(usize),
    PreviousLevel,
    NextLevel,
    BackLevels,
    Play,
    Edit,
    New,
    Menu,
    Resume,
    Restart,
    Practice,
    Icons,
    Icon(usize),
    Save,
    SaveAs,
    Open,
    Music,
    Background,
    Name,
    Rotation,
    Trigger,
    Test,
    Tool(usize),
    Grid,
    Undo,
    Delete,
    Duplicate,
    Redo,
    EditorMode(EditorMode),
    EditorCategory(usize),
    EditorSettings,
    BlockTexture(usize),
    Rotate(f64),
    Move(f64, f64),
    ZoomIn,
    ZoomOut,
    ResetView,
    Snap,
    ExitEditor,
    CancelExit,
    ConfirmExit,
    CancelPrompt,
    Pause,
    Settings,
    CloseSettings,
    NoDeathEffect,
    HideCheckpointButtons,
    VolumeStep(crate::storage::MusicChannel, i16),
    AddCheckpoint,
    RemoveCheckpoint,
}
impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::menu::MenuArt>()
            .init_resource::<super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .add_systems(
                Startup,
                (
                    super::menu::load_art,
                    super::editor_ui::load_art,
                    load_dialog_font,
                    setup_attempt_overlay,
                ),
            )
            .add_systems(
                Update,
                (
                    buttons,
                    super::settings::sliders,
                    rebuild,
                    hud,
                    super::editor_ui::tooltip,
                    update_attempt_overlay,
                )
                    .chain()
                    .in_set(super::FrameSet::Interface),
            );
    }
}
fn label(commands: &mut Commands, parent: Entity, text: impl Into<String>, size: f32) -> Entity {
    let entity = commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}
fn panel(commands: &mut Commands, node: Node) -> Entity {
    commands
        .spawn((
            UiRoot,
            node,
            BackgroundColor(Color::srgba(0.02, 0.04, 0.09, 0.92)),
        ))
        .id()
}
fn rebuild(
    mut commands: Commands,
    mut game: ResMut<Game>,
    roots: Query<Entity, With<UiRoot>>,
    art: Res<super::menu::MenuArt>,
    editor_art: Res<super::editor_ui::EditorArt>,
    dialog_font: Res<DialogFont>,
) {
    if !game.ui_dirty {
        return;
    }
    game.ui_dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if game.exit_dialog.is_some() {
        build_exit_dialog(&mut commands, &game, &dialog_font.0);
    }
    if game.settings_open {
        super::settings::build(&mut commands, &game, &dialog_font.0);
    }
    if let Some(prompt) = &game.prompt {
        let root = panel(
            &mut commands,
            Node {
                position_type: PositionType::Absolute,
                left: percent(10),
                top: percent(30),
                width: percent(80),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(24)),
                row_gap: px(14),
                ..default()
            },
        );
        label(&mut commands, root, prompt.label, 24.);
        label(&mut commands, root, format!("{}▏", prompt.value), 19.);
        label(
            &mut commands,
            root,
            "Enter: apply   Ctrl+A: select all",
            15.,
        );
        label(&mut commands, root, &game.status, 15.);
        let cancel = commands
            .spawn((
                Button,
                Action::CancelPrompt,
                Name::new("Cancel input"),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(12),
                    top: px(12),
                    width: px(36),
                    height: px(36),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.1, 0.1)),
            ))
            .id();
        super::editor_icons::draw(&mut commands, cancel, super::editor_icons::Icon::Exit);
        commands.entity(root).add_child(cancel);
        return;
    }
    if game.screen == Screen::Playing {
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
        let text = commands
            .spawn((
                Hud,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(18),
                    top: px(14),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(text);
        let pause = commands
            .spawn((
                Button,
                Action::Pause,
                Name::new("Pause (Esc / P)"),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(18),
                    top: px(14),
                    width: px(44),
                    height: px(44),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.1, 0.2, 0.8)),
            ))
            .id();
        super::editor_icons::draw(&mut commands, pause, super::editor_icons::Icon::Pause);
        commands.entity(root).add_child(pause);
        if game.simulation.as_ref().is_some_and(|sim| sim.practice)
            && !game.profile.settings.hide_checkpoint_buttons
        {
            for (left, action, icon, name) in [
                (
                    24.,
                    Action::AddCheckpoint,
                    super::editor_icons::Icon::Flag,
                    "Add checkpoint (Z)",
                ),
                (
                    94.,
                    Action::RemoveCheckpoint,
                    super::editor_icons::Icon::Trash,
                    "Remove checkpoint (X)",
                ),
            ] {
                let button = commands
                    .spawn((
                        Button,
                        action,
                        Name::new(name),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(left),
                            bottom: px(24),
                            width: px(52),
                            height: px(52),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.03, 0.1, 0.2, 0.8)),
                    ))
                    .id();
                super::editor_icons::draw(&mut commands, button, icon);
                commands.entity(root).add_child(button);
            }
        }
        return;
    }
    if game.screen == Screen::Editor {
        super::editor_ui::build(&mut commands, &game, &editor_art);
        return;
    }
    super::menu::build(&mut commands, &game, &art);
}

#[allow(clippy::type_complexity)]
fn buttons(
    mut game: ResMut<Game>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<
        (
            &Interaction,
            &Action,
            &mut BackgroundColor,
            Option<&super::menu::ImageButton>,
            Option<&mut UiTransform>,
        ),
        With<Button>,
    >,
) {
    let mut selected = None;
    for (interaction, action, mut color, image, transform) in &mut query {
        if let Some(image) = image {
            color.0 = image.background;
            if let Some(mut transform) = transform {
                transform.scale = Vec2::splat(match interaction {
                    Interaction::Pressed => 0.94,
                    Interaction::Hovered => 1.06,
                    Interaction::None => 1.,
                });
            }
        } else {
            color.0 = match interaction {
                Interaction::None => Color::srgb(0.08, 0.15, 0.28),
                Interaction::Hovered => Color::srgb(0.12, 0.3, 0.5),
                Interaction::Pressed => Color::srgb(0.18, 0.5, 0.65),
            };
        }
        if *interaction == Interaction::Pressed && mouse.just_pressed(MouseButton::Left) {
            selected = Some(*action);
        }
    }
    let Some(action) = selected else {
        return;
    };
    if game.exit_dialog.is_some() && !matches!(action, Action::CancelExit | Action::ConfirmExit) {
        return;
    }
    if game.settings_open
        && !matches!(
            action,
            Action::CloseSettings
                | Action::NoDeathEffect
                | Action::HideCheckpointButtons
                | Action::VolumeStep(..)
        )
    {
        return;
    }
    match action {
        Action::Settings => game.open_settings(),
        Action::CloseSettings => game.close_settings(),
        Action::NoDeathEffect | Action::HideCheckpointButtons => {
            let settings = &mut game.profile.settings;
            let value = match action {
                Action::NoDeathEffect => &mut settings.no_death_effect,
                _ => &mut settings.hide_checkpoint_buttons,
            };
            *value = !*value;
            game.persist();
            game.ui_dirty = true;
        }
        Action::VolumeStep(channel, step) => {
            let value =
                (i16::from(game.profile.settings.volume(channel)) + step).clamp(0, 100) as u8;
            game.profile.settings.set_volume(channel, value);
            game.persist();
        }
        Action::AddCheckpoint | Action::RemoveCheckpoint => {
            if game.attempt.playing()
                && !game.block_fixed_frame
                && let Some(sim) = game.simulation.as_mut().filter(|s| s.practice)
            {
                if matches!(action, Action::AddCheckpoint) {
                    sim.checkpoint();
                } else {
                    sim.remove_checkpoint();
                }
            }
        }
        Action::CancelExit => game.cancel_exit(),
        Action::ConfirmExit => game.confirm_dialog_exit(),
        Action::CancelPrompt => {
            game.prompt = None;
            game.confirm_exit = false;
            game.ui_dirty = true;
        }
        Action::Pause => game.screen(Screen::Paused),
        Action::Levels => {
            game.refresh();
            game.screen(Screen::Levels);
        }
        Action::PreviousLevel => game.browse_level(-1),
        Action::NextLevel => game.browse_level(1),
        Action::BackLevels => game.leave_level(),
        Action::Select(i) => {
            game.selected = i;
            game.ui_dirty = true;
        }
        Action::Play => game.start(false),
        Action::Edit => {
            if game.testing && game.editor.is_some() {
                game.screen(Screen::Editor);
            } else {
                game.edit();
            }
        }
        Action::New => {
            if game.editor.as_ref().is_some_and(|e| e.dirty) && !game.confirm_exit {
                game.confirm_exit = true;
                game.status = "Unsaved edits: click New again to discard, or save first".into();
                game.ui_dirty = true;
            } else {
                game.editor = Some(Editor::new(Level::empty("New Level"), None));
                game.screen(Screen::Editor);
            }
        }
        Action::Menu => {
            game.update_progress();
            game.persist();
            game.testing = false;
            game.screen(Screen::Menu);
        }
        Action::Resume => game.screen(Screen::Playing),
        Action::Restart => game.restart(),
        Action::Practice => {
            if let Some(s) = game.simulation.as_mut() {
                s.practice = !s.practice;
                s.checkpoints.clear();
                s.restart(false);
            }
            game.pending_seek = Some(0.);
            game.attempt.enter(super::attempt::Phase::Spawning);
            game.clear_trail = true;
            game.add_attempt();
            game.screen(Screen::Playing);
        }
        Action::Icons => game.screen(Screen::Icons),
        Action::Icon(i) => {
            game.profile.cube_icon = i;
            game.persist();
            game.ui_dirty = true;
        }
        Action::Save => game.save_editor(false),
        Action::SaveAs => game.save_editor(true),
        Action::Open => game.open_prompt(Field::Open),
        Action::Music => game.open_prompt(Field::Music),
        Action::Background => game.open_prompt(Field::Background),
        Action::Name => game.open_prompt(Field::Name),
        Action::Rotation => game.open_prompt(Field::Rotation),
        Action::Trigger => game.open_prompt(Field::Trigger),
        Action::Test => game.start(true),
        Action::ExitEditor => game.exit_editor(),
        other => {
            let mut changed = false;
            if let Some(editor) = game.editor.as_mut() {
                let before = editor.revision;
                match other {
                    Action::Tool(i) => editor.choose_tool(i),
                    Action::EditorMode(mode) => editor.set_mode(mode),
                    Action::EditorCategory(category) => editor.category = category,
                    Action::EditorSettings => editor.settings_open = !editor.settings_open,
                    Action::BlockTexture(n) => {
                        editor.texture = n;
                        editor.choose_tool(1);
                        editor.category = 0;
                        editor.mode = EditorMode::Build;
                    }
                    Action::Grid => editor.grid = !editor.grid,
                    Action::Snap => editor.snap_to_grid = !editor.snap_to_grid,
                    Action::Undo => editor.undo(),
                    Action::Redo => editor.redo(),
                    Action::Delete => editor.delete(),
                    Action::Duplicate => editor.duplicate(),
                    Action::Rotate(degrees) => editor.rotate(degrees),
                    Action::Move(x, y) => {
                        editor.move_selected(crate::geometry::Point::new(x, y), true)
                    }
                    Action::ZoomIn => editor.zoom = (editor.zoom * 1.2).min(10.),
                    Action::ZoomOut => editor.zoom = (editor.zoom / 1.2).max(0.1),
                    Action::ResetView => {
                        editor.zoom = 1.;
                        editor.camera = crate::geometry::Point::new(1200., 640.);
                    }
                    _ => {}
                }
                changed = editor.revision != before;
            }
            game.ui_dirty = true;
            game.scene_dirty |= changed;
            if changed {
                game.confirm_exit = false;
            }
        }
    }
}
fn hud(
    game: Res<Game>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut text: Query<&mut Text, With<Hud>>,
) {
    *timer += time.delta_secs();
    if *timer < 0.1 {
        return;
    }
    *timer = 0.;
    let Some(sim) = &game.simulation else {
        return;
    };
    for mut text in &mut text {
        text.0 = format!(
            "{}  ·  {}%  ·  {}  ·  {:.0} FPS\nEsc / P: pause   R: restart{}{}",
            sim.level.name,
            sim.progress(),
            sim.state.player.mode.name(),
            1. / time.delta_secs().max(0.0001),
            if sim.practice {
                "   Z/X: checkpoints"
            } else {
                ""
            },
            if game.status.is_empty() {
                String::new()
            } else {
                format!("\n{}", game.status)
            }
        );
    }
}

#[derive(Component)]
struct AttemptOverlay;
fn setup_attempt_overlay(mut commands: Commands) {
    commands.spawn((
        AttemptOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(1000),
        bevy::ui::FocusPolicy::Pass,
    ));
}
fn update_attempt_overlay(
    game: Res<Game>,
    mut overlays: Query<&mut BackgroundColor, With<AttemptOverlay>>,
) {
    let alpha = if game.screen == Screen::Playing {
        game.attempt.fade()
    } else {
        0.
    };
    for mut color in &mut overlays {
        color.0 = Color::srgba(0., 0., 0., alpha);
    }
}

fn build_exit_dialog(commands: &mut Commands, game: &Game, font: &Handle<Font>) {
    let root = commands
        .spawn((
            UiRoot,
            GlobalZIndex(2000),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                width: px(560),
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                row_gap: px(22),
                align_items: AlignItems::Center,
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(14)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.14, 0.25)),
            BorderColor::all(Color::srgb(0.5, 0.8, 1.)),
        ))
        .id();
    commands.entity(root).add_child(panel);
    let title = if game.exit_dialog == Some(ExitTarget::Editor) {
        "Выйти из редактора?"
    } else {
        "Вы уверены, что хотите выйти?"
    };
    dialog_label(commands, panel, title, 25., font);
    dialog_label(commands, panel, "Enter — выйти, Esc — отменить", 16., font);
    if game.editor.as_ref().is_some_and(|e| e.dirty) {
        dialog_label(
            commands,
            panel,
            "Несохранённые изменения уровня будут потеряны.",
            16.,
            font,
        );
    }
    let row = commands
        .spawn(Node {
            column_gap: px(24),
            ..default()
        })
        .id();
    commands.entity(panel).add_child(row);
    for (text, action, color) in [
        ("Отменить", Action::CancelExit, Color::srgb(0.1, 0.4, 0.15)),
        ("Выйти", Action::ConfirmExit, Color::srgb(0.55, 0.12, 0.08)),
    ] {
        let button = commands
            .spawn((
                Button,
                action,
                Node {
                    width: px(190),
                    height: px(52),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BackgroundColor(color),
                super::menu::ImageButton { background: color },
            ))
            .id();
        dialog_label(commands, button, text, 22., font);
        commands.entity(row).add_child(button);
    }
}

pub(super) fn dialog_label(
    commands: &mut Commands,
    parent: Entity,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    let entity = label(commands, parent, text, size);
    commands.entity(entity).insert(TextFont {
        font: font.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn click_button(app: &mut App, matches_action: impl Fn(&Action) -> bool) {
        for mut interaction in app
            .world_mut()
            .query::<&mut Interaction>()
            .iter_mut(app.world_mut())
        {
            *interaction = Interaction::None;
        }
        let button = app
            .world_mut()
            .query::<(Entity, &Action)>()
            .iter(app.world())
            .find(|(_, action)| matches_action(action))
            .unwrap()
            .0;
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        {
            let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
            mouse.reset_all();
            mouse.press(MouseButton::Left);
        }
        app.update();
    }

    #[test]
    fn star_counter_is_always_visible_even_with_the_removed_legacy_option_disabled() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.profile =
            serde_json::from_value(serde_json::json!({"settings":{"show_star_counter":false}}))
                .unwrap();
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<super::super::menu::MenuArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .add_systems(Update, rebuild);
        for screen in [
            Screen::Menu,
            Screen::Levels,
            Screen::Icons,
            Screen::Paused,
            Screen::Complete,
        ] {
            app.world_mut().resource_mut::<Game>().screen(screen);
            app.update();
            let counters = app
                .world_mut()
                .query::<&Name>()
                .iter(app.world())
                .filter(|name| name.as_str() == "Star counter")
                .count();
            assert_eq!(counters, 1, "missing star counter on {screen:?}");
        }
    }

    #[test]
    fn settings_persist_and_checkpoint_buttons_follow_the_visibility_option() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-options-ui-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = crate::storage::Profile::default();
        let mut sim = crate::simulation::Simulation::new(Level::empty("Settings")).unwrap();
        sim.practice = true;
        sim.status = crate::simulation::Status::Dead(1.);
        game.simulation = Some(sim);
        game.attempt.enter(super::super::attempt::Phase::Dying);
        game.screen(Screen::Paused);
        game.open_settings();
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<super::super::menu::MenuArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(
                Update,
                (buttons, super::super::settings::sliders, rebuild).chain(),
            );
        app.update();
        click_button(&mut app, |a| matches!(a, Action::NoDeathEffect));
        click_button(&mut app, |a| matches!(a, Action::HideCheckpointButtons));
        click_button(&mut app, |a| {
            matches!(
                a,
                Action::VolumeStep(crate::storage::MusicChannel::Level, -5)
            )
        });
        // Underlying pause actions are blocked by the settings modal.
        click_button(&mut app, |a| matches!(a, Action::Resume));
        assert_eq!(app.world().resource::<Game>().screen, Screen::Paused);
        click_button(&mut app, |a| matches!(a, Action::CloseSettings));
        let saved = crate::storage::Profile::load(&root).unwrap();
        assert!(saved.settings.no_death_effect);
        assert!(saved.settings.hide_checkpoint_buttons);
        assert_eq!(saved.settings.level_music_volume, 95);
        let game = app.world().resource::<Game>();
        assert!(!game.settings_open);
        assert_eq!(game.attempt.phase, super::super::attempt::Phase::Dying);
        assert_eq!(game.attempt.elapsed, 0.);
        assert_eq!(
            game.simulation.as_ref().unwrap().status,
            crate::simulation::Status::Dead(1.)
        );
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.attempt.enter(super::super::attempt::Phase::Playing);
            game.simulation.as_mut().unwrap().status = crate::simulation::Status::Running;
            game.screen(Screen::Playing);
        }
        app.update();
        let checkpoint_buttons = app
            .world_mut()
            .query::<&Action>()
            .iter(app.world())
            .filter(|a| matches!(a, Action::AddCheckpoint | Action::RemoveCheckpoint))
            .count();
        assert_eq!(checkpoint_buttons, 0);
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.profile.settings.hide_checkpoint_buttons = false;
            game.ui_dirty = true;
        }
        app.update();
        click_button(&mut app, |a| matches!(a, Action::AddCheckpoint));
        assert_eq!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .checkpoints
                .len(),
            1
        );
        click_button(&mut app, |a| matches!(a, Action::RemoveCheckpoint));
        assert!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .checkpoints
                .is_empty()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn editor_exit_requires_confirmation_for_clean_and_dirty_levels() {
        for dirty in [false, true] {
            let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
            game.edit();
            game.editor.as_mut().unwrap().dirty = dirty;
            let original = game.editor.as_ref().unwrap().level.clone();
            game.escape();
            let mut app = App::new();
            app.insert_resource(game)
                .init_resource::<super::super::menu::MenuArt>()
                .init_resource::<super::super::editor_ui::EditorArt>()
                .init_resource::<DialogFont>()
                .init_resource::<ButtonInput<MouseButton>>()
                .add_systems(Update, (buttons, rebuild).chain());
            app.update();
            let game = app.world().resource::<Game>();
            assert_eq!(game.screen, Screen::Editor);
            assert_eq!(game.exit_dialog, Some(ExitTarget::Editor));
            let labels = app
                .world_mut()
                .query::<&Text>()
                .iter(app.world())
                .map(|t| t.0.as_str())
                .collect::<Vec<_>>();
            assert!(labels.contains(&"Выйти из редактора?"));
            assert_eq!(
                labels.contains(&"Несохранённые изменения уровня будут потеряны."),
                dirty
            );

            // Escape dismisses the editor dialog instead of discarding the level.
            app.world_mut().resource_mut::<Game>().escape();
            app.update();
            let game = app.world().resource::<Game>();
            assert!(game.exit_dialog.is_none());
            assert_eq!(game.screen, Screen::Editor);
            assert_eq!(game.editor.as_ref().unwrap().level, original);

            click_button(&mut app, |action| matches!(action, Action::ExitEditor));
            click_button(&mut app, |action| matches!(action, Action::CancelExit));
            let game = app.world().resource::<Game>();
            assert_eq!(game.screen, Screen::Editor);
            assert!(game.exit_dialog.is_none());
            assert_eq!(game.editor.as_ref().unwrap().dirty, dirty);
            assert_eq!(game.editor.as_ref().unwrap().level, original);

            click_button(&mut app, |action| matches!(action, Action::ExitEditor));
            click_button(&mut app, |action| matches!(action, Action::ConfirmExit));
            let game = app.world().resource::<Game>();
            assert_eq!(game.screen, Screen::Menu);
            assert!(game.editor.is_none());
            assert!(game.exit_dialog.is_none());
            assert!(!game.exit_requested);
        }
    }

    #[test]
    fn exit_dialog_buttons_cancel_to_main_menu_and_confirm_exit() {
        let mut app = App::new();
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.edit();
        game.editor.as_mut().unwrap().dirty = true;
        let original = game.editor.as_ref().unwrap().level.clone();
        game.screen(Screen::Menu);
        game.escape();
        app.insert_resource(game)
            .init_resource::<super::super::menu::MenuArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Update, (buttons, rebuild).chain());
        app.update();
        let labels = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"Отменить"));
        assert!(labels.contains(&"Выйти"));
        let cancel = app
            .world_mut()
            .query::<(Entity, &Action)>()
            .iter(app.world())
            .find(|(_, action)| matches!(action, Action::CancelExit))
            .unwrap()
            .0;
        *app.world_mut().get_mut::<Interaction>(cancel).unwrap() = Interaction::Pressed;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        let game = app.world().resource::<Game>();
        assert!(game.exit_dialog.is_none());
        assert_eq!(game.screen, Screen::Menu);
        assert!(game.editor.as_ref().unwrap().dirty);
        assert_eq!(game.editor.as_ref().unwrap().level, original);
        app.world_mut().resource_mut::<Game>().escape();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.update();
        let confirm = app
            .world_mut()
            .query::<(Entity, &Action)>()
            .iter(app.world())
            .find(|(_, action)| matches!(action, Action::ConfirmExit))
            .unwrap()
            .0;
        *app.world_mut().get_mut::<Interaction>(confirm).unwrap() = Interaction::Pressed;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().resource::<Game>().exit_requested);
    }
    #[test]
    fn menus_and_editor_build_without_a_window() {
        let mut app = App::new();
        app.insert_resource(Game::new(std::path::PathBuf::from(env!(
            "CARGO_MANIFEST_DIR"
        ))))
        .init_resource::<super::super::menu::MenuArt>()
        .init_resource::<super::super::editor_ui::EditorArt>()
        .init_resource::<DialogFont>()
        .add_systems(Update, rebuild);
        for screen in [Screen::Menu, Screen::Levels, Screen::Icons, Screen::Loading] {
            app.world_mut().resource_mut::<Game>().screen(screen);
            app.update();
            let mut query = app.world_mut().query_filtered::<Entity, With<UiRoot>>();
            assert_eq!(query.iter(app.world()).count(), 1);
        }
        app.world_mut().resource_mut::<Game>().edit();
        app.update();
        let mut query = app.world_mut().query::<&Button>();
        assert!(query.iter(app.world()).count() > 25);
    }
}
