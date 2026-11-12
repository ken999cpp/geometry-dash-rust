# Interface reference

Main menu, level selection, gameplay HUD and pause use the installed original's
UI sprite frames and Pusab bitmap fonts. `scripts/import_native_ui.py` extracts
the named frames and font metrics from a supplied Resources directory without
changing the installation. The bundled extracts live in `assets/native_ui/`.
Re-importing requires an installed copy of Geometry Dash.

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
finishes with a bounded fine wait before presentation. Swap-chain and cleanup
work count toward the next budget; a missed deadline does not cause catch-up
bursts. The window uses AutoNoVsync to avoid adding a second independent wait.
The selected limit remains capped by the current monitor's refresh rate.

On 2026-11-13 the isolated 240 FPS presentation-gate measurement reported
239.981 FPS average, 4.1667 ms median, 4.1669 ms p99 and 4.2453 ms maximum over
480 frames. The running gameplay HUD also showed 240 FPS. This measures the
limiter on this machine; heavy workloads can still cause real dropped frames.

Run the isolated measurement with:

```sh
cargo test --lib measure_240_fps_presentation_gate -- --ignored --nocapture
```
