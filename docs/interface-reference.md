# Interface reference

Main menu, level selection, gameplay HUD and pause use the installed original's
UI sprite frames and Pusab bitmap fonts. `scripts/import_native_ui.py` extracts
the named frames and font metrics from a supplied Resources directory without
changing the installation. The bundled extracts live in `assets/native_ui/`.
Re-importing requires an installed copy of Geometry Dash.

The main-menu Quit Game dialog uses `square01_001-uhd.png` from the installation:
a silver frame around the blue panel, a gold title, Helvetica Neue bitmap text
from `chatFont-uhd.fnt`, red emphasis on “quit”, and green Cancel/Yes buttons.
Its 300x140-point layout was compared with the original's exit popup. Escape or
Cancel dismisses it; Enter or Yes confirms application exit. The editor keeps
its separate unsaved-change confirmation.

UI dimensions use a 320-point vertical viewport. Positions and dimensions use
viewport-height units, including font glyphs, so resizing keeps the original
proportions. The menus were compared with 1852x1073 window captures, then checked
in the Rust window at 1280x720. Local references are in the ignored
`dist/menu-reference/` directory.

The main menu uses the original logo, cross-shaped play button, icon/editor
buttons, social buttons, achievement/settings/stats/Newgrounds row and daily/news
art. Level selection uses original difficulty faces, rating stars, gray coins,
white navigation arrows, border decorations, bitmap labels and progress bars.
It displays the levels and progress present in this port. E opens the selected
level in the editor. Pause uses the original panel proportions, four main
buttons, side controls, music/SFX sliders, best/current progress and coins.
E also opens the editor from pause. Online community/reward services are not
implemented; local stats panels report this game's saved data.

The gameplay HUD centers progress at the top. Its FPS counter measures a rolling
second of render-thread presentation intervals, rather than reciprocating one
main-thread delta. The limiter sleeps for the bulk of each frame budget and
finishes with a bounded fine wait before presentation. Normal swap-chain overhead remains inside the gate-to-gate budget. A long
present or OS stall after the gate starts a fresh interval; missed deadlines do
not cause catch-up bursts. The window defaults to AutoNoVsync to avoid adding a second independent wait;
VSync can be selected explicitly in Advanced.
The selected limit remains capped by the current monitor's refresh rate.

On 2026-11-13 the isolated 240 FPS presentation-gate measurement reported
239.981 FPS average, 4.1667 ms median, 4.1669 ms p99 and 4.2453 ms maximum over
480 frames. The running gameplay HUD also showed 240 FPS. This measures the
limiter on this machine; heavy workloads can still cause real dropped frames.

Run the isolated measurement with:

```sh
cargo test --lib measure_240_fps_presentation_gate -- --ignored --nocapture
```


## Settings and supported gameplay art

Settings was compared again with the installed original on 2026-11-13. The main
page uses the original hanging chains, table borders, green button surfaces,
bitmap labels, music/SFX sliders and Menu Music checkbox. Options uses the
original seven-tab arrangement, two-column checkboxes and information buttons.
Video Options uses the brown native frame, resolution arrows and explicit Apply;
Advanced provides VSync, Show FPS and the existing refresh-capped 60/120/144/240
choices. Escape returns one page at a time. Texture quality is High because the
bundled native extracts are UHD.

Cursor visibility/confinement, pause-button side, progress bar, percentage,
elapsed time, auto-retry, faster reset and the existing death/practice controls
are saved settings. Menu, level and SFX volume remain independent; the main
settings Music slider controls level music and Audio exposes all three channels.
Inactive options for unsupported input modes remain disabled, with information
buttons. Account/community pages describe local functionality; they do not
pretend to connect to the original's servers. The Editor/Keys tabs currently
show this port's controls, and Other shows local game information.

`scripts/import_native_game.py` reconstructs supported game art from the native
UHD frames: two-color cube/ship/wave/ball icons, robot/spider poses from the
original animation descriptions, blocks, spikes, pads, orbs, checkpoints,
portal halves, completion labels and the remaining legacy menu buttons.
`assets/native_ui/game-sprites.json` records each exported canvas in native
units. Pads retain a full transparent logical cell so their trimmed frame is
not stretched into a tall object. Portal layers share a pivot and render behind
and in front of the player. Physics and level coordinates are unchanged.


## Editor reference

The installed original's Build/Edit/Delete, level settings, editor pause and Exit
were inspected on 2026-11-13. The editor uses native undo/redo, music/test, zoom,
property, mode and transform frames, bottom tabs, native button surfaces and the
black world grid. Pause follows the five-button center column, seven gray side
buttons with white labels/icons, Time panel and three audio sliders. Editor Exit
uses the silver 300x140 panel, gold title, Chat font, yellow ?Exit?, red ?lost? and
No/Yes buttons. Cancel returns to pause; save failures cannot discard a document.

The importer currently exports 166 UI surfaces plus three fonts and 92 gameplay
frames/poses. The three long editor surfaces are sliced from the original button
images with their border pixels preserved. The default editor background uses the
full square native texture. Robot portals now use their native front/back layers.
The object catalog and advanced dialogs remain limited to supported port features;
[editor.md](editor.md) documents those limits and working controls.

The presentation regression also injects a 12 ms stall between the gate and
present. The following frame still waits a full selected interval.
