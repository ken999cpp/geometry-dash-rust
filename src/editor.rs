use crate::{
    geometry::Point,
    level::{GRID, Level, LevelObject},
};
use std::{collections::VecDeque, path::PathBuf};

pub const TOOLS: [&str; 19] = [
    "select",
    "block",
    "platform",
    "spike",
    "minispike",
    "special_object",
    "jump_pad",
    "yellow_orb",
    "cube_portal",
    "ship_portal",
    "wave_portal",
    "robot_portal",
    "spider_portal",
    "ball_portal",
    "background_color_change",
    "ground_color_change",
    "color_change",
    "end_level",
    "start_pos",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Build,
    Edit,
    Delete,
}
pub struct Editor {
    pub level: Level,
    pub path: Option<PathBuf>,
    pub selected: Option<usize>,
    pub tool: usize,
    last_build_tool: usize,
    pub grid: bool,
    pub zoom: f64,
    pub camera: Point,
    pub rotation: f64,
    pub texture: usize,
    pub dirty: bool,
    pub revision: u64,
    pub mode: EditorMode,
    pub category: usize,
    pub snap_to_grid: bool,
    pub settings_open: bool,
    undo: VecDeque<Level>,
    redo: Vec<Level>,
}
impl Editor {
    pub fn new(level: Level, path: Option<PathBuf>) -> Self {
        Self {
            level,
            path,
            selected: None,
            tool: 1,
            last_build_tool: 1,
            grid: true,
            zoom: 1.,
            camera: Point::new(1200., 640.),
            rotation: 0.,
            texture: 1,
            dirty: false,
            revision: 0,
            mode: EditorMode::Build,
            category: 0,
            snap_to_grid: true,
            settings_open: false,
            undo: VecDeque::with_capacity(50),
            redo: Vec::new(),
        }
    }
    pub fn choose_tool(&mut self, tool: usize) {
        self.tool = tool;
        if tool > 0 {
            self.last_build_tool = tool;
            self.mode = EditorMode::Build;
        } else {
            self.mode = EditorMode::Edit;
        }
    }
    pub fn set_mode(&mut self, mode: EditorMode) {
        if self.tool > 0 {
            self.last_build_tool = self.tool;
        }
        self.mode = mode;
        self.tool = if mode == EditorMode::Build {
            self.last_build_tool
        } else {
            0
        };
    }
    pub fn remember(&mut self) {
        self.redo.clear();
        if self.undo.len() == 50 {
            self.undo.pop_front();
        }
        self.undo.push_back(self.level.clone());
        self.dirty = true;
        self.revision += 1;
    }
    pub fn undo(&mut self) {
        if let Some(level) = self.undo.pop_back() {
            self.redo.push(self.level.clone());
            self.level = level;
            self.selected = None;
            self.dirty = true;
            self.revision += 1;
        }
    }
    pub fn redo(&mut self) {
        if let Some(level) = self.redo.pop() {
            if self.undo.len() == 50 {
                self.undo.pop_front();
            }
            self.undo.push_back(self.level.clone());
            self.level = level;
            self.selected = None;
            self.dirty = true;
            self.revision += 1;
        }
    }
    pub fn snap(&self, p: Point) -> Point {
        if self.snap_to_grid {
            Point::new((p.x / GRID).floor() * GRID, (p.y / GRID).floor() * GRID)
        } else {
            p
        }
    }
    pub fn select_at(&mut self, p: Point) {
        self.selected = self
            .level
            .objects
            .iter()
            .enumerate()
            .rev()
            .find(|(_, o)| o.editor_contains(p))
            .map(|(i, _)| i);
        if let Some(i) = self.selected {
            self.rotation = self.level.objects[i].rotation;
        }
    }
    pub fn place(&mut self, p: Point) {
        if self.tool == 0 {
            self.select_at(p);
            return;
        }
        self.remember();
        let p = self.snap(p);
        let tool = TOOLS[self.tool];
        let mut object = LevelObject::new(
            if tool.ends_with("color_change") {
                "trigger"
            } else {
                tool
            },
            p.x,
            p.y,
        );
        object.rotation = self.rotation;
        if object.kind == "start_pos" {
            object.extra.insert("mode".into(), "auto".into());
            object.extra.insert("gravity".into(), "auto".into());
        }
        if object.kind == "block" {
            object.texture = Some(format!("block{}", self.texture));
        }
        if object.kind == "trigger" {
            object.trigger_type = Some(tool.into());
            object
                .trigger_params
                .insert("color".into(), "#ffffff".into());
            object.trigger_params.insert("duration".into(), 0.into());
            object.trigger_params.insert("alpha".into(), 1.into());
        }
        self.level.objects.push(object);
        self.selected = Some(self.level.objects.len() - 1);
    }
    pub fn cycle_start_setting(&mut self, gravity: bool) {
        let Some(id) = self
            .selected
            .filter(|&id| self.level.objects[id].kind == "start_pos")
        else {
            return;
        };
        let (key, options): (&str, &[&str]) = if gravity {
            ("gravity", &["auto", "normal", "reverse"])
        } else {
            (
                "mode",
                &["auto", "cube", "ship", "wave", "robot", "spider", "ball"],
            )
        };
        let current = self.level.objects[id]
            .extra
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("auto");
        let next =
            options[(options.iter().position(|&v| v == current).unwrap_or(0) + 1) % options.len()];
        self.remember();
        self.level.objects[id].extra.insert(key.into(), next.into());
    }
    pub fn delete(&mut self) {
        if let Some(i) = self.selected {
            self.remember();
            self.level.objects.remove(i);
            self.selected = None;
        }
    }
    pub fn duplicate(&mut self) {
        if let Some(i) = self.selected {
            self.remember();
            let mut o = self.level.objects[i].clone();
            o.x += GRID;
            self.level.objects.push(o);
            self.selected = Some(self.level.objects.len() - 1);
        }
    }
    pub fn move_selected(&mut self, offset: Point, remember: bool) {
        if let Some(i) = self.selected {
            if self.level.objects[i].editor_marker() && offset.x == 0. {
                return;
            }
            if remember {
                self.remember();
            }
            let o = &mut self.level.objects[i];
            o.x += offset.x;
            if !o.editor_marker() {
                o.y += offset.y;
            }
            self.revision += 1;
            self.dirty = true;
        }
    }
    pub fn rotate(&mut self, delta: f64) {
        self.rotation = (self.rotation + delta).rem_euclid(360.);
        if let Some(i) = self.selected {
            self.remember();
            self.level.objects[i].rotation = self.rotation;
        }
    }
    pub fn save(&mut self, path: PathBuf) -> Result<(), String> {
        self.level.save(&path)?;
        self.path = Some(path);
        self.dirty = false;
        Ok(())
    }
}
