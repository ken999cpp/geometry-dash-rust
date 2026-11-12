# Geometry Dash — Rust

A Bevy 0.19.1 desktop game and level editor, ported from the sibling C++/Qt project.
The game contains cube, ship, wave, robot, spider and ball modes, the three original
levels, and a Mode Laboratory level for trying the additional modes.

## Run

Install a recent stable Rust toolchain, then run:

```sh
cargo run --release
```

Linux requires the development packages for ALSA, Wayland, X11 and xkbcommon,
plus a Vulkan-capable graphics driver. On Debian/Ubuntu:

```sh
sudo apt install build-essential pkg-config libasound2-dev libudev-dev \
  libwayland-dev libxkbcommon-dev libx11-dev libxi-dev libxcursor-dev libxrandr-dev
```

On Windows use the Rust MSVC toolchain and Visual Studio C++ build tools.
Keep `assets/` and `levels/` beside the packaged executable. During development,
the executable finds them in the Cargo project automatically. Set
`GEOMETRY_DASH_BUNDLE_DIR` to override the bundle directory.

## Controls

| Context | Controls |
| --- | --- |
| Main menu | Left: icon kit; center: levels; right: create a level |
| Level selection | Left/Right arrows or page dots: browse; click the title, Enter or Space: play; tools button: edit |
| Main menu exit | Esc: open exit confirmation; Esc again or Cancel: dismiss; Enter or Exit: quit |
| Gameplay | Space, Up, or left mouse: action; Esc, P or pause icon: pause; R: restart |
| Pause | Esc: return to the same level page (editor after playtesting); P or play button: resume |
| Practice | Z: add checkpoint; X: remove checkpoint |
| Settings | Gear in the main menu, level selection or pause; Esc/Done: save and close |
| Editor | Bottom modes: build, select/move, delete; bottom categories: objects and triggers |
| Editor camera | WASD, middle/right drag: pan; wheel: zoom; 0: reset zoom |
| Editor objects | Arrows: move 80 units; Shift+arrow: move 1 unit; R/T: rotate 2.5° |
| Editor commands | Delete; Ctrl+D: duplicate; Ctrl+Z: undo; Ctrl+Shift+Z / Ctrl+Y: redo; G: toggle grid |
| Editor files | Ctrl+O: open; Ctrl+S: save; Ctrl+Shift+S: save as; F5: test |
| Start position | E: edit selected/paused level; Triggers → START POS; select marker, M: mode, V: gravity; F5: test from marker |
| Path/settings dialogs | Type a value; Ctrl+A: replace all; Enter: apply; Esc or close icon: cancel |

Editor buttons use artwork or vector icons; hover to see their function and
shortcut. The bottom palette contains the original 11 block textures and object
categories. The right tools include rotation, triggers, grid visibility and grid
snapping. The top gear opens file/name/music/background settings. Trigger values
use `#RRGGBB alpha duration_seconds`. Editor Exit or Esc opens a confirmation with
Cancel and Exit; New requires a second click when edits would be discarded.
See [editor controls](docs/editor.md).

Robot jumps higher while held (up to about 0.278 seconds of assistance). Ball changes
gravity on a press while grounded. Spider switches to the nearest opposite solid
surface on a press; hazards along that path remain lethal. Wave retains the
original constant diagonal speed. Ground jumps consume their input press, so
holding through an airborne yellow orb does not activate it. A fresh airborne
press may be held until the next orb; one press activates at most one orb.

Cube and ship parameters are in `src/constants.rs`; see [physics notes](docs/physics.md)
for the C++ movement profile. All movement, rotation, hitbox, camera and transition
parameters are centralized there; see the [manual tuning guide](docs/physics-tuning.md)
for their meaning and the remaining C++ algorithm differences. Running speed is
830.880288 units/s, with native GD 2.2 velocity rounding and movement parameters.
The cube's fresh-press jump was compared with a captured Steam Stereo Madness
trajectory; the projected sample error is below 1.5 pixels. See the scope and
remaining compatibility differences in [physics notes](docs/physics.md).
Death stops level music immediately, shows the
death effect for one second, and fades into a stationary 0.30-second spawn reveal
before movement resumes. Backgrounds and ground patterns scroll in both
menus and gameplay, with the ground rendered above the background. The menu cube
uses the gameplay jump impulse, gravity and running speed.

