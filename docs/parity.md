# Port acceptance checklist

Implementation exists for the features below. Automated core checks are distinct
from manual desktop and C++ comparison checks; unchecked items have not been
validated on both supported platforms.

Local validation: `cargo check`, formatting, Clippy with warnings denied, the Linux
release build, 40 core tests and 20 Bevy ECS/UI/audio tests pass. Desktop launch attempts
are blocked by sandbox access to both Wayland and X11; rendered gameplay and audio
have not been manually verified. Windows builds are configured in CI but have not
been run from this environment.

| Area | Implemented behavior | Acceptance check |
| --- | --- | --- |
| Level files | Three original levels, JSON round trip, unknown field preservation | Automated |
| Simulation | 240 Hz, ordered collisions, buffered input, interpolated rendering | Automated core; manual camera review pending |
| Movement | Cube hold jumps, ship thrust/caps, constant-speed wave | Automated core; C++ trajectory comparison pending |
| New modes | Robot hold, ball gravity, spider surface switching, portals | Automated core; manual mixed-mode play pending |
| Objects | Blocks, platforms, spikes, minispikes, special objects, pads, orbs | Automated representative contacts; original-level playthrough pending |
| Practice | Add/remove checkpoints, full simulation restore, music seeking | Automated state restore; manual audio sync pending |
| Triggers | Background/ground/general colors, opacity, duration, completion | Automated timeline restore; visual comparison pending |
| Game UI | Main menu, level choice, pause, restart, completion, loading assets | Manual pending |
| Profile | Attempts, normal/practice progress, cube icon choice | Import tested; desktop restart check pending |
| Editor | Palette, all original objects, grid, selection, drag, move, rotate, duplicate, delete | Undo tested; desktop interaction check pending |
| Editor files | New/open/save/save-as, name/music/background and trigger settings | Failed-save protection tested; desktop dialogs pending |
| Rendering/audio | Original images, layered portals, wave trail, music/SFX | Linux/Windows manual pending |

## Intentional changes

- Qt widgets are replaced by Bevy UI in one window. File selection uses in-app
  path dialogs rather than native operating-system dialogs.
- Physics uses a fixed logical 1440-unit viewport height. This removes the C++
  dependency between display dimensions, floor position and collision geometry.
- All gameplay interactions run on fixed ticks. Input does not execute jumps
  outside the physics loop. Ship displacement preserves the C++ update's two
  vertical advances, with surface constraints resolved afterward.
- Rotated contacts use oriented rectangles and actual vertical surface spans.
  The legacy spike, minispike and special-object collision rectangles are retained.
- JSON IDs and sorted broad-phase order replace Qt scene iteration order.
- Practice restores trigger activation and complete color-transition timelines.
- User progress and editable copies are separated from bundled templates.
- Robot visuals remain geometric. Ball, spider and their portals have generated
  PNG artwork; the level-rating star is also a PNG. Original checkpoint and small
  cube assets render practice markers and the ship rider. See [sprite notes](sprite-prompts.md).
- Audio uses supplied Ogg copies. User music imports currently accept Ogg only.
- Ship uses the original C++ movement constants and free-flight trajectory. Cube
  parameters currently match the user-restored C++ values; algorithm differences
  remain documented in the [tuning guide](physics-tuning.md). See
  [physics notes](physics.md). Death/reveal phases use frame time, block fixed-step
  catch-up and hide the reset under a fade; no velocity ramp is applied.
- Camera depth includes the scrolling backgrounds and menu ground; gameplay ground
  uses separate color, gradient and repeating-pattern layers.
- The editor uses an icon HUD with a bottom object palette, separate build/edit/delete
  modes, hover hints, redo and independent grid snapping; see [editor controls](editor.md).
- Editor triggers use circular markers above the level while preserving JSON and
  gameplay coordinates. Ground layers render above the background in every screen.
- Escape in the main menu opens application exit confirmation, including Russian
  Cancel/Exit labels in a bundled Cyrillic font. Another Escape cancels; Enter exits. During a
  level, Escape pauses; Escape from pause returns to that level's page or the
  unsaved playtest editor. P or the play button resumes. Editor exit uses a separate
  modal with cancellation, including for unchanged levels.
- Game settings provide No Death Effect, Hide Checkpoint Buttons and separate
  menu/level music volume. The star counter always appears in menus. Profiles retain settings and
  one-time normal-completion star rewards across restarts.
- Level browsing starts on the first level when entering from the main menu and
  resets on return to the main menu. Returning directly from pause preserves the
  level just played.
- Yellow-orb input is buffered once per fresh press and consumed by jumps, pads
  and orbs. Practice checkpoints restore that input state.

## Menu presentation

The main screen uses the original icon-kit/editor artwork and circular play button
in a centered row. The background scrolls and cycles color, with a jumping cube
above the ground. The title is drawn from local pixel glyphs. Level selection uses
one page per level, side arrows, page dots and Normal/Practice progress bars.
Pause and completion retain the original graphical actions. The three original
levels appear in game order before custom levels; Left/Right and Enter also work.

Visual references: [main menu screenshot](https://steamcommunity.com/sharedfiles/filedetails/?id=2779416588&l=czech)
and [Stereo Madness selection screenshot](https://steamcommunity.com/sharedfiles/filedetails/?id=2872315073&l=dutch).

## Manual release gate

- [ ] Play all original levels and compare representative jump/flight/camera sequences with C++.
- [ ] Exercise each menu, both cube icons, and normal/practice completion.
- [ ] Check pause, death, restart and checkpoint music seeking on Linux and Windows.
- [ ] Test every editor tool, undo after dragging, settings, and save/reopen.
- [ ] Launch packaged builds from a directory other than the source tree.
- [ ] Check window resize, focus loss, low/high FPS, and asset error feedback.
- [ ] Run Git setup in a shell where `.git` and global excludes are writable.

