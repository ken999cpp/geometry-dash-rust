use super::{Field, Game, Screen};
use crate::{editor::EditorMode, geometry::Point, simulation::TickInput};
use bevy::{
    app::{RunFixedMainLoop, RunFixedMainLoopSystems},
    input::{
        ButtonState,
        keyboard::KeyboardInput,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    },
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RunFixedMainLoop,
            capture.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(PostUpdate, sync_cursor_visibility);
    }
}

fn sync_cursor_visibility(
    game: Res<Game>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    for (window, mut cursor) in &mut windows {
        let visible = game.screen != Screen::Playing
            || !window.focused
            || game.settings_open
            || game.exit_dialog.is_some()
            || game.prompt.is_some();
        if cursor.visible != visible {
            cursor.visible = visible;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn capture(
    mut game: ResMut<Game>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut events: MessageReader<KeyboardInput>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        game.input = TickInput::default();
        game.dragging = None;
        if game.screen == Screen::Playing {
            game.screen(Screen::Paused);
        }
        events.clear();
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        game.escape();
        events.clear();
        return;
    }
    if game.exit_dialog.is_some() {
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            game.confirm_dialog_exit();
        }
        game.input = TickInput::default();
        events.clear();
        return;
    }
    if game.settings_open {
        game.input = TickInput::default();
        events.clear();
        return;
    }
    // A short chord can be pressed and released between rendered frames.
    let ctrl = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.just_released(KeyCode::ControlLeft)
        || keys.just_released(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
        || keys.just_released(KeyCode::ShiftLeft)
        || keys.just_released(KeyCode::ShiftRight);
    if game.prompt.is_some() {
        if ctrl && keys.just_pressed(KeyCode::KeyA) {
            game.prompt.as_mut().unwrap().value.clear();
            game.ui_dirty = true;
        }
        for event in events.read().filter(|e| e.state == ButtonState::Pressed) {
            match event.key_code {
                KeyCode::Backspace => {
                    game.prompt.as_mut().unwrap().value.pop();
                    game.ui_dirty = true;
                }
                KeyCode::Enter => {}
                _ => {
                    if !ctrl && let Some(text) = &event.text {
                        game.prompt
                            .as_mut()
                            .unwrap()
                            .value
                            .extend(text.chars().filter(|c| !c.is_control()));
                        game.ui_dirty = true;
                    }
                }
            }
        }
        if keys.just_pressed(KeyCode::Enter) {
            let result = game.submit_prompt();
            game.report(result, "Updated");
            if game.prompt.is_none() {
                game.refresh();
            }
        }
        return;
    }
    if game.menu_panel.is_some() {
        if keys.just_pressed(KeyCode::KeyE)
            && matches!(game.screen, Screen::Levels | Screen::Paused)
        {
            game.edit();
        }
        game.input = Default::default();
        events.clear();
        return;
    }
    if game.pause_hidden {
        if mouse.just_pressed(MouseButton::Left) {
            game.pause_hidden = false;
            game.ui_dirty = true;
        }
        events.clear();
        return;
    }
    events.clear();
    if game.screen == Screen::Paused && keys.just_pressed(KeyCode::KeyE) {
        game.edit();
        return;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        match game.screen {
            Screen::Playing => game.screen(Screen::Paused),
            Screen::Paused => game.screen(Screen::Playing),
            _ => {}
        }
        return;
    }
    if game.screen == Screen::Levels {
        if keys.just_pressed(KeyCode::KeyE) {
            game.edit();
            return;
        }
        if keys.just_pressed(KeyCode::ArrowLeft) {
            game.browse_level(-1);
        }
        if keys.just_pressed(KeyCode::ArrowRight) {
            game.browse_level(1);
        }
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
            game.start(false);
        }
        return;
    }
    if game.screen == Screen::Playing {
        if mouse.just_pressed(MouseButton::Left)
            && game.simulation.as_ref().is_some_and(|sim| sim.practice)
            && !game.profile.settings.hide_checkpoint_buttons
            && window.cursor_position().is_some_and(|p| {
                (24. ..=146.).contains(&p.x)
                    && (window.height() - 76. ..=window.height() - 24.).contains(&p.y)
            })
        {
            game.input = Default::default();
            return;
        }
        // The HUD pause control must not also send a jump to the fixed loop.
        if mouse.just_pressed(MouseButton::Left)
            && window.cursor_position().is_some_and(|p| {
                let scale = window.height() / 320.;
                p.x >= window.width() - 24. * scale && p.y <= 24. * scale
            })
        {
            game.screen(Screen::Paused);
            return;
        }
        if game.suppress_action_until_release {
            game.input = TickInput::default();
            if !keys.pressed(KeyCode::Space)
                && !keys.pressed(KeyCode::ArrowUp)
                && !mouse.pressed(MouseButton::Left)
            {
                game.suppress_action_until_release = false;
            } else {
                return;
            }
        }
        if !game.attempt.playing() || game.block_fixed_frame {
            game.input = TickInput::default();
            return;
        }
        let held = keys.pressed(KeyCode::Space)
            || keys.pressed(KeyCode::ArrowUp)
            || mouse.pressed(MouseButton::Left);
        if (held && !game.input.held)
            || keys.just_pressed(KeyCode::Space)
            || keys.just_pressed(KeyCode::ArrowUp)
            || mouse.just_pressed(MouseButton::Left)
        {
            game.input.pressed = true;
        }
        if (!held && game.input.held)
            || keys.just_released(KeyCode::Space)
            || keys.just_released(KeyCode::ArrowUp)
            || mouse.just_released(MouseButton::Left)
        {
            game.input.released = true;
        }
        game.input.held = held;
        if let Some(sim) = game.simulation.as_mut() {
            if keys.just_pressed(KeyCode::KeyZ) {
                sim.checkpoint();
            }
            if keys.just_pressed(KeyCode::KeyX) {
                sim.remove_checkpoint();
            }
        }
        if keys.just_pressed(KeyCode::KeyR) {
            game.restart();
        }
        return;
    }
    if game.screen != Screen::Editor {
        return;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        game.save_editor(shift);
        return;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyO) {
        game.open_prompt(Field::Open);
        return;
    }
    if keys.just_pressed(KeyCode::F5) {
        game.start(true);
        return;
    }
    let Some(editor) = game.editor.as_mut() else {
        return;
    };
    let old_revision = editor.revision;
    if keys.just_pressed(KeyCode::KeyM) {
        editor.cycle_start_setting(false);
    }
    if keys.just_pressed(KeyCode::KeyV) {
        editor.cycle_start_setting(true);
    }
    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        if shift {
            editor.redo();
        } else {
            editor.undo();
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        editor.redo();
    }
    if ctrl && keys.just_pressed(KeyCode::KeyD) {
        editor.duplicate();
    }
    if keys.just_pressed(KeyCode::Delete) {
        editor.delete();
    }
    if keys.just_pressed(KeyCode::KeyG) {
        editor.grid = !editor.grid;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        editor.rotate(2.5);
    }
    if keys.just_pressed(KeyCode::KeyT) {
        editor.rotate(-2.5);
    }
    if keys.just_pressed(KeyCode::Digit0) {
        editor.zoom = 1.;
    }
    let step = if shift { 1. } else { crate::level::GRID };
    for (key, x, y) in [
        (KeyCode::ArrowLeft, -step, 0.),
        (KeyCode::ArrowRight, step, 0.),
        (KeyCode::ArrowUp, 0., step),
        (KeyCode::ArrowDown, 0., -step),
    ] {
        if keys.just_pressed(key) {
            editor.move_selected(Point::new(x, y), true);
        }
    }
    let pan = 1000. * time.delta_secs_f64() / editor.zoom;
    if !ctrl {
        if keys.pressed(KeyCode::KeyA) {
            editor.camera.x -= pan;
        }
        if keys.pressed(KeyCode::KeyD) {
            editor.camera.x += pan;
        }
        if keys.pressed(KeyCode::KeyW) {
            editor.camera.y += pan;
        }
        if keys.pressed(KeyCode::KeyS) {
            editor.camera.y -= pan;
        }
    }
    let in_view = window.cursor_position().is_some_and(|p| {
        super::editor_ui::canvas_contains(
            p,
            Vec2::new(window.width(), window.height()),
            editor.settings_open,
        )
    });
    if in_view {
        editor.zoom = (editor.zoom * 1.2_f64.powf(scroll.delta.y as f64)).clamp(0.1, 10.);
        if mouse.pressed(MouseButton::Middle) || mouse.pressed(MouseButton::Right) {
            let units = crate::constants::VIEWPORT_HEIGHT / window.height() as f64 / editor.zoom;
            editor.camera.x -= motion.delta.x as f64 * units;
            editor.camera.y += motion.delta.y as f64 * units;
        }
        if let Some(cursor) = window.cursor_position() {
            let units = crate::constants::VIEWPORT_HEIGHT / window.height() as f64 / editor.zoom;
            let world = editor.camera
                + Point::new(
                    (cursor.x - window.width() / 2.) as f64 * units,
                    (window.height() / 2. - cursor.y) as f64 * units,
                );
            if mouse.just_pressed(MouseButton::Left) {
                match editor.mode {
                    EditorMode::Build => editor.place(world),
                    EditorMode::Edit => editor.select_at(world),
                    EditorMode::Delete => {
                        editor.select_at(world);
                        editor.delete();
                    }
                }
                let rev = editor.revision;
                game.dragging = Some(world);
                game.drag_changed = false;
                game.scene_dirty |= old_revision != rev;
                game.ui_dirty = true;
            } else if mouse.pressed(MouseButton::Left) {
                let previous = game.dragging;
                let remember = !game.drag_changed;
                if let Some(previous) = previous {
                    let editor = game.editor.as_mut().unwrap();
                    if editor.mode == EditorMode::Edit {
                        let delta = if editor.snap_to_grid {
                            let p = editor.snap(world);
                            let q = editor.snap(previous);
                            p - q
                        } else {
                            world - previous
                        };
                        if delta != Point::default() {
                            editor.move_selected(delta, remember);
                            game.dragging = Some(world);
                            game.drag_changed = true;
                            game.scene_dirty = true;
                        }
                    }
                }
            }
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        game.dragging = None;
    }
    if let Some(editor) = game.editor.as_ref()
        && editor.revision != old_revision
    {
        game.scene_dirty = true;
        game.ui_dirty = true;
        game.confirm_exit = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press_escape(app: &mut App) {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        keys.press(KeyCode::Escape);
        app.update();
    }

    #[test]
    fn escape_leaves_pause_on_the_same_level_page_and_confirms_application_exit_in_menu() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-exit-{}", std::process::id()));
        let root = game.root.clone();
        game.screen = Screen::Playing;
        let level = game
            .catalog
            .iter()
            .find(|entry| entry.level.name == "Back On Track")
            .unwrap()
            .level
            .clone();
        game.simulation = Some(crate::simulation::Simulation::new(level).unwrap());
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .insert_resource(Time::<()>::default())
            .insert_resource(Time::<Real>::default())
            .add_message::<KeyboardInput>()
            .add_message::<bevy::app::AppExit>()
            .add_systems(PreUpdate, super::super::attempt::advance)
            .add_systems(Update, capture)
            .add_systems(FixedUpdate, super::super::step)
            .add_systems(Last, super::super::exit_application);
        app.world_mut().spawn(Window::default());
        press_escape(&mut app);
        let game = app.world().resource::<Game>();
        assert!(game.exit_dialog.is_none());
        assert_eq!(game.screen, Screen::Paused);
        assert!(!game.exit_requested);
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty()
        );
        for _ in 0..240 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        assert_eq!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .state
                .tick,
            0
        );
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(KeyCode::KeyP);
        }
        app.update();
        assert_eq!(app.world().resource::<Game>().screen, Screen::Playing);
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .state
                .tick,
            1
        );
        press_escape(&mut app);
        assert_eq!(app.world().resource::<Game>().screen, Screen::Paused);
        press_escape(&mut app);
        let game = app.world().resource::<Game>();
        assert_eq!(game.screen, Screen::Levels);
        assert_eq!(game.catalog[game.selected].level.name, "Back On Track");
        assert!(game.exit_dialog.is_none());
        assert!(game.simulation.is_none());
        for screen in [Screen::Levels, Screen::Icons] {
            app.world_mut().resource_mut::<Game>().screen(screen);
            press_escape(&mut app);
            assert_eq!(app.world().resource::<Game>().screen, Screen::Menu);
            assert!(app.world().resource::<Game>().exit_dialog.is_none());
        }
        press_escape(&mut app);
        assert_eq!(
            app.world().resource::<Game>().exit_dialog,
            Some(super::super::ExitTarget::Application)
        );
        assert_eq!(app.world().resource::<Game>().screen, Screen::Menu);
        app.world_mut().resource_mut::<Game>().cancel_exit();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
        assert!(app.world().resource::<Game>().exit_dialog.is_none());
        assert_eq!(app.world().resource::<Game>().screen, Screen::Menu);
        press_escape(&mut app);
        assert!(app.world().resource::<Game>().exit_dialog.is_some());
        press_escape(&mut app);
        assert!(app.world().resource::<Game>().exit_dialog.is_none());
        assert!(
            app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty()
        );
        press_escape(&mut app);
        assert!(app.world().resource::<Game>().exit_dialog.is_some());
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(KeyCode::Enter);
        }
        app.update();
        assert!(
            !app.world()
                .resource::<Messages<bevy::app::AppExit>>()
                .is_empty()
        );
        assert!(root.join("profile.json").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn leaving_a_paused_playtest_restores_the_unsaved_editor() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.edit();
        let editor = game.editor.as_mut().unwrap();
        editor.dirty = true;
        editor.camera = Point::new(1840., 640.);
        let level = editor.level.clone();
        game.simulation = Some(crate::simulation::Simulation::new(level.clone()).unwrap());
        game.testing = true;
        game.screen(Screen::Paused);
        game.escape();
        assert_eq!(game.screen, Screen::Editor);
        assert!(!game.testing);
        assert!(game.simulation.is_none());
        assert!(game.exit_dialog.is_none());
        let editor = game.editor.as_ref().unwrap();
        assert_eq!(editor.level, level);
        assert_eq!(editor.camera, Point::new(1840., 640.));
        assert!(editor.dirty);
    }
}
