# Level editor HUD

The editor uses a bottom object palette and compact icon controls around the
canvas. Buttons contain artwork or vector geometry; descriptions and shortcuts
appear in a separate hover hint. Object data remains compatible with existing JSON.

- Bottom left: build (block icon), select/move (cursor), delete (trash).
- Bottom tabs: blocks/platforms, hazards, gameplay objects/portals, color/end triggers.
- Bottom thumbnails: choose an object or one of the 11 block textures, then click the canvas.
- Left: playtest, undo, redo, zoom in/out and reset view.
- Right: duplicate, delete selected, exact rotation, rotate 90 degrees, trigger settings,
  grid visibility, grid snapping and movement arrows.
- Top right: save, settings and exit. Settings contains new/open/save-as, name,
  music and background controls.

Select mode supports dragging and arrow-key movement. Shift+arrow moves one unit.
R/T rotate by 2.5 degrees; the angle icon accepts an exact value. Grid visibility
and snapping are independent. Ctrl+Z undoes; Ctrl+Shift+Z or Ctrl+Y redoes. Editing
after undo starts a new history branch. Both histories are bounded to 50 operations.

Ctrl+S saves, Ctrl+Shift+S saves as, Ctrl+O opens and F5 playtests. The pause-menu
editor button returns to the same unsaved document after playtesting. File dialogs
accept typed paths. Unsaved-discard warnings stay visible while hovering controls.

Color and end triggers appear as circular markers in rows above the level. Their
colors and BG/G/C/END labels identify their purpose. Selecting one shows a circular
outline and a vertical activation guide; dragging or moving it changes only its X
coordinate. Legacy JSON Y coordinates and gameplay activation remain unchanged.
Escape closes input dialogs. Otherwise, Escape or the editor Exit button opens
"Выйти из редактора?" with Cancel/Exit buttons, even for an unchanged level.
Unsaved documents show a discard warning. Cancel or another Escape keeps the
document; Exit returns to the main menu. Escape from a paused playtest returns to
the same unsaved document and camera position. Application exit confirmation
appears only in the main menu.

Canvas input shares the HUD's dimensions so clicks on object thumbnails, file
settings or tool buttons cannot place/delete/drag world objects. Tests check these
regions at 960×640, 1280×720 and 1920×1080, all palette categories, absence of text
inside button hierarchies, undo/redo branching and selection-mode transitions.

Layout reference: [Geometry Dash editor guide](https://www.robtopgames.com/files/GDEditor.pdf).

## Start positions

Open a level with E from selection or pause. In the editor's Triggers category,
choose START POS and place it where the player should begin. Select the marker
to change its mode and gravity with the two property buttons, or M and V.
The default Auto settings inherit earlier mode/gravity portals. F5 starts at the
selected start marker, or the last marker when another object is selected.
Deleting all start markers restores the ordinary playtest start.

Start positions are saved in level JSON and participate in undo, redo, movement,
duplication and deletion. They are editor helpers: ordinary level play ignores
them, they do not extend level length, and playtests do not award saved progress.
Restart returns to the same start position. Preceding color-trigger timelines,
the portal corridor, camera and music offset are restored at that position.
