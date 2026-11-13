# Level editor

The editor was compared with the installed original on 2026-11-13. Its Build,
Edit and Delete toolbars, 90-point bottom panel, side controls, pause menu and
Exit popup use extracted native sprites and bitmap fonts in a 320-point viewport.
The native categories retain the object types supported by this port. Unavailable
categories, decoration filters, warp and Platformer remain disabled.

- Build: choose blocks, hazards, pads/orbs, mode portals, color/end triggers or Start Pos.
- Edit: select or drag objects; Swipe selects a rectangle. Copy/paste and duplicate
  preserve relative positions and unknown JSON fields. Transform tools support
  flips, rotation and separate X/Y scale. Rotate also rotates group positions.
- The nine step controls choose 5 blocks, 1 block, a custom step, fractions of a
  block, or native-unit increments. The four large arrows move the selection.
- Delete: remove individual objects, all matching objects in the current layer,
  or its Start Pos markers. None/Static/custom-type filters restrict deletion.
  Undo restores a bulk deletion in one operation.
- Right-side controls provide copy/paste, trigger/Start Pos settings, group IDs,
  property/color paste, deselection and editor-layer controls. Group IDs are
  preserved as metadata; advanced group-based triggers are not implemented.
- Gear: native level-settings layout, background/ground color and song selection.
  Wrench/Options: name, new/open/save-as, music and background file controls.
- Camera: WASD or middle/right drag pans; wheel or +/- buttons zoom; 0 resets zoom.
  The top scrollbar moves along the level. View hides editor controls and blocks edits.

Ctrl+C/Ctrl+V copy and paste; Ctrl+D duplicates. Arrows move by the selected step;
Shift+arrow moves one Rust unit. R/T rotate by 2.5 degrees. Ctrl+Z undoes;
Ctrl+Shift+Z or Ctrl+Y redoes. Grid visibility and snapping are independent.
History retains 50 operations. Unknown object properties survive editing.

## Pause, saving and exit

Escape opens the native editor pause menu: Resume, Save and Play, Save and Exit,
Save, Exit. Its side panel includes settings, export, auto-save, audio and help.
Music, SFX and Editor SFX volumes are independent. Time changes playtest speed
from 0.25x to 1.75x and adjusts music playback; normal gameplay remains at 240 Hz.
Auto-save writes dirty documents every 30 seconds when enabled.

Exit opens the original silver-framed **Exit** popup: ?Exit without saving? All
unsaved changes will be lost!? with yellow/red emphasis and green **No / Yes**
buttons. No or Escape returns to pause with the document intact. Yes discards
and returns to this port's main menu. Save and Exit/Play proceed only after a
successful write. A failed write keeps the document and displays the error.

Ctrl+S saves, Ctrl+Shift+S exports to another path, Ctrl+O opens and F5 tests.
Unsaved levels receive a path under the user level directory. Bundled templates
are saved as user copies. New/Open retain discard protection. Path dialogs accept
typed paths, Ctrl+A, Enter and Escape; custom audio accepts Ogg and backgrounds PNG.

## Start positions and markers

Place Start Pos from the final supported category. Select it and use Edit Special,
M (mode) or V (gravity). Auto inherits preceding portals. F5 starts at the selected
marker, or the last one. E, the editor button or leaving a paused playtest restores
the same unsaved document. Playtests do not award progress. Restart restores the
same position, preceding color timeline, corridor, camera and music offset.

Color/end triggers retain circular editor markers and their activation guides.
Moving these markers changes X only. Existing JSON Y and gameplay activation
remain intact. Scale metadata changes both rendered size and collision geometry;
flips also reflect local collision offsets.

The full original object catalog, Platformer, advanced trigger dialogs, text/font
objects, decoration and numeric object-ID filters, online sharing and editor death
statistics are not implemented. Their visible controls remain inactive.
