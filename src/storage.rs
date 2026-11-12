use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn data_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("GEOMETRY_DASH_DATA_DIR") {
        return p.into();
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    base.unwrap_or_else(std::env::temp_dir)
        .join("geometry-dash-rust")
}
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| format!("Cannot save {}: {e}", path.display()))
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Progress {
    pub normal: u32,
    pub practice: u32,
    pub attempts: u32,
    #[serde(default)]
    pub earned_stars: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicChannel {
    Menu,
    Level,
    Sfx,
}
pub const FPS_LIMITS: [u32; 4] = [60, 120, 144, 240];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub no_death_effect: bool,
    pub hide_checkpoint_buttons: bool,
    pub menu_music_volume: u8,
    pub level_music_volume: u8,
    pub fps_limit: u32,
    pub sfx_volume: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            no_death_effect: false,
            hide_checkpoint_buttons: false,
            menu_music_volume: 100,
            level_music_volume: 100,
            fps_limit: 60,
            sfx_volume: 100,
        }
    }
}
impl Settings {
    pub fn fps_limit(&self) -> u32 {
        FPS_LIMITS
            .into_iter()
            .rev()
            .find(|&fps| fps <= self.fps_limit)
            .unwrap_or(60)
    }

    pub fn volume(&self, channel: MusicChannel) -> u8 {
        match channel {
            MusicChannel::Menu => self.menu_music_volume,
            MusicChannel::Level => self.level_music_volume,
            MusicChannel::Sfx => self.sfx_volume,
        }
        .min(100)
    }
    pub fn set_volume(&mut self, channel: MusicChannel, value: u8) -> bool {
        let volume = match channel {
            MusicChannel::Menu => &mut self.menu_music_volume,
            MusicChannel::Level => &mut self.level_music_volume,
            MusicChannel::Sfx => &mut self.sfx_volume,
        };
        let value = value.min(100);
        let changed = *volume != value;
        *volume = value;
        changed
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub levels: BTreeMap<String, Progress>,
    #[serde(default)]
    pub cube_icon: usize,
    #[serde(default)]
    pub settings: Settings,
}
impl Profile {
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join("profile.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let mut profile: Self = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for channel in [MusicChannel::Menu, MusicChannel::Level, MusicChannel::Sfx] {
            profile
                .settings
                .set_volume(channel, profile.settings.volume(channel));
        }
        profile.settings.fps_limit = profile.settings.fps_limit();
        Ok(profile)
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        write_json(&root.join("profile.json"), self)
    }
    pub fn import(&mut self, level: &crate::level::Level) {
        let progress = self.levels.entry(level.name.clone()).or_insert(Progress {
            normal: level.normal_progress,
            practice: level.practice_progress,
            attempts: level.attempts,
            earned_stars: 0,
        });
        if progress.normal >= 100 {
            progress.earned_stars = progress.earned_stars.max(level.stars());
        }
    }
    pub fn total_stars(&self) -> u32 {
        self.levels.values().fold(0_u32, |total, progress| {
            total.saturating_add(progress.earned_stars)
        })
    }
}
