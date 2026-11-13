use super::{ExitTarget, Field, Game, Screen};
use crate::{
    editor::{Editor, EditorMode},
    level::Level,
};
use bevy::prelude::*;

pub struct InterfacePlugin;
#[derive(Component)]
pub(super) struct UiRoot;
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
    EditorFiles,
    LevelColor(bool),
    EditorPause,
    EditorView,
    EditorSwipe,
    EditorRotateObjects,
    EditorCopy,
    EditorPaste,
    EditorCopyState,
    EditorPasteState(bool),
    EditorFlip(bool),
    EditorDeselect,
    EditorPalettePage(i32),
    EditorSavePlay,
    EditorSaveExit,
    EditorSelectionSettings,
    EditorSong(i32),
    EditorHelp,
    EditorMoveStep(f64),
    EditorNudge(f64, f64),
    EditorCustomStep,
    EditorDeleteAll(bool),
    EditorDeleteFilter(u8),
    ObjectScale,
    ObjectGroups,
    ObjectLayer,
    EditorLayer(i32),
    EditorAllLayers,
    EditorGoToLayer,
    BlockTexture(usize),
    Rotate(f64),
    ZoomIn,
    ZoomOut,
    Snap,
    ExitEditor,
    CancelExit,
    ConfirmExit,
    CancelPrompt,
    Pause,
    Settings,
    CloseSettings,
    SettingsPage(super::settings::Page),
    SettingsBack,
    Setting(super::settings::Toggle),
    VideoFullscreen,
    VideoBorderless,
    VideoResolution(i32),
    VideoApply,
    NoDeathEffect,
    HideCheckpointButtons,
    FpsLimit(u32),
    AddCheckpoint,
    RemoveCheckpoint,
    StartMode,
    StartGravity,
    ExitApplication,
    Summary(&'static str),
    CloseSummary,
    HidePause,
    External(&'static str),
}
impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::menu::MenuArt>()
            .init_resource::<super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<super::native_menu::NativeArt>()
            .init_resource::<super::frame_rate::FrameTiming>()
            .add_systems(
                Startup,
                (
                    super::menu::load_art,
                    super::editor_ui::load_art,
                    load_dialog_font,
                    super::native_menu::load,
                    setup_attempt_overlay,
                ),
            )
            .add_systems(
                Update,
                (
                    buttons,
                    super::settings::sliders,
                    super::editor_ui::controls,
                    super::settings::sync_window,
                    rebuild,
                    super::settings::scale_slices,
                    hud,
                    super::native_menu::refresh_text,
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
    native_art: Res<super::native_menu::NativeArt>,
) {
    if !game.ui_dirty {
        return;
    }
    game.ui_dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if matches!(
        game.screen,
        Screen::Menu | Screen::Levels | Screen::Paused | Screen::Playing
    ) {
        if game.screen == Screen::Playing {
            super::native_menu::build_hud(&mut commands, &game, &native_art);
        } else if !game.pause_hidden {
            super::native_menu::build(&mut commands, &game, &native_art);
        }
        if let Some(title) = game.menu_panel {
            super::native_menu::summary(&mut commands, &game, &native_art, title);
        }
        if game.exit_dialog.is_some() {
            build_exit_dialog(&mut commands, &game, &dialog_font.0, &native_art);
        }
        if game.settings_open {
            super::settings::build(&mut commands, &game, &native_art);
        }
        return;
    }
    if game.exit_dialog.is_some() {
        build_exit_dialog(&mut commands, &game, &dialog_font.0, &native_art);
    }
    if game.settings_open {
        super::settings::build(&mut commands, &game, &native_art);
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
        dialog_label(&mut commands, root, prompt.label, 24., &dialog_font.0);
        dialog_label(
            &mut commands,
            root,
            &format!("{}▏", prompt.value),
            19.,
            &dialog_font.0,
        );
        dialog_label(
            &mut commands,
            root,
            "Enter: apply   Ctrl+A: select all",
            15.,
            &dialog_font.0,
        );
        dialog_label(&mut commands, root, &game.status, 15., &dialog_font.0);
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
        super::native_menu::image(
            &mut commands,
            cancel,
            &native_art,
            "GJ_closeBtn_001.png",
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        );
        commands.entity(root).add_child(cancel);
        return;
    }
    if game.screen == Screen::Editor {
        super::editor_ui::build(&mut commands, &game, &editor_art, &native_art);
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
            Option<&super::menu::MirroredButton>,
        ),
        (With<Button>, Without<bevy::ui::InteractionDisabled>),
    >,
) {
    let mut selected = None;
    for (interaction, action, mut color, image, transform, mirrored) in &mut query {
        if let Action::FpsLimit(fps) = *action {
            if !game.display_rate.supports(fps) {
                color.0 = Color::srgb(0.1, 0.11, 0.14);
                continue;
            }
            if *interaction == Interaction::Pressed && mouse.just_pressed(MouseButton::Left) {
                selected = Some(*action);
            }
            color.0 = if fps
                == game
                    .display_rate
                    .selected_fps(game.profile.settings.fps_limit())
            {
                Color::srgb(0.12, 0.5, 0.22)
            } else if *interaction == Interaction::Hovered {
                Color::srgb(0.12, 0.3, 0.5)
            } else {
                Color::srgb(0.08, 0.15, 0.28)
            };
            continue;
        }
        if let Some(image) = image {
            color.0 = image.background;
            if let Some(mut transform) = transform {
                transform.scale = Vec2::splat(match interaction {
                    Interaction::Pressed => 0.94,
                    Interaction::Hovered => 1.06,
                    Interaction::None => 1.,
                });
                if mirrored.is_some() {
                    transform.scale.x = -transform.scale.x;
                }
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
    if game.menu_panel.is_some() && !matches!(action, Action::CloseSummary) {
        return;
    }
    if game.exit_dialog.is_some() && !matches!(action, Action::CancelExit | Action::ConfirmExit) {
        return;
    }
    if game.settings_open
        && !matches!(
            action,
            Action::CloseSettings
                | Action::SettingsPage(..)
                | Action::SettingsBack
                | Action::Setting(..)
                | Action::VideoFullscreen
                | Action::VideoBorderless
                | Action::VideoResolution(..)
                | Action::VideoApply
                | Action::NoDeathEffect
                | Action::HideCheckpointButtons
                | Action::FpsLimit(..)
        )
    {
        return;
    }
    if game.prompt.is_some() && !matches!(action, Action::CancelPrompt) {
        return;
    }
    if !game.settings_open
        && let Some(editor) = &game.editor
    {
        let allowed = if editor.start_settings_open {
            matches!(
                action,
                Action::StartMode | Action::StartGravity | Action::EditorSelectionSettings
            )
        } else if editor.files_open {
            matches!(
                action,
                Action::EditorFiles
                    | Action::New
                    | Action::Open
                    | Action::SaveAs
                    | Action::Name
                    | Action::Background
                    | Action::Music
            )
        } else if editor.settings_open {
            matches!(
                action,
                Action::EditorSettings
                    | Action::New
                    | Action::Open
                    | Action::SaveAs
                    | Action::Name
                    | Action::Background
                    | Action::Music
                    | Action::EditorSong(..)
                    | Action::EditorFiles
                    | Action::LevelColor(..)
            )
        } else if editor.pause_open {
            matches!(
                action,
                Action::EditorPause
                    | Action::EditorSavePlay
                    | Action::EditorSaveExit
                    | Action::Save
                    | Action::ExitEditor
                    | Action::Settings
                    | Action::SaveAs
                    | Action::Music
                    | Action::EditorHelp
                    | Action::Setting(..)
                    | Action::CancelExit
                    | Action::ConfirmExit
            )
        } else {
            true
        };
        if !allowed {
            return;
        }
    }
    match action {
        Action::Settings => game.open_settings(),
        Action::CloseSettings => game.close_settings(),
        Action::SettingsBack => game.settings_back(),
        Action::SettingsPage(page) => {
            if page == super::settings::Page::Graphics {
                game.settings_video = super::settings::VideoOptions::from(&game.profile.settings);
            }
            game.settings_page = page;
            game.ui_dirty = true;
        }
        Action::Setting(setting) => {
            setting.flip(&mut game.profile.settings);
            game.persist();
            game.ui_dirty = true;
        }
        Action::VideoFullscreen => {
            game.settings_video.fullscreen = !game.settings_video.fullscreen;
            game.ui_dirty = true;
        }
        Action::VideoBorderless => {
            game.settings_video.borderless = !game.settings_video.borderless;
            game.ui_dirty = true;
        }
        Action::VideoResolution(delta) => {
            game.settings_video.resolution = (game.settings_video.resolution as i32 + delta)
                .rem_euclid(super::settings::RESOLUTIONS.len() as i32)
                as usize;
            game.ui_dirty = true;
        }
        Action::VideoApply => {
            let video = game.settings_video;
            game.profile.settings.fullscreen = video.fullscreen;
            game.profile.settings.borderless = video.borderless;
            game.profile.settings.resolution = video.resolution;
            game.persist();
            game.settings_back();
        }
        Action::FpsLimit(fps) => {
            if game.settings_open && game.display_rate.supports(fps) {
                game.profile.settings.fps_limit = fps;
                game.persist();
                game.ui_dirty = true;
            }
        }
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
                game.leave_level();
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
            game.pending_seek = Some(game.simulation.as_ref().map_or(0., |s| s.state.elapsed));
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
        Action::Save => {
            game.save_editor(false);
        }
        Action::EditorSavePlay => game.save_editor_then(true, false),
        Action::EditorSaveExit => game.save_editor_then(false, true),
        Action::EditorPause => {
            if let Some(editor) = game.editor.as_mut() {
                editor.pause_open = !editor.pause_open;
            }
            game.ui_dirty = true;
        }
        Action::EditorSelectionSettings => {
            if game.editor.as_ref().is_some_and(|e| {
                e.selected
                    .is_some_and(|i| e.level.objects[i].kind == "start_pos")
            }) {
                let editor = game.editor.as_mut().unwrap();
                editor.start_settings_open = !editor.start_settings_open;
                game.ui_dirty = true;
            } else {
                game.open_prompt(Field::Trigger);
            }
        }
        Action::EditorHelp => {
            game.open_settings();
            game.settings_page = super::settings::Page::Options(super::settings::Tab::Keys);
        }
        Action::EditorSong(delta) => {
            let songs = ["StereoMadness", "BackOnTrack", "Polargeist"];
            if let Some(editor) = game.editor.as_mut() {
                let current = songs
                    .iter()
                    .position(|s| editor.level.music.contains(s))
                    .unwrap_or(if delta >= 0 { 2 } else { 0 });
                let next = (current as i32 + delta).rem_euclid(songs.len() as i32) as usize;
                editor.remember();
                editor.level.music = format!("assets/music/{}.mp3", songs[next]);
            }
            game.ui_dirty = true;
        }
        Action::SaveAs => {
            game.save_editor(true);
        }
        Action::Open => game.open_prompt(Field::Open),
        Action::Music => game.open_prompt(Field::Music),
        Action::Background => game.open_prompt(Field::Background),
        Action::LevelColor(ground) => game.open_prompt(Field::LevelColor(ground)),
        Action::Name => game.open_prompt(Field::Name),
        Action::Rotation => game.open_prompt(Field::Rotation),
        Action::ObjectScale => game.open_prompt(Field::ObjectScale),
        Action::ObjectGroups => game.open_prompt(Field::ObjectGroups),
        Action::ObjectLayer => game.open_prompt(Field::EditorLayer),
        Action::EditorCustomStep => game.open_prompt(Field::EditorStep),
        Action::Trigger => game.open_prompt(Field::Trigger),
        Action::Test => game.start(true),
        Action::ExitApplication => game.escape(),
        Action::Summary(title) => {
            game.menu_panel = Some(title);
            game.ui_dirty = true;
        }
        Action::CloseSummary => {
            game.menu_panel = None;
            game.ui_dirty = true;
        }
        Action::HidePause => {
            game.pause_hidden = true;
            game.ui_dirty = true;
        }
        Action::External(url) => {
            super::native_menu::open_link(url);
        }
        Action::ExitEditor => game.exit_editor(),
        other => {
            let mut changed = false;
            if let Some(editor) = game.editor.as_mut() {
                let before = editor.revision;
                match other {
                    Action::StartMode => editor.cycle_start_setting(false),
                    Action::StartGravity => editor.cycle_start_setting(true),
                    Action::EditorView => editor.view_only = !editor.view_only,
                    Action::EditorSwipe => editor.swipe = !editor.swipe,
                    Action::EditorRotateObjects => editor.rotate_objects = !editor.rotate_objects,
                    Action::EditorCopy => editor.copy(),
                    Action::EditorPaste => editor.paste(),
                    Action::EditorCopyState => editor.copy_state(),
                    Action::EditorPasteState(color_only) => editor.paste_state(color_only),
                    Action::EditorFlip(horizontal) => editor.flip(horizontal),
                    Action::EditorDeselect => editor.deselect(),
                    Action::EditorMoveStep(step) => editor.move_step = step,
                    Action::EditorNudge(x, y) => editor.move_selected(
                        crate::geometry::Point::new(x * editor.move_step, y * editor.move_step),
                        true,
                    ),
                    Action::EditorDeleteAll(start) => editor.delete_all(start),
                    Action::EditorDeleteFilter(filter) => {
                        editor.delete_filter = match filter {
                            1 => Some("solid".into()),
                            2 => editor
                                .selected
                                .map(|i| editor.level.objects[i].kind.clone()),
                            _ => None,
                        }
                    }
                    Action::EditorLayer(delta) => {
                        editor.layer = (editor.layer + delta).clamp(0, 999);
                        editor.show_all_layers = false;
                        editor.deselect();
                        changed = true;
                    }
                    Action::EditorAllLayers => {
                        editor.show_all_layers = !editor.show_all_layers;
                        editor.deselect();
                        changed = true;
                    }
                    Action::EditorGoToLayer => {
                        if let Some(i) = editor.selected {
                            editor.layer = editor.level.objects[i]
                                .extra
                                .get("editor_layer")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32;
                            editor.show_all_layers = false;
                            changed = true;
                        }
                    }
                    Action::EditorPalettePage(delta) => {
                        editor.palette_page = (editor.palette_page as i32 + delta).max(0) as usize
                    }
                    Action::Tool(i) => editor.choose_tool(i),
                    Action::EditorMode(mode) => editor.set_mode(mode),
                    Action::EditorCategory(category) => {
                        editor.category = category;
                        editor.palette_page = 0;
                    }
                    Action::EditorSettings => editor.settings_open = !editor.settings_open,
                    Action::EditorFiles => editor.files_open = !editor.files_open,
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
                    Action::ZoomIn => editor.zoom = (editor.zoom * 1.2).min(10.),
                    Action::ZoomOut => editor.zoom = (editor.zoom / 1.2).max(0.1),
                    _ => {}
                }
                changed |= editor.revision != before;
            }
            game.ui_dirty = true;
            game.scene_dirty |= changed;
            if changed {
                game.confirm_exit = false;
            }
        }
    }
}
#[allow(clippy::type_complexity)]
fn hud(
    game: Res<Game>,
    time: Res<Time>,
    timing: Res<super::frame_rate::FrameTiming>,
    mut timer: Local<f32>,
    mut text: Query<(
        &mut super::native_menu::BitmapText,
        Option<&super::native_menu::HudProgress>,
        Option<&super::native_menu::HudFps>,
        Option<&super::native_menu::HudTime>,
    )>,
    mut bars: Query<&mut Node, With<super::native_menu::HudBar>>,
) {
    *timer += time.delta_secs();
    if *timer < 0.1 {
        return;
    }
    *timer = 0.;
    let Some(sim) = &game.simulation else {
        return;
    };
    for mut bar in &mut bars {
        bar.width = percent(sim.progress() as f32);
    }
    for (mut label, progress, fps, clock) in &mut text {
        let value = if progress.is_some() {
            Some(format!("{}%", sim.progress()))
        } else if fps.is_some() {
            Some(format!("{:.0} FPS", timing.fps()))
        } else if clock.is_some() {
            Some(format!("{:.2}", sim.state.elapsed))
        } else {
            None
        };
        if let Some(value) = value
            && label.value != value
        {
            label.value = value;
        }
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

fn build_exit_dialog(
    commands: &mut Commands,
    game: &Game,
    _font: &Handle<Font>,
    art: &super::native_menu::NativeArt,
) {
    match game.exit_dialog {
        Some(ExitTarget::Application) => super::native_menu::build_quit_dialog(commands, art),
        Some(ExitTarget::Editor) => super::native_menu::build_editor_exit_dialog(commands, art),
        None => {}
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
    fn fps_buttons_reject_unsupported_rates_and_persist_supported_choices() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-fps-ui-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = crate::storage::Profile::default();
        game.display_rate.refresh_rate_millihertz = Some(144_000);
        game.open_settings();
        game.settings_page = super::super::settings::Page::Advanced;
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<super::super::menu::MenuArt>()
            .init_resource::<super::super::native_menu::NativeArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Update, (buttons, rebuild).chain());
        app.update();
        let mut disabled = app
            .world_mut()
            .query::<(&Action, Option<&bevy::ui::InteractionDisabled>)>()
            .iter(app.world())
            .filter_map(|(action, disabled)| match action {
                Action::FpsLimit(fps) => Some((*fps, disabled.is_some())),
                _ => None,
            })
            .collect::<Vec<_>>();
        disabled.sort_unstable_by_key(|v| v.0);
        assert_eq!(
            disabled,
            vec![(60, false), (120, false), (144, false), (240, true)]
        );
        // Even a synthetic press on a disabled option must be rejected.
        click_button(&mut app, |a| matches!(a, Action::FpsLimit(240)));
        assert_eq!(
            app.world().resource::<Game>().profile.settings.fps_limit,
            60
        );
        for fps in [120, 144, 60] {
            click_button(
                &mut app,
                |a| matches!(a, Action::FpsLimit(value) if *value == fps),
            );
            assert_eq!(
                app.world().resource::<Game>().profile.settings.fps_limit,
                fps
            );
            assert_eq!(
                crate::storage::Profile::load(&root)
                    .unwrap()
                    .settings
                    .fps_limit,
                fps
            );
        }
        // Unknown monitor information also blocks settings above 60 FPS.
        app.world_mut().resource_mut::<Game>().display_rate = Default::default();
        click_button(&mut app, |a| matches!(a, Action::FpsLimit(120)));
        assert_eq!(
            app.world().resource::<Game>().profile.settings.fps_limit,
            60
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_screens_use_level_ratings_instead_of_an_extra_global_star_widget() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.profile =
            serde_json::from_value(serde_json::json!({"settings":{"show_star_counter":false}}))
                .unwrap();
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<super::super::menu::MenuArt>()
            .init_resource::<super::super::native_menu::NativeArt>()
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
            assert_eq!(
                counters,
                usize::from(matches!(screen, Screen::Icons | Screen::Complete)),
                "unexpected global star widget on {screen:?}"
            );
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
            .init_resource::<super::super::native_menu::NativeArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(
                Update,
                (buttons, super::super::settings::sliders, rebuild).chain(),
            );
        app.update();
        click_button(&mut app, |a| {
            matches!(
                a,
                Action::SettingsPage(super::super::settings::Page::Options(
                    super::super::settings::Tab::Gameplay
                ))
            )
        });
        click_button(&mut app, |a| {
            matches!(
                a,
                Action::SettingsPage(super::super::settings::Page::Options(
                    super::super::settings::Tab::Visual
                ))
            )
        });
        click_button(&mut app, |a| matches!(a, Action::NoDeathEffect));
        click_button(&mut app, |a| {
            matches!(
                a,
                Action::SettingsPage(super::super::settings::Page::Options(
                    super::super::settings::Tab::Practice
                ))
            )
        });
        click_button(&mut app, |a| matches!(a, Action::HideCheckpointButtons));
        click_button(&mut app, |a| matches!(a, Action::SettingsBack));
        let slider = app
            .world_mut()
            .query::<(Entity, &super::super::settings::MusicSlider)>()
            .iter(app.world())
            .find(|(_, s)| s.0 == crate::storage::MusicChannel::Level)
            .unwrap()
            .0;
        *app.world_mut().get_mut::<Interaction>(slider).unwrap() = Interaction::Pressed;
        app.world_mut()
            .get_mut::<bevy::ui::RelativeCursorPosition>(slider)
            .unwrap()
            .normalized = Some(Vec2::new(0.45, 0.));
        {
            let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
            mouse.reset_all();
            mouse.press(MouseButton::Left);
        }
        app.update();
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
                .init_resource::<super::super::native_menu::NativeArt>()
                .init_resource::<super::super::editor_ui::EditorArt>()
                .init_resource::<DialogFont>()
                .init_resource::<ButtonInput<MouseButton>>()
                .add_systems(Update, (buttons, rebuild).chain());
            app.update();
            assert!(
                app.world()
                    .resource::<Game>()
                    .editor
                    .as_ref()
                    .unwrap()
                    .pause_open
            );
            assert!(app.world().resource::<Game>().exit_dialog.is_none());
            click_button(&mut app, |action| matches!(action, Action::ExitEditor));
            assert_eq!(
                app.world().resource::<Game>().exit_dialog,
                Some(ExitTarget::Editor)
            );
            let labels = app
                .world_mut()
                .query::<&super::super::native_menu::BitmapText>()
                .iter(app.world())
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>();
            assert!(labels.contains(&"Exit"));
            assert!(labels.contains(&"Exit without saving? All unsaved changes"));
            assert!(labels.contains(&"will be lost!"));
            assert!(labels.contains(&"No"));
            assert!(labels.contains(&"Yes"));

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
            .init_resource::<super::super::native_menu::NativeArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Update, (buttons, rebuild).chain());
        app.update();
        let labels = app
            .world_mut()
            .query::<&super::super::native_menu::BitmapText>()
            .iter(app.world())
            .map(|t| t.value.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"Quit Game"));
        assert!(labels.contains(&"Are you sure you want to quit?"));
        assert!(labels.contains(&"Cancel"));
        assert!(labels.contains(&"Yes"));
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
        .init_resource::<super::super::native_menu::NativeArt>()
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
    #[test]
    fn settings_video_changes_apply_explicitly_and_escape_returns_one_page() {
        use super::super::settings::{Page, Tab};
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-video-ui-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = crate::storage::Profile::default();
        game.screen(Screen::Paused);
        game.open_settings();
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<super::super::menu::MenuArt>()
            .init_resource::<super::super::native_menu::NativeArt>()
            .init_resource::<super::super::editor_ui::EditorArt>()
            .init_resource::<DialogFont>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Update, (buttons, rebuild).chain());
        app.update();
        click_button(&mut app, |a| {
            matches!(a, Action::SettingsPage(Page::Graphics))
        });
        click_button(&mut app, |a| matches!(a, Action::VideoFullscreen));
        assert!(!app.world().resource::<Game>().profile.settings.fullscreen);
        assert!(app.world().resource::<Game>().settings_video.fullscreen);
        click_button(&mut app, |a| matches!(a, Action::VideoApply));
        assert!(
            crate::storage::Profile::load(&root)
                .unwrap()
                .settings
                .fullscreen
        );
        assert_eq!(app.world().resource::<Game>().settings_page, Page::Main);
        click_button(&mut app, |a| {
            matches!(a, Action::SettingsPage(Page::Graphics))
        });
        click_button(&mut app, |a| {
            matches!(a, Action::SettingsPage(Page::Advanced))
        });
        for expected in [Page::Graphics, Page::Main] {
            app.world_mut().resource_mut::<Game>().escape();
            app.update();
            let game = app.world().resource::<Game>();
            assert!(game.settings_open);
            assert_eq!(game.settings_page, expected);
            assert_eq!(game.screen, Screen::Paused);
        }
        click_button(&mut app, |a| {
            matches!(a, Action::SettingsPage(Page::Options(Tab::Gameplay)))
        });
        click_button(&mut app, |a| {
            matches!(a, Action::Setting(super::super::settings::Toggle::Time))
        });
        assert!(
            crate::storage::Profile::load(&root)
                .unwrap()
                .settings
                .show_time
        );
        app.world_mut().resource_mut::<Game>().escape();
        app.world_mut().resource_mut::<Game>().escape();
        assert!(!app.world().resource::<Game>().settings_open);
        std::fs::remove_dir_all(root).unwrap();
    }
}
