# Generated sprite assets

Generated with the built-in ImageGen tool, with `transparent_background: true`.
Existing assets were retained. Each final prompt uses the following exact template,
with `{subject}` replaced by one of the five subjects below:

> Asset type: production PNG game sprite on a genuinely transparent background. {subject} Center exactly one isolated asset, tightly framed with 5% clear padding. Sharp smooth edges, no blur, no gradients fading into a background, no drop shadow. The sprite should fill the canvas and remain legible when rendered at 24 to 80 pixels. Match the bold black-outlined lime/cyan/yellow geometric visual language of classic Geometry Dash UI and character sprites.

## star

One single five-point golden yellow rating star sprite for a Geometry Dash-style 2D game HUD. Bold nearly-black outline, small white highlight, saturated yellow upper facets and orange lower facets, clean symmetric recognizable silhouette. No face, letters, numbers, shadow outside the silhouette or additional items.

## ball

One single circular rolling ball player icon sprite for a Geometry Dash-style 2D game. A saturated lime-green circular gear with eight broad pointed teeth, thick nearly-black outline, white inner edge and cyan inner ring, simple black geometric center with lime facets. Crisp geometric cartoon game art, symmetric compact silhouette. No face, text, floor, motion trail or additional items.

## spider

One single side-view spider player icon sprite for a Geometry Dash-style 2D game, facing right. Squat angular lime-green mechanical head/body, two bright cyan rectangular eyes, black outlines with white edge accents, four compact angular jointed legs below the body, compact broad silhouette. Crisp geometric cartoon game art, readable as an angular mechanical spider at 64 pixels. No scenery, text, floor, trail or additional items.

## ball_portal

One single vertical ball-mode portal sprite for a Geometry Dash-style 2D game. Orange and golden-yellow upright elongated oval ring with a hollow fully transparent center, thick nearly-black contour and thin white highlights, symmetric three small angular block projections on each side. Include one small orange circular cog emblem integrated at the TOP of the ring to identify rolling-ball mode; the main hollow center must stay empty. Front-facing flat 2D geometric cartoon game art. No letters, numbers, glow cloud, ground or additional items.

## spider_portal

One single vertical spider-mode portal sprite for a Geometry Dash-style 2D game. Saturated magenta/purple upright elongated oval ring with a hollow fully transparent center, thick nearly-black contour and thin white highlights, symmetric three small angular block projections on each side. Include one small angular purple mechanical-spider emblem integrated at the TOP of the ring to identify spider mode; the main hollow center must stay empty. Front-facing flat 2D geometric cartoon game art. No letters, numbers, glow cloud, ground or additional items.

## Saved files

- `assets/ui/star.png`
- `assets/ball/Ball001.png`
- `assets/spider/Spider001.png`
- `assets/portals/BallPortal.png`
- `assets/portals/SpiderPortal.png`

The portals have transparent centers. These images are consumed by the level
selection UI, player rendering and editor palette. The checkpoint and ship rider
reuse the original PNG assets.