Level ratings use a PNG star. Ball, spider and their portals have dedicated PNG
sprites; checkpoints and the ship's cube rider use the original artwork. Generated
asset paths and prompts are in [sprite notes](docs/sprite-prompts.md). The exit dialog
uses bundled Liberation Sans for Cyrillic text, with its license in `assets/fonts/`.

Game settings are saved automatically. No Death Effect hides death particles
while retaining the restart delay. Hide Checkpoint Buttons hides the practice
HUD controls; Z/X and checkpoint markers remain available. Two volume sliders
control menu and level music independently, including mute at 0%.
FPS options are 60, 120, 144 and 240, capped by the current monitor's refresh rate.
Unavailable options are disabled; moving the window to a slower monitor lowers and
saves the limit automatically. Unknown refresh rates use a conservative 60 FPS
limit, with VSync retained. Rendering limits do not change the 240 Hz physics tick.
The renderer waits before presenting each frame to enforce the selected limit,
including after a slow frame. Changes apply to the next rendered frame.
The star counter always appears in menus; its former visibility option is ignored in old profiles.
Normal completion awards a level's stars once; practice,
editor playtests and repeat completions award no additional stars. Existing normal
completions are imported. The bundled levels award 1, 2 and 3 stars; custom JSON
may supply a `stars` value from 0 to 10.

Returning from level selection to the main menu resets browsing to the first
level. Opening level selection from the main menu also starts at Stereo Madness.
Returning directly from pause still opens the page of the level just played.

## Files and compatibility

Bundled levels are templates. Saving an edited template asks for a writable copy.
User levels, imported assets, statistics and icon selection are stored under:

- Linux: `$XDG_DATA_HOME/geometry-dash-rust`, defaulting to `~/.local/share/geometry-dash-rust`.
- Windows: `%LOCALAPPDATA%/geometry-dash-rust`.
- Override either with `GEOMETRY_DASH_DATA_DIR`.

Original JSON coordinates, object names, trigger settings and unknown fields are
preserved. Unknown objects remain editable but prevent gameplay until supported.
Statistics from original JSON files are imported when no saved profile entry exists.

The original MP3/WAV files are retained. The bundle also contains Ogg versions,
resolved automatically from the legacy audio paths; only the Vorbis decoder is
needed at runtime. Custom music imports accept Ogg and backgrounds accept PNG.
The asset dialogs accept existing bundle references or absolute file paths.
Imported files receive `user://filename` references; share the matching user asset
files with custom levels. Original assets retain their original ownership; this
port does not assign them a new license.

Menus reuse the original C++ project's graphical buttons, with a scrolling,
color-cycling background, pixel title, paged level selection and Normal/Practice
progress bars. The pause and completion screens use the original circular buttons.

## Checks

```sh
cargo check
cargo fmt --check
cargo test --no-default-features
cargo test
cargo build --release
```

To measure jump range/apex/rotation and ship response without a window:

```sh
cargo run --no-default-features --example physics_probe
```

Core tests run without a window, GPU or audio device. They cover level round trips,
frame-rate-independent replays, collision geometry, all six movement modes,
checkpoints, triggers, undo and failed saves. Desktop builds run in Linux/Windows CI.
See [parity checklist](docs/parity.md) for manual acceptance checks and differences.

## Git setup

```sh
bash scripts/setup-git.sh
```

This initializes `main` and appends `AGENTS.md` and `specs.md` to Git's global
excludes file without replacing existing rules. Those names are ignored in all
repositories for this user. `.gitignore` covers build output and temporary files;
`Cargo.lock`, assets and levels are intended to be tracked. Git setup requires a
writable `.git` directory and writable global excludes file.

When this environment protects `.git` from writes, a prepared commit can be
exported to the ignored `dist/geometry-dash-prepared.bundle`. In a normal terminal,
run `bash scripts/apply-prepared-commit.sh` to initialize Git, import that commit
and configure `origin`. It preserves working files and refuses to replace unrelated
commit history. Repeating the command after import keeps the current branch and
files; output does not require a pager such as `less`. Creating the GitHub
repository and publishing require account access.

Optional Codex skills for this desktop game are `screenshot` and `gh-fix-ci`.
`bash scripts/setup-skills.sh` installs them using the bundled Codex skill installer;
`--dry-run` previews the commands. Run it in a normal terminal with network access
and a writable Codex skills directory. Installed skills are available next turn.

