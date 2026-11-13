mod attempt;
mod audio;
mod editor_ui;
mod frame_rate;
mod input;
mod menu;
mod native_menu;
mod render;
mod settings;
mod ui;
mod wave;

use crate::{
    editor::Editor,
    level::Level,
    simulation::{GameEvent, Simulation, Status, TickInput},
    storage::{self, Profile},
};
use bevy::{
    asset::io::{AssetSource, AssetSourceBuilder},
    prelude::*,
};
use std::path::{Path, PathBuf};

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FrameSet {
    World,
    Interface,
    Audio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Levels,
    Loading,
    Playing,
    Paused,
    Complete,
    Editor,
    Icons,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitTarget {
    Application,
    Editor,
}
#[derive(Clone)]
pub struct CatalogEntry {
    pub level: Level,
    pub path: PathBuf,
}
#[derive(Clone, Copy)]
pub enum Field {
    Open,
    SaveAs,
    Music,
    Background,
    Rotation,
    Trigger,
    Name,
    ObjectScale,
    ObjectGroups,
    EditorLayer,
    LevelColor(bool),
    EditorStep,
}
pub struct Prompt {
    pub field: Field,
    pub value: String,
    pub label: &'static str,
}

#[derive(Resource)]
pub struct Game {
    pub screen: Screen,
    pub catalog: Vec<CatalogEntry>,
    pub selected: usize,
    pub simulation: Option<Simulation>,
    pub editor: Option<Editor>,
    pub profile: Profile,
    pub root: PathBuf,
    pub bundle: PathBuf,
    pub input: TickInput,
    pub status: String,
    pub prompt: Option<Prompt>,
    pub ui_dirty: bool,
    pub scene_dirty: bool,
    pub pending_seek: Option<f64>,
    pub sfx: Option<&'static str>,
    pub testing: bool,
    pub dragging: Option<crate::geometry::Point>,
    pub drag_changed: bool,
    pub confirm_exit: bool,
    pub exit_dialog: Option<ExitTarget>,
    pub exit_requested: bool,
    pub settings_open: bool,
    settings_page: settings::Page,
    settings_video: settings::VideoOptions,
    display_rate: frame_rate::DisplayRate,
    suppress_action_until_release: bool,
    attempt: attempt::AttemptFlow,
    block_fixed_frame: bool,
    clear_trail: bool,
    menu_panel: Option<&'static str>,
    pause_hidden: bool,
}
impl Game {
    fn new(bundle: PathBuf) -> Self {
        let root = storage::data_dir();
        let (profile, status) = match Profile::load(&root) {
            Ok(p) => (p, String::new()),
            Err(e) => (
                Profile::default(),
                format!("Profile could not be read: {e}"),
            ),
        };
        let mut g = Self {
            screen: Screen::Menu,
            catalog: vec![],
            selected: 0,
            simulation: None,
            editor: None,
            profile,
            root,
            bundle,
            input: TickInput::default(),
            status,
            prompt: None,
            ui_dirty: true,
            scene_dirty: true,
            pending_seek: None,
            sfx: None,
            testing: false,
            dragging: None,
            drag_changed: false,
            confirm_exit: false,
            exit_dialog: None,
            exit_requested: false,
            settings_open: false,
            settings_page: Default::default(),
            settings_video: Default::default(),
            display_rate: Default::default(),
            suppress_action_until_release: false,
            attempt: Default::default(),
            block_fixed_frame: false,
            clear_trail: false,
            menu_panel: None,
            pause_hidden: false,
        };
        g.refresh();
        g
    }
    pub fn refresh(&mut self) {
        let mut catalog = Vec::new();
        for dir in [self.bundle.join("levels"), self.root.join("levels")] {
            let Ok(files) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut paths = files
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "json"))
                .collect::<Vec<_>>();
            paths.sort();
            for path in paths {
                match Level::load(&path) {
                    Ok(level) => {
                        self.profile.import(&level);
                        if let Some(i) = catalog
                            .iter()
                            .position(|e: &CatalogEntry| e.level.name == level.name)
                        {
                            catalog.remove(i);
                        }
                        catalog.push(CatalogEntry { level, path });
                    }
                    Err(e) => self.status = format!("Level could not be read: {e}"),
                }
            }
        }
        catalog.sort_by(|a, b| {
            let order = |name: &str| match name {
                "Stereo Madness" => 0,
                "Back On Track" => 1,
                "Polargeist" => 2,
                _ => 3,
            };
            order(&a.level.name)
                .cmp(&order(&b.level.name))
                .then(a.level.name.cmp(&b.level.name))
        });
        self.catalog = catalog;
        self.selected = self.selected.min(self.catalog.len().saturating_sub(1));
        self.ui_dirty = true;
    }
    pub fn screen(&mut self, screen: Screen) {
        if screen == Screen::Menu || (self.screen == Screen::Menu && screen == Screen::Levels) {
            self.selected = 0;
        }
        let is_level = |s| {
            matches!(
                s,
                Screen::Loading | Screen::Playing | Screen::Paused | Screen::Complete
            )
        };
        let same_scene = is_level(self.screen) && is_level(screen);
        self.screen = screen;
        self.pause_hidden = false;
        self.menu_panel = None;
        self.input = TickInput::default();
        self.ui_dirty = true;
        self.scene_dirty |= !same_scene;
        self.confirm_exit = false;
    }
    pub fn escape(&mut self) {
        if self.pause_hidden {
            self.pause_hidden = false;
            self.ui_dirty = true;
            return;
        }
        if self.menu_panel.take().is_some() {
            self.ui_dirty = true;
            return;
        }
        if self.exit_dialog.is_some() {
            self.cancel_exit();
            return;
        }
        if self.settings_open {
            self.settings_back();
            return;
        }
        if self.prompt.take().is_some() {
            self.confirm_exit = false;
            self.ui_dirty = true;
            return;
        }
        match self.screen {
            Screen::Menu => {
                self.exit_dialog = Some(ExitTarget::Application);
                self.input = Default::default();
                self.dragging = None;
                self.ui_dirty = true;
            }
            Screen::Playing => self.screen(Screen::Paused),
            Screen::Paused => self.leave_level(),
            Screen::Levels | Screen::Icons => self.screen(Screen::Menu),
            Screen::Loading | Screen::Complete => self.leave_level(),
            Screen::Editor => {
                if let Some(editor) = self.editor.as_mut() {
                    if editor.files_open {
                        editor.files_open = false;
                    } else if editor.start_settings_open {
                        editor.start_settings_open = false;
                    } else if editor.settings_open {
                        editor.settings_open = false;
                    } else if editor.view_only {
                        editor.view_only = false;
                    } else {
                        editor.pause_open = !editor.pause_open;
                    }
                }
                self.ui_dirty = true;
            }
        }
    }
    pub fn exit_editor(&mut self) {
        self.exit_dialog = Some(ExitTarget::Editor);
        self.confirm_exit = false;
        self.input = Default::default();
        self.dragging = None;
        self.ui_dirty = true;
    }
    pub fn confirm_dialog_exit(&mut self) {
        match self.exit_dialog {
            Some(ExitTarget::Application) => self.exit_requested = true,
            Some(ExitTarget::Editor) => {
                self.exit_dialog = None;
                self.editor = None;
                self.testing = false;
                self.status.clear();
                self.refresh();
                self.screen(Screen::Menu);
            }
            None => {}
        }
    }
    pub fn cancel_exit(&mut self) {
        self.exit_dialog = None;
        self.exit_requested = false;
        self.input = Default::default();
        self.suppress_action_until_release = true;
        self.ui_dirty = true;
    }
    pub fn open_settings(&mut self) {
        if self.screen == Screen::Playing {
            self.screen(Screen::Paused);
        }
        self.settings_open = true;
        self.settings_page = Default::default();
        self.settings_video = settings::VideoOptions::from(&self.profile.settings);
        self.input = Default::default();
        self.ui_dirty = true;
    }
    pub fn close_settings(&mut self) {
        self.settings_open = false;
        self.suppress_action_until_release = true;
        self.persist();
        self.ui_dirty = true;
    }
    pub fn settings_back(&mut self) {
        if self.settings_page == settings::Page::Main {
            self.close_settings();
        } else {
            self.settings_page = self.settings_page.parent();
            self.ui_dirty = true;
        }
    }
    pub fn leave_level(&mut self) {
        if self.testing && self.editor.is_some() {
            self.testing = false;
            self.simulation = None;
            self.pending_seek = None;
            self.screen(Screen::Editor);
            return;
        }
        self.update_progress();
        self.persist();
        let name = self.simulation.as_ref().map(|s| s.level.name.clone());
        self.simulation = None;
        self.testing = false;
        self.pending_seek = None;
        self.refresh();
        if let Some(name) = name
            && let Some(selected) = self
                .catalog
                .iter()
                .position(|entry| entry.level.name == name)
        {
            self.selected = selected;
        }
        self.screen(Screen::Levels);
    }
    pub fn browse_level(&mut self, direction: isize) {
        if !self.catalog.is_empty() {
            self.selected = (self.selected as isize + direction)
                .rem_euclid(self.catalog.len() as isize) as usize;
            self.ui_dirty = true;
        }
    }
    pub fn report(&mut self, result: Result<(), String>, success: &str) {
        self.status = match result {
            Ok(()) => success.into(),
            Err(e) => e,
        };
        self.ui_dirty = true;
    }
    pub fn persist(&mut self) {
        let result = self.profile.save(&self.root);
        if let Err(e) = result {
            self.status = format!("Cannot save profile: {e}");
            self.ui_dirty = true;
        }
    }
    pub fn start(&mut self, testing: bool) {
        let level = if testing {
            self.editor.as_ref().map(|e| e.level.clone())
        } else {
            self.catalog.get(self.selected).map(|e| e.level.clone())
        };
        let Some(level) = level else {
            self.status = "Create or open a level first".into();
            self.ui_dirty = true;
            return;
        };
        match Simulation::new(level) {
            Ok(mut sim) => {
                if testing && let Some(editor) = &self.editor {
                    let marker = editor
                        .selected
                        .filter(|&id| editor.level.objects[id].kind == "start_pos")
                        .or_else(|| {
                            editor
                                .level
                                .objects
                                .iter()
                                .rposition(|o| o.kind == "start_pos")
                        });
                    if let Some(marker) = marker
                        && let Err(error) = sim.start_at(marker)
                    {
                        self.status = error;
                        self.ui_dirty = true;
                        return;
                    }
                }
                let offset = sim.state.elapsed;
                self.testing = testing;
                self.simulation = Some(sim);
                self.screen(Screen::Loading);
                self.pending_seek = Some(offset);
                self.sfx = Some("playSound");
                if !testing {
                    self.add_attempt();
                }
            }
            Err(e) => {
                self.status = e;
                self.ui_dirty = true;
            }
        }
    }
    pub fn add_attempt(&mut self) {
        if self.testing {
            return;
        }
        if let Some(sim) = &self.simulation {
            self.profile
                .levels
                .entry(sim.level.name.clone())
                .or_default()
                .attempts += 1;
            self.persist();
        }
    }
    pub fn update_progress(&mut self) {
        if self.testing {
            return;
        }
        if let Some(sim) = &self.simulation {
            let progress = if sim.status == Status::Complete {
                100
            } else {
                sim.progress()
            };
            let p = self
                .profile
                .levels
                .entry(sim.level.name.clone())
                .or_default();
            if sim.practice {
                p.practice = p.practice.max(progress);
            } else {
                p.normal = p.normal.max(progress);
                if sim.status == Status::Complete {
                    p.earned_stars = p.earned_stars.max(sim.level.stars());
                }
            }
        }
    }
    pub fn restart(&mut self) {
        // Repeated clicks/held R must not bypass the visible death/reveal phases.
        if !self.attempt.playing()
            || self
                .simulation
                .as_ref()
                .is_some_and(|s| matches!(s.status, Status::Dead(_)))
        {
            return;
        }
        self.screen(Screen::Playing);
        self.attempt.enter(attempt::Phase::Restarting);
    }
    fn reset_attempt(&mut self) {
        if let Some(sim) = self.simulation.as_mut() {
            sim.restart(true);
            self.pending_seek = Some(sim.state.elapsed);
        }
        self.add_attempt();
        self.input = Default::default();
        self.clear_trail = true;
    }
    pub fn edit(&mut self) {
        let entry = if matches!(self.screen, Screen::Paused | Screen::Complete) {
            self.simulation.as_ref().map(|s| (s.level.clone(), None))
        } else {
            self.catalog
                .get(self.selected)
                .map(|e| (e.level.clone(), Some(e.path.clone())))
        };
        let (level, path) = entry.unwrap_or((Level::empty("New Level"), None));
        self.editor = Some(Editor::new(level, path));
        self.testing = false;
        self.screen(Screen::Editor);
    }
    pub fn save_editor(&mut self, save_as: bool) -> bool {
        let Some(editor) = self.editor.as_ref() else {
            return false;
        };
        let name = editor.level.name.replace(['/', '\\'], "_");
        let path = editor
            .path
            .clone()
            .filter(|p| p.starts_with(&self.root))
            .unwrap_or_else(|| self.root.join("levels").join(format!("{name}.json")));
        if save_as {
            let name = editor.level.name.replace(['/', '\\'], "_");
            self.prompt = Some(Prompt {
                field: Field::SaveAs,
                value: self
                    .root
                    .join("levels")
                    .join(format!("{name}.json"))
                    .display()
                    .to_string(),
                label: "Save level as (JSON path)",
            });
            self.ui_dirty = true;
            return false;
        }
        let result = self.editor.as_mut().unwrap().save(path);
        let saved = result.is_ok();
        self.report(result, "Level saved");
        self.refresh();
        saved
    }
    pub fn save_editor_then(&mut self, play: bool, exit: bool) {
        if !self.save_editor(false) {
            return;
        }
        if play {
            if let Some(editor) = self.editor.as_mut() {
                editor.pause_open = false;
            }
            self.start(true);
        } else if exit {
            self.exit_dialog = Some(ExitTarget::Editor);
            self.confirm_dialog_exit();
        }
    }
    pub fn open_prompt(&mut self, field: Field) {
        let Some(e) = &self.editor else {
            return;
        };
        let (value, label) = match field {
            Field::Open => (
                self.root.join("levels").display().to_string(),
                "Open a level (JSON path)",
            ),
            Field::SaveAs => {
                self.save_editor(true);
                return;
            }
            Field::Music => (
                e.level.music.clone(),
                "Music: asset path or absolute Ogg file path",
            ),
            Field::Background => (
                e.level.background.clone(),
                "Background: asset path or absolute PNG file path",
            ),
            Field::Rotation => (e.rotation.to_string(), "Rotation in degrees"),
            Field::Name => (e.level.name.clone(), "Level name"),
            Field::EditorStep => (
                format!("{}", e.move_step / crate::constants::GD_SCALE),
                "Custom movement step (native units, 0.1 to 300)",
            ),
            Field::LevelColor(ground) => (
                e.level
                    .extra
                    .get(if ground {
                        "ground_color"
                    } else {
                        "background_color"
                    })
                    .and_then(|v| v.as_str())
                    .unwrap_or(if ground { "#0066ff" } else { "#287dff" })
                    .to_owned(),
                "Level color: #RRGGBB",
            ),
            Field::ObjectScale => {
                let scale = e
                    .selected
                    .map(|i| e.level.objects[i].scale())
                    .unwrap_or(crate::geometry::Point::new(1., 1.));
                (
                    format!("{} {}", scale.x, scale.y),
                    "Object scale: X Y (0.1 to 4)",
                )
            }
            Field::ObjectGroups => {
                let groups = e
                    .selected
                    .and_then(|i| e.level.objects[i].extra.get("groups"))
                    .and_then(|v| v.as_array())
                    .map(|v| {
                        v.iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                (
                    groups,
                    "Object group IDs: comma-separated numbers (1 to 9999)",
                )
            }
            Field::EditorLayer => (
                e.selected
                    .and_then(|i| e.level.objects[i].extra.get("editor_layer"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(e.layer as i64)
                    .to_string(),
                "Editor layer (0 to 999)",
            ),
            Field::Trigger => {
                let Some(o) = e
                    .selected
                    .and_then(|i| e.level.objects.get(i))
                    .filter(|o| o.kind == "trigger")
                else {
                    self.status = "Select a color trigger first".into();
                    self.ui_dirty = true;
                    return;
                };
                (
                    format!(
                        "{} {} {}",
                        o.trigger_params
                            .get("color")
                            .and_then(|v| v.as_str())
                            .unwrap_or("#ffffff"),
                        o.trigger_params
                            .get("alpha")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.),
                        o.trigger_params
                            .get("duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.)
                    ),
                    "Trigger: #RRGGBB alpha duration_seconds",
                )
            }
        };
        self.prompt = Some(Prompt {
            field,
            value,
            label,
        });
        self.ui_dirty = true;
    }
    pub fn submit_prompt(&mut self) -> Result<(), String> {
        let Some(prompt) = self.prompt.take() else {
            return Ok(());
        };
        let Some(editor) = self.editor.as_mut() else {
            return Ok(());
        };
        let value = prompt.value.trim();
        let result = (|| {
            match prompt.field {
                Field::Open => {
                    let path = PathBuf::from(value);
                    let level = Level::load(&path)?;
                    if editor.dirty && !self.confirm_exit {
                        self.confirm_exit = true;
                        return Err(
                            "Unsaved edits. Submit again to replace them, or cancel and save."
                                .into(),
                        );
                    }
                    *editor = Editor::new(level, Some(path));
                }
                Field::SaveAs => {
                    editor.save(PathBuf::from(value))?;
                }
                Field::Music | Field::Background => {
                    let mut reference = value.to_string();
                    if Path::new(value).is_absolute() {
                        let path = Path::new(value);
                        let expected = if matches!(prompt.field, Field::Music) {
                            "ogg"
                        } else {
                            "png"
                        };
                        if path.extension().and_then(|p| p.to_str()) != Some(expected) {
                            return Err(format!("Import a .{expected} file"));
                        }
                        let name = path.file_name().ok_or("Invalid filename")?;
                        let dest = self.root.join("assets").join(name);
                        std::fs::create_dir_all(dest.parent().unwrap())
                            .map_err(|e| e.to_string())?;
                        std::fs::copy(path, &dest).map_err(|e| e.to_string())?;
                        reference = format!("user://{}", name.to_string_lossy());
                    } else {
                        let relative = crate::level::asset_path(value);
                        let candidate = self.bundle.join("assets").join(&relative);
                        let ogg = candidate.with_extension("ogg");
                        let valid = if value.starts_with("user://") {
                            self.root
                                .join("assets")
                                .join(value.trim_start_matches("user://"))
                                .is_file()
                        } else {
                            candidate.is_file()
                                || (matches!(prompt.field, Field::Music) && ogg.is_file())
                        };
                        if !value.is_empty() && !valid {
                            return Err("Asset file does not exist".into());
                        }
                    }
                    editor.remember();
                    if matches!(prompt.field, Field::Music) {
                        editor.level.music = reference;
                    } else {
                        editor.level.background = reference;
                    }
                }
                Field::Rotation => {
                    let degrees = value.parse::<f64>().map_err(|_| "Enter a finite number")?;
                    if !degrees.is_finite() {
                        return Err("Enter a finite number".into());
                    }
                    editor.rotate(degrees - editor.rotation);
                }
                Field::Name => {
                    if value.is_empty() {
                        return Err("Name cannot be empty".into());
                    }
                    editor.remember();
                    editor.level.name = value.into();
                }
                Field::EditorStep => {
                    let step = value
                        .parse::<f64>()
                        .map_err(|_| "Enter a finite movement step")?;
                    if !step.is_finite() || !(0.1..=300.).contains(&step) {
                        return Err("Step must be between 0.1 and 300".into());
                    }
                    editor.move_step = step * crate::constants::GD_SCALE;
                }
                Field::LevelColor(ground) => {
                    if value.len() != 7
                        || !value.starts_with('#')
                        || u32::from_str_radix(&value[1..], 16).is_err()
                    {
                        return Err("Color must be #RRGGBB".into());
                    }
                    editor.remember();
                    editor.level.extra.insert(
                        if ground {
                            "ground_color"
                        } else {
                            "background_color"
                        }
                        .into(),
                        value.into(),
                    );
                }
                Field::ObjectScale => {
                    let values = value
                        .split_whitespace()
                        .map(|v| v.parse::<f64>().map_err(|_| "Enter X and Y scale"))
                        .collect::<Result<Vec<_>, _>>()?;
                    if values.is_empty()
                        || values.len() > 2
                        || values
                            .iter()
                            .any(|v| !v.is_finite() || !(0.1..=4.).contains(v))
                    {
                        return Err("Scale must be between 0.1 and 4".into());
                    }
                    editor.set_scale(values[0], *values.get(1).unwrap_or(&values[0]));
                }
                Field::ObjectGroups => {
                    let mut groups = value
                        .split(|c: char| c == ',' || c.is_whitespace())
                        .filter(|s| !s.is_empty())
                        .map(|v| v.parse::<u32>().map_err(|_| "Use numeric group IDs"))
                        .collect::<Result<Vec<_>, _>>()?;
                    if groups.iter().any(|g| !(1..=9999).contains(g)) {
                        return Err("Group IDs must be between 1 and 9999".into());
                    }
                    groups.sort_unstable();
                    groups.dedup();
                    let ids = editor.selection();
                    if !ids.is_empty() {
                        editor.remember();
                        for i in ids {
                            editor.level.objects[i]
                                .extra
                                .insert("groups".into(), serde_json::json!(groups));
                        }
                    }
                }
                Field::EditorLayer => {
                    let layer = value.parse::<i32>().map_err(|_| "Enter an editor layer")?;
                    if !(0..=999).contains(&layer) {
                        return Err("Layer must be between 0 and 999".into());
                    }
                    let ids = editor.selection();
                    if ids.is_empty() {
                        editor.layer = layer;
                    } else {
                        editor.remember();
                        for i in ids {
                            editor.level.objects[i]
                                .extra
                                .insert("editor_layer".into(), layer.into());
                        }
                    }
                }
                Field::Trigger => {
                    let fields = value.split_whitespace().collect::<Vec<_>>();
                    if fields.len() != 3 {
                        return Err("Use: #RRGGBB alpha duration_seconds".into());
                    }
                    if fields[0].len() != 7
                        || !fields[0].starts_with('#')
                        || u32::from_str_radix(&fields[0][1..], 16).is_err()
                    {
                        return Err("Color must be #RRGGBB".into());
                    }
                    let alpha = fields[1].parse::<f64>().map_err(|_| "Invalid alpha")?;
                    let duration = fields[2].parse::<f64>().map_err(|_| "Invalid duration")?;
                    if !(0. ..=1.).contains(&alpha) || !duration.is_finite() || duration < 0. {
                        return Err("Alpha must be 0–1; duration must be nonnegative".into());
                    }
                    if let Some(i) = editor.selected {
                        editor.remember();
                        let o = &mut editor.level.objects[i];
                        o.trigger_params.insert("color".into(), fields[0].into());
                        o.trigger_params.insert("alpha".into(), alpha.into());
                        o.trigger_params.insert("duration".into(), duration.into());
                    }
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.prompt = Some(prompt);
        } else {
            self.confirm_exit = false;
        }
        self.ui_dirty = true;
        self.scene_dirty = true;
        result
    }
}

pub fn run() {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let bundle = std::env::var_os("GEOMETRY_DASH_BUNDLE_DIR")
        .map(PathBuf::from)
        .or_else(|| exe.filter(|p| p.join("assets").is_dir()))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let game = Game::new(bundle.clone());
    let user_assets = game.root.join("assets");
    App::new()
        .register_asset_source(
            "user",
            AssetSourceBuilder::new(AssetSource::get_default_reader(
                user_assets.display().to_string(),
            )),
        )
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: bundle.join("assets").display().to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        title: "Geometry Dash · Rust".into(),
                        resolution: (1280, 720).into(),
                        resize_constraints: bevy::window::WindowResizeConstraints {
                            min_width: 960.,
                            min_height: 640.,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(game)
        .insert_resource(Time::<Fixed>::from_hz(crate::simulation::HZ))
        .configure_sets(
            Update,
            (FrameSet::World, FrameSet::Interface, FrameSet::Audio).chain(),
        )
        .add_plugins((
            frame_rate::FrameRatePlugin,
            input::InputPlugin,
            render::PresentationPlugin,
            ui::InterfacePlugin,
            audio::GameAudioPlugin,
        ))
        .add_systems(FixedUpdate, step)
        .add_systems(PreUpdate, attempt::advance)
        .add_systems(PreUpdate, editor_ui::sync_test_rate)
        .add_systems(Last, exit_application)
        .run();
}

fn step(
    mut game: ResMut<Game>,
    mut music: Query<(Option<&AudioSink>, &mut PlaybackSettings), With<audio::Music>>,
) {
    if game.screen != Screen::Playing
        || game.exit_dialog.is_some()
        || !game.attempt.playing()
        || game.block_fixed_frame
    {
        return;
    }
    let input = game.input;
    game.input.pressed = false;
    game.input.released = false;
    let Some(sim) = game.simulation.as_mut() else {
        return;
    };
    sim.tick(input);
    let mut died = false;
    let mut restarted = None;
    let mut completed = false;
    for event in &sim.events {
        match event {
            GameEvent::Died => died = true,
            GameEvent::Restarted(t) => restarted = Some(*t),
            GameEvent::Completed => completed = true,
            _ => {}
        }
    }
    game.update_progress();
    if died {
        game.attempt.enter(attempt::Phase::Dying);
        game.input = Default::default();
        // Silence the old attempt at the collision tick, before persistence or
        // the respawn timer; the next attempt creates a new stream at its offset.
        for (sink, mut settings) in &mut music {
            settings.paused = true;
            if let Some(sink) = sink {
                sink.stop();
            }
        }
        game.sfx = Some("explode");
        game.persist();
    }
    if let Some(t) = restarted {
        game.pending_seek = Some(t);
        game.scene_dirty = true;
        game.add_attempt();
    }
    if completed {
        game.sfx = Some("endStart");
        game.persist();
        game.screen(Screen::Complete);
    }
}

fn exit_application(mut game: ResMut<Game>, mut exits: MessageWriter<bevy::app::AppExit>) {
    if game.exit_requested {
        game.update_progress();
        game.persist();
        game.exit_requested = false;
        exits.write(bevy::app::AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_save_transitions_require_success_and_preserve_the_document_on_error() {
        let root = std::env::temp_dir().join(format!("gd-editor-save-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("levels"), b"blocked directory").unwrap();
        let mut game = Game::new(root.clone());
        game.root = root.clone();
        game.editor = Some(Editor::new(Level::empty("Save test"), None));
        game.screen(Screen::Editor);
        let editor = game.editor.as_mut().unwrap();
        editor.dirty = true;
        editor.pause_open = true;
        let original = editor.level.clone();
        for play in [false, true] {
            game.save_editor_then(play, !play);
            assert_eq!(game.screen, Screen::Editor);
            let editor = game.editor.as_ref().unwrap();
            assert!(editor.pause_open && editor.dirty);
            assert_eq!(editor.level, original);
            assert!(!game.status.is_empty());
        }
        std::fs::remove_file(root.join("levels")).unwrap();
        game.save_editor_then(true, false);
        assert!(game.testing);
        assert!(!game.editor.as_ref().unwrap().dirty);
        assert_eq!(
            Level::load(&root.join("levels/Save test.json")).unwrap(),
            original
        );
        game.screen(Screen::Editor);
        game.editor.as_mut().unwrap().pause_open = true;
        game.save_editor_then(false, true);
        assert_eq!(game.screen, Screen::Menu);
        assert!(game.editor.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn level_browsing_restarts_at_the_first_page_after_visiting_main_menu() {
        let mut game = Game::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.screen(Screen::Levels);
        game.browse_level(2);
        assert_eq!(game.catalog[game.selected].level.name, "Polargeist");
        game.escape();
        assert_eq!(game.screen, Screen::Menu);
        assert_eq!(game.selected, 0);
        game.screen(Screen::Levels);
        assert_eq!(game.catalog[game.selected].level.name, "Stereo Madness");
        // Returning directly from pause still shows the same level's page.
        game.selected = 2;
        game.screen(Screen::Paused);
        game.screen(Screen::Levels);
        assert_eq!(game.selected, 2);
        game.screen(Screen::Menu);
        game.selected = 2;
        game.screen(Screen::Levels);
        assert_eq!(game.selected, 0);
    }
    #[test]
    fn stars_are_awarded_once_for_normal_completion_only() {
        let mut game = Game::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.profile = Profile::default();
        let mut level = Level::empty("Polargeist");
        level.objects.push(crate::level::LevelObject::new(
            "end_level",
            40.,
            crate::level::GROUND,
        ));
        for (practice, testing, expected) in [
            (true, false, 0),
            (false, true, 0),
            (false, false, 3),
            (false, false, 3),
        ] {
            let mut sim = Simulation::new(level.clone()).unwrap();
            sim.practice = practice;
            for _ in 0..8 {
                sim.tick(Default::default());
            }
            assert_eq!(sim.status, Status::Complete);
            game.testing = testing;
            game.simulation = Some(sim);
            game.update_progress();
            game.update_progress();
            assert_eq!(game.profile.total_stars(), expected);
        }
    }
    #[test]
    fn editor_playtest_uses_start_pos_and_music_offset_without_awarding_progress() {
        let mut game = Game::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-start-pos-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = Profile::default();
        let mut level = Level::empty("Start position test");
        level
            .objects
            .push(crate::level::LevelObject::new("block", 20000., 420.));
        level
            .objects
            .push(crate::level::LevelObject::new("ship_portal", 1000., 738.));
        level
            .objects
            .push(crate::level::LevelObject::new("start_pos", 5000., 820.));
        game.editor = Some(Editor::new(level, None));
        game.start(true);
        let state = &game.simulation.as_ref().unwrap().state;
        assert_eq!(state.player.mode, crate::simulation::PlayerMode::Ship);
        assert!(state.player.position.x > 5000.);
        assert_eq!(game.pending_seek, Some(state.elapsed));
        game.update_progress();
        assert!(!game.profile.levels.contains_key("Start position test"));
        let track = game.editor.as_ref().unwrap().level.clone();
        game.catalog = vec![CatalogEntry {
            level: track,
            path: PathBuf::new(),
        }];
        game.selected = 0;
        game.start(false);
        assert_eq!(
            game.simulation.as_ref().unwrap().state.player.position.x,
            40.
        );
        assert_eq!(game.pending_seek, Some(0.));
        std::fs::remove_dir_all(root).unwrap();
    }
}
