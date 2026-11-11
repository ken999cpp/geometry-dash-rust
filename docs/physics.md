# Cube, ship and attempt timing

Movement is based on the sibling C++ project: `headers/constants.h`,
`src/cube.cpp`, `src/ship.cpp` and `src/wave.cpp`. The Rust parameters are
centralized in [`src/constants.rs`](../src/constants.rs); the GD 2.2 conversion
and the previous speed multipliers have been removed. All movement, rotation,
hitbox, camera and transition parameters are editable there. See the
[manual tuning guide](physics-tuning.md) for a parameter map and algorithm differences.

Horizontal speed is 900 units/s. The current user-edited cube gravity is 9450
units/s² and jump impulse is 1850 units/s. These values are preserved. Gravity
applies before vertical displacement on every tick, including takeoff, with no
fall-speed cap. A flat-ground jump lasts 93 ticks, travels 348.75 units and reaches
an apex of 177.231771 units. The menu cube uses the
same discrete trajectory expressed analytically.

Yellow orbs use a single pending action from a fresh press. Ground jumps and pads
consume it, so continuing to hold a previous jump cannot activate an orb later
in that flight. A fresh airborne press can remain held until an orb is reached;
release cancels it, and activating an orb consumes it. Orbs already within reach
on the press tick retain priority. The pending action and held-input state are
part of the player snapshot and restore with practice checkpoints.

Ship gravity is 1800 units/s² while airborne; held thrust is 4300 units/s².
Vertical velocity is capped at ±500 units/s. The C++ update computes two vertical
displacements from that velocity; this free-flight trajectory is preserved.
Surface constraints are resolved once afterward, preserving the Rust collision
system. Rotation follows velocity × 0.07, interpolated at 20/s. Regression tests
compare each free-flight tick against an independent translation of ship.cpp.

Wave retains the original 1400 / √2 vertical speed. Robot, ball and spider were
introduced by the Rust port and keep their existing behavior. Input and collisions
remain deterministic at 240 Hz, independent of rendering and display dimensions.

Death stops the previous music sink on the collision tick, including pausing a
pending playback before a sink has loaded. Desktop attempt presentation advances
once per rendered frame using real time, separately from the fixed simulation.
The death effect lasts one second; a fade covers the reset, followed by a stationary
0.30-second reveal. Physics and music resume together after that reveal. Large
frame deltas cannot skip these visible phases, and repeated restart input during
death/reveal cannot bypass them. The first ready frame drains its fixed-step batch
without moving the player, preventing catch-up motion on the new attempt.

The camera follows the player from the first live frame at a constant horizontal
anchor; it no longer switches abruptly from a stationary camera to scrolling.
There is no speed ramp. Restarts reuse static level entities and clear transient
trails; the previous/current player positions are reset together under the fade.
Practice retains the saved checkpoint velocity and music offset. Death SFX remains
separate from music.

No Death Effect disables the death ring and particles. The death sound, one-second
wait and spawn fade/reveal remain unchanged. Separate menu/level music volume
settings range from 0% to 100%; existing and pending playback are updated without
restarting the track. Settings and earned stars are saved in `profile.json`.

Regression checks cover jump range/apex, ship takeoff/reversal/caps/inverted
gravity, frame-rate independence, death timing, the pending-audio case and a full
desktop attempt transition subjected to thousands of fixed catch-up steps.

# Scenery

The 2D camera stays at Z=0 so the scenery lies inside its clipping planes.
Backgrounds use Z=-200; ground layers use Z=80 through 83, above the background
and world objects, in both menus and levels. Backgrounds use repeated panels with a 50-unit/s parallax
scroll. The ground has separate color, gradient, repeating pattern and surface-line
layers. Level scroll derives from simulation time, so it freezes on pause/death and
restores consistently after a checkpoint. Menu scenery uses menu time.

Escape opens application exit confirmation only in the main menu. Cancel stays
in the menu; another Escape dismisses the dialog. Enter or the Exit button saves progress and sends the
application exit event. During a level, Escape opens pause, freezing the simulation,
attempt transition and music. Escape from pause leaves the attempt and returns to
that level's selection page, saving progress and preserving the selected level
across catalog refresh. Paused playtests return to the same unsaved editor. P or
the play button resumes a paused attempt. Escape closes path/settings input
dialogs and returns from level selection or the icon kit to the main menu. Visiting
the main menu resets level browsing to the first page; entering selection from the
main menu also selects the first level.
