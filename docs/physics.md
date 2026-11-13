# Physics reference and validation

On 2026-11-12, the running Steam original was observed in windowed mode. A fresh
cube jump in Stereo Madness was captured at seven points. The first spike was
used as a world-distance clock, avoiding input and capture timing uncertainty.
Five airborne points fit a parabola with a maximum residual of 0.35 screen pixels.
The Rust regression projects its trajectory into the same 1042-pixel client area
and checks those points with a 1.5-pixel tolerance. Captures and analysis are local
in the ignored `dist/physics-reference/` directory.

The implementation uses normal-size, 1x GD 2.2 coefficients documented in the
[verified updateJump reconstruction](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2081/PlayerObject/updateJump.cpp),
[speed constants](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2081/PlayerObject/updateTimeMod.cpp), and
[velocity rounding](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2081/PlayerObject/setYVelocity.cpp).
These are community reconstructions, not an official engine specification.

Level files keep their 80-unit blocks; a native 30-unit block scales by 80/30.
Native Y velocity is converted using 54 ticks/s (60 × 0.9), then rounded to three
native decimal places on each velocity change. Physics still runs at 240 Hz.
Horizontal speed is 830.880288 units/s, gravity is 7450.955424 units/s² and the
cube impulse is 1609.92 units/s. A fresh jump lasts 103 ticks (0.429167 s), travels
356.586124 units and reaches 170.2584 units above its starting center. Automatic
held jumps retain one extra gravity step at takeoff, matching the original's
different handling of a fresh press and a subsequent held jump. Falling is capped
at 2160 units/s. The menu cube follows the same quantized fresh-jump trajectory.

Yellow orbs use a single pending action from a fresh press. Ground jumps and pads
consume it, so continuing to hold a previous jump cannot activate an orb later
in that flight. A fresh airborne press can remain held until an orb is reached;
release cancels it, and activating an orb consumes it. Orbs already within reach
on the press tick retain priority. The pending action and held-input state are
part of the player snapshot and restore with practice checkpoints.

Ship acceleration varies around the native falling threshold instead of using
the old C++ constant thrust and doubled Y displacement. Held acceleration uses
0.5 or 0.4 times gravity; released acceleration uses 0.32 or 0.48. Rise speed is
capped at 1152 units/s and fall speed at 921.6. Rotation approaches the actual
movement heading. Wave uses equal horizontal and vertical displacement (45°),
with smoothed rotation. Robot starts at half the cube impulse, cancels gravity
while its limited boost is held, and uses 0.9 times cube gravity. Ball switches
with 0.3 times the cube impulse (gravity flip first halves it); ball and spider
use 0.6 times cube gravity. A released robot boost cannot resume during the same
jump, and an orb/pad invalidates boost. Robot, ball and spider accept an airborne
press held until landing, consuming it once.

Surface contacts use the full player width, including leading/trailing edges,
with the native 10-point ground / 6-point ship snap thresholds. Cube staircase
landings nudge X by at most one native point to preserve the previous landing
offset on the recognized 4-block-up, 5-block-down and 3-block-double-up patterns.
The bundled Stereo Madness regression presses while falling at x=11950,
Y=1000 toward the lower platform, keeps holding through landing, catches the
first upper platform before its center enters it, and continues to the second
upper platform.

Yellow pads apply 16 native velocity units, reduced to 9.6 for ball/spider.
Yellow orbs apply 11.18, 10.062 for robot, and 7.826 for ball/spider. Flying-mode
entry and exit each halve momentum and reset rotation; a gravity portal halves
momentum only when it changes gravity. See the reconstructed
[pad modifiers](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2091/GJBaseGameLayer/getBumpMod.cpp),
[pad impulse](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2081/PlayerObject/propellPlayer.cpp),
[orbs](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2081/PlayerObject/ringJump.cpp), and
[gravity flip](https://github.com/StarryDawn72/open-source-gd-project/blob/main/versions/2.2081/PlayerObject/flipGravity.cpp).

Wave collision size is 10x10 native units; visual size is independent. The upward
pointing PNG is rotated by -90 degrees before applying the movement angle and
retains its aspect ratio. Its trail stores physics direction changes, merging
collinear movement. A joined triangle ribbon uses shared miter corners, without
round caps or point markers. Only its head interpolates; no corners are lost
between render frames. Floor/ceiling contact adds the exact intersection before
the horizontal segment, and pause/restart/checkpoint behavior is covered.

The viewport represents 320 native units vertically. The player anchor is 210
native units from the left edge, as measured in the original window. The flight
corridor is ten blocks high and follows its portal's vertical placement, rather
than a fixed global floor. Ball/spider corridors are eight blocks high. Entering
another portal of the same mode also updates these bounds. The first ship section
of Stereo Madness uses floor 420, ceiling 1220 and camera center 820 in the existing
80-unit JSON grid. The old fixed 260/1060 bounds left floor blocks floating above
the ground and hid the ceiling connections. Level object coordinates remain fixed.
Practice snapshots and editor start positions retain the corridor and camera.

Editor start markers are excluded from ordinary playback and level length.
Playtests reconstruct earlier color-trigger timelines, inherit preceding mode and
gravity portals when Auto is selected, and seek music to the matching X offset.
An explicit start mode/gravity can override that inheritance. Restart restores the
same initial snapshot. See [editor controls](editor.md) and [UI/FPS validation](interface-reference.md).

This is not a claim of complete engine equivalence. The directly measured
reference is the fresh cube jump; other movement coefficients are source-based.
The regression suite covers the six implemented modes, both gravity directions,
yellow pads/orbs, and all 36 mode-to-mode portal combinations. These checks do
not constitute a frame-by-frame original capture for every interaction.
Rotated surface contacts and the spider's hazard/surface search remain simplified.
Inverted ship motion uses symmetric 2.2
gravity rather than emulating the optional legacy gravity bug. Mini modes, other
speed portals, slopes and unsupported modes need separate implementation and
reference captures. Original level layouts are the existing port's JSON files.

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
in the menu; another Escape dismisses the dialog. Enter or the Yes button saves progress and sends the
application exit event. During a level, Escape opens pause, freezing the simulation,
attempt transition and music. Escape from pause leaves the attempt and returns to
that level's selection page, saving progress and preserving the selected level
across catalog refresh. Paused playtests return to the same unsaved editor. P or
the play button resumes a paused attempt. Escape closes path/settings input
dialogs and returns from level selection or the icon kit to the main menu. Visiting
the main menu resets level browsing to the first page; entering selection from the
main menu also selects the first level.
