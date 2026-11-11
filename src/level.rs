use crate::{
    constants,
    geometry::{Point, Rect},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

pub const GRID: f64 = constants::GRID_SIZE;
pub const GROUND: f64 = constants::GROUND_HEIGHT_FOR_CUBE;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelObject {
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    #[serde(
        default,
        rename = "triggerType",
        skip_serializing_if = "Option::is_none"
    )]
    pub trigger_type: Option<String>,
    #[serde(
        default,
        rename = "triggerParams",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub trigger_params: BTreeMap<String, Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
impl LevelObject {
    pub fn new(kind: &str, x: f64, y: f64) -> Self {
        Self {
            kind: kind.into(),
            x,
            y,
            rotation: 0.,
            texture: None,
            trigger_type: None,
            trigger_params: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
    pub fn supported(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "block"
                | "platform"
                | "spike"
                | "minispike"
                | "special_object"
                | "jump_pad"
                | "yellow_orb"
                | "cube_portal"
                | "ship_portal"
                | "wave_portal"
                | "robot_portal"
                | "spider_portal"
                | "ball_portal"
                | "ReverseGravityPortal"
                | "NormalGravityPortal"
                | "trigger"
                | "end_level"
        ) && (self.kind != "trigger"
            || matches!(
                self.trigger_type.as_deref(),
                Some("color_change" | "ground_color_change" | "background_color_change")
            ))
    }
    pub fn size(&self) -> Point {
        match self.kind.as_str() {
            "platform" => Point::new(constants::BLOCK_SIZE, constants::PLATFORM_HEIGHT),
            "minispike" => Point::new(constants::BLOCK_SIZE, constants::MINISPIKE_HEIGHT),
            "jump_pad" => Point::new(constants::JUMP_PAD_WIDTH, constants::BLOCK_SIZE),
            k if k.ends_with("portal") || k.ends_with("Portal") => {
                Point::new(constants::PORTAL_WIDTH, constants::PORTAL_HEIGHT)
            }
            "trigger" | "end_level" => Point::new(constants::TRIGGER_WIDTH, constants::BLOCK_SIZE),
            _ => Point::new(constants::BLOCK_SIZE, constants::BLOCK_SIZE),
        }
    }
    /// JSON uses bottom-left positions with Y increasing upward.
    pub fn rect(&self) -> Rect {
        Rect::new(
            Point::new(self.x, self.y) + self.size() * 0.5,
            self.size(),
            -self.rotation,
        )
    }
    pub fn editor_marker(&self) -> bool {
        matches!(self.kind.as_str(), "trigger" | "end_level")
    }
    /// Editor-only markers keep legacy JSON coordinates and gameplay activation intact.
    pub fn editor_rect(&self) -> Rect {
        if self.editor_marker() {
            let row = match self.trigger_type.as_deref() {
                Some("ground_color_change") => 1.,
                Some("color_change") => 2.,
                _ => 0.,
            };
            Rect::new(
                Point::new(self.x + 20., 1000. + row * 88.),
                Point::new(88., 88.),
                0.,
            )
        } else {
            self.rect()
        }
    }
    pub fn editor_contains(&self, point: Point) -> bool {
        let rect = self.editor_rect();
        if self.editor_marker() {
            let delta = point - rect.center;
            delta.dot(delta) <= 44. * 44.
        } else {
            rect.contains(point)
        }
    }
    pub fn hitbox(&self) -> Rect {
        let mut r = self.rect();
        let (offset, size) = match self.kind.as_str() {
            "spike" => (
                Point::default(),
                Point::new(
                    constants::SPIKE_HITBOX_WIDTH,
                    constants::SPIKE_HITBOX_HEIGHT,
                ),
            ),
            "special_object" => (
                Point::new(0., constants::SPECIAL_OBJECT_HITBOX_OFFSET_Y),
                Point::new(
                    constants::SPECIAL_OBJECT_HITBOX_WIDTH,
                    constants::SPECIAL_OBJECT_HITBOX_HEIGHT,
                ),
            ),
            "minispike" => (
                Point::new(0., constants::MINISPIKE_HITBOX_OFFSET_Y),
                Point::new(
                    constants::MINISPIKE_HITBOX_SIZE,
                    constants::MINISPIKE_HITBOX_SIZE,
                ),
            ),
            "jump_pad" => (
                Point::new(0., constants::JUMP_PAD_HITBOX_OFFSET_Y),
                Point::new(constants::JUMP_PAD_WIDTH, constants::JUMP_PAD_HITBOX_HEIGHT),
            ),
            _ => (Point::default(), self.size()),
        };
        let [a, b] = r.axes();
        r.center = r.center + a * offset.x + b * offset.y;
        r.half = size * 0.5;
        r
    }
    pub fn solid(&self) -> bool {
        matches!(self.kind.as_str(), "block" | "platform")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    #[serde(default)]
    pub objects: Vec<LevelObject>,
    #[serde(default)]
    pub music: String,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default, rename = "normalProgress")]
    pub normal_progress: u32,
    #[serde(default, rename = "practiceProgress")]
    pub practice_progress: u32,
    #[serde(default)]
    pub attempts: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
fn default_background() -> String {
    "assets/background/stereomadness_bw.png".into()
}
impl Level {
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.into(),
            objects: vec![],
            music: String::new(),
            background: default_background(),
            normal_progress: 0,
            practice_progress: 0,
            attempts: 0,
            extra: BTreeMap::new(),
        }
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let level: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        level.validate()?;
        Ok(level)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Level name cannot be empty".into());
        }
        for (id, o) in self.objects.iter().enumerate() {
            if !o.x.is_finite() || !o.y.is_finite() || !o.rotation.is_finite() {
                return Err(format!("Object {id} has non-finite coordinates"));
            }
        }
        Ok(())
    }
    pub fn validate_playable(&self) -> Result<(), String> {
        self.validate()?;
        if let Some(o) = self.objects.iter().find(|o| !o.supported()) {
            return Err(format!(
                "Unsupported object or trigger: {} {:?}",
                o.kind, o.trigger_type
            ));
        }
        Ok(())
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        crate::storage::write_json(path, self)
    }
    pub fn length(&self) -> f64 {
        self.objects
            .iter()
            .filter(|o| o.kind != "end_level")
            .map(|o| o.x)
            .fold(0., f64::max)
            + constants::END_LEVEL_INDENT
    }
    pub fn stars(&self) -> u32 {
        match self.name.as_str() {
            "Stereo Madness" => 1,
            "Back On Track" => 2,
            "Polargeist" => 3,
            _ => self
                .extra
                .get("stars")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .min(10) as u32,
        }
    }
}

/// Resolve old Qt resource names relative to Bevy's asset directory.
pub fn asset_path(path: &str) -> String {
    path.trim_start_matches("qrc:")
        .trim_start_matches(':')
        .trim_start_matches('/')
        .strip_prefix("assets/")
        .unwrap_or(path.trim_start_matches('/'))
        .to_owned()
}
