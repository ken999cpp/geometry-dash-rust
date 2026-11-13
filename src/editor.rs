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
    pub selected_many: Vec<usize>,
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
    pub files_open: bool,
    pub pause_open: bool,
    pub view_only: bool,
    pub swipe: bool,
    pub rotate_objects: bool,
    pub move_step: f64,
    pub delete_filter: Option<String>,
    pub palette_page: usize,
    pub start_settings_open: bool,
    pub test_speed: f64,
    pub layer: i32,
    pub show_all_layers: bool,
    clipboard: Vec<LevelObject>,
    state_clipboard: Option<LevelObject>,
    undo: VecDeque<Level>,
    redo: Vec<Level>,
}
impl Editor {
    pub fn new(level: Level, path: Option<PathBuf>) -> Self {
        Self {
            level,
            path,
            selected: None,
            selected_many: Vec::new(),
            tool: 1,
            last_build_tool: 1,
            grid: true,
            zoom: 1.,
            camera: Point::new(crate::constants::VIEWPORT_HEIGHT * 16. / 9. / 2., 640.),
            rotation: 0.,
            texture: 1,
            dirty: false,
            revision: 0,
            mode: EditorMode::Build,
            category: 0,
            snap_to_grid: true,
            settings_open: false,
            files_open: false,
            pause_open: false,
            view_only: false,
            swipe: false,
            rotate_objects: false,
            move_step: GRID,
            delete_filter: None,
            palette_page: 0,
            start_settings_open: false,
            test_speed: 1.,
            layer: 0,
            show_all_layers: false,
            clipboard: Vec::new(),
            state_clipboard: None,
            undo: VecDeque::with_capacity(50),
            redo: Vec::new(),
        }
    }
    pub fn choose_tool(&mut self, tool: usize) {
        self.view_only = false;
        self.tool = tool;
        if tool > 0 {
            self.last_build_tool = tool;
            self.mode = EditorMode::Build;
        } else {
            self.mode = EditorMode::Edit;
        }
    }
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.view_only = false;
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
    pub fn selection(&self) -> Vec<usize> {
        if self.selected.is_none() {
            return Vec::new();
        }
        if self
            .selected
            .is_some_and(|i| !self.selected_many.contains(&i))
        {
            return self.selected.into_iter().collect();
        }
        if self.selected_many.is_empty() {
            self.selected.into_iter().collect()
        } else {
            self.selected_many
                .iter()
                .copied()
                .filter(|&i| i < self.level.objects.len())
                .collect()
        }
    }
    pub fn deselect(&mut self) {
        self.selected = None;
        self.selected_many.clear();
    }
    pub fn object_visible(&self, object: &LevelObject) -> bool {
        self.show_all_layers
            || object
                .extra
                .get("editor_layer")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                == self.layer as i64
    }
    pub fn set_scale(&mut self, x: f64, y: f64) {
        let ids = self.selection();
        if ids.is_empty() {
            return;
        }
        self.remember();
        for i in ids {
            let center = self.level.objects[i].rect().center;
            let object = &mut self.level.objects[i];
            object.extra.insert("scale_x".into(), x.into());
            object.extra.insert("scale_y".into(), y.into());
            let size = object.size();
            object.x = center.x - size.x / 2.;
            object.y = center.y - size.y / 2.;
        }
    }
    pub fn select_rect(&mut self, a: Point, b: Point) {
        self.selected_many = self
            .level
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| self.object_visible(o))
            .filter(|(_, o)| {
                let p = o.editor_rect().center;
                p.x >= a.x.min(b.x)
                    && p.x <= a.x.max(b.x)
                    && p.y >= a.y.min(b.y)
                    && p.y <= a.y.max(b.y)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selected_many.last().copied();
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn can_paste(&self) -> bool {
        !self.clipboard.is_empty()
    }
    pub fn can_paste_state(&self) -> bool {
        self.state_clipboard.is_some()
    }
    pub fn copy(&mut self) {
        self.clipboard = self
            .selection()
            .iter()
            .map(|&i| self.level.objects[i].clone())
            .collect();
    }
    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        let mut objects = self.clipboard.clone();
        let origin = Point::new(objects[0].x, objects[0].y);
        let offset = self.snap(self.camera) - origin;
        self.remember();
        let first = self.level.objects.len();
        for object in &mut objects {
            object.x += offset.x;
            object.y += offset.y;
        }
        self.level.objects.extend(objects);
        self.selected_many = (first..self.level.objects.len()).collect();
        self.selected = self.selected_many.last().copied();
        self.set_mode(EditorMode::Edit);
    }
    pub fn copy_state(&mut self) {
        self.state_clipboard = self
            .selected
            .and_then(|i| self.level.objects.get(i))
            .cloned();
    }
    pub fn paste_state(&mut self, color_only: bool) {
        let Some(source) = self.state_clipboard.clone() else {
            return;
        };
        let ids = self.selection();
        if ids.is_empty() {
            return;
        }
        self.remember();
        for i in ids {
            let object = &mut self.level.objects[i];
            if !color_only {
                let center = object.rect().center;
                object.rotation = source.rotation;
                object.texture = source.texture.clone();
                for key in [
                    "flip_x",
                    "flip_y",
                    "groups",
                    "color",
                    "scale_x",
                    "scale_y",
                    "editor_layer",
                ] {
                    if let Some(v) = source.extra.get(key) {
                        object.extra.insert(key.into(), v.clone());
                    } else {
                        object.extra.remove(key);
                    }
                }
                let size = object.size();
                object.x = center.x - size.x / 2.;
                object.y = center.y - size.y / 2.;
                object.trigger_params = source.trigger_params.clone();
            } else {
                for key in ["color", "alpha"] {
                    if let Some(value) = source.trigger_params.get(key) {
                        object.trigger_params.insert(key.into(), value.clone());
                    }
                }
            }
        }
    }
    pub fn flip(&mut self, horizontal: bool) {
        let ids = self.selection();
        if ids.is_empty() {
            return;
        }
        self.remember();
        let center = ids
            .iter()
            .map(|&i| self.level.objects[i].rect().center)
            .fold(Point::default(), |a, b| a + b)
            * (1. / ids.len() as f64);
        let key = if horizontal { "flip_x" } else { "flip_y" };
        for i in ids {
            let object = &mut self.level.objects[i];
            let p = object.rect().center;
            if horizontal {
                object.x += 2. * (center.x - p.x);
            } else {
                object.y += 2. * (center.y - p.y);
            }
            let current = object
                .extra
                .get(key)
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            object.extra.insert(key.into(), (!current).into());
        }
    }
    pub fn undo(&mut self) {
        if let Some(level) = self.undo.pop_back() {
            self.redo.push(self.level.clone());
            self.level = level;
            self.deselect();
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
            self.deselect();
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
            .find(|(_, o)| self.object_visible(o) && o.editor_contains(p))
            .map(|(i, _)| i);
        self.selected_many = self.selected.into_iter().collect();
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
        if self.layer != 0 {
            object
                .extra
                .insert("editor_layer".into(), self.layer.into());
        }
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
        self.selected_many = self.selected.into_iter().collect();
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
        let mut ids = self.selection();
        if ids.is_empty() {
            return;
        }
        self.remember();
        ids.sort_unstable();
        ids.dedup();
        for i in ids.into_iter().rev() {
            self.level.objects.remove(i);
        }
        self.deselect();
    }
    pub fn select_delete_at(&mut self, p: Point) {
        self.selected = self
            .level
            .objects
            .iter()
            .enumerate()
            .rev()
            .find(|(_, o)| self.object_visible(o) && self.deletable(o) && o.editor_contains(p))
            .map(|(i, _)| i);
        self.selected_many = self.selected.into_iter().collect();
    }
    fn deletable(&self, o: &LevelObject) -> bool {
        match self.delete_filter.as_deref() {
            None => true,
            Some("solid") => o.solid(),
            Some(kind) => o.kind == kind,
        }
    }
    pub fn delete_all(&mut self, start_only: bool) {
        self.selected_many = self
            .level
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                self.object_visible(o)
                    && if start_only {
                        o.kind == "start_pos"
                    } else {
                        self.deletable(o)
                    }
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selected_many.last().copied();
        self.delete();
    }
    pub fn duplicate(&mut self) {
        let ids = self.selection();
        if ids.is_empty() {
            return;
        }
        let copies = ids
            .iter()
            .map(|&i| {
                let mut o = self.level.objects[i].clone();
                o.x += GRID;
                o
            })
            .collect::<Vec<_>>();
        self.remember();
        let first = self.level.objects.len();
        self.level.objects.extend(copies);
        self.selected_many = (first..self.level.objects.len()).collect();
        self.selected = self.selected_many.last().copied();
    }
    pub fn move_selected(&mut self, offset: Point, remember: bool) {
        let ids = self.selection();
        if ids.is_empty() {
            return;
        }
        if remember {
            self.remember();
        }
        for i in ids {
            let o = &mut self.level.objects[i];
            o.x += offset.x;
            if !o.editor_marker() {
                o.y += offset.y;
            }
        }
        self.revision += 1;
        self.dirty = true;
    }
    pub fn rotate(&mut self, delta: f64) {
        self.rotation = (self.rotation + delta).rem_euclid(360.);
        let ids = self.selection();
        if ids.is_empty() {
            return;
        }
        self.remember();
        let center = ids
            .iter()
            .map(|&i| self.level.objects[i].rect().center)
            .fold(Point::default(), |a, b| a + b)
            * (1. / ids.len() as f64);
        for i in ids {
            self.level.objects[i].rotation =
                (self.level.objects[i].rotation + delta).rem_euclid(360.);
            if self.rotate_objects && !self.level.objects[i].editor_marker() {
                let object = &mut self.level.objects[i];
                let p = object.rect().center - center;
                let (sin, cos) = (-delta).to_radians().sin_cos();
                let p = center + Point::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
                    - object.size() * 0.5;
                object.x = p.x;
                object.y = p.y;
            }
        }
    }
    pub fn save(&mut self, path: PathBuf) -> Result<(), String> {
        self.level.save(&path)?;
        self.path = Some(path);
        self.dirty = false;
        Ok(())
    }
}
