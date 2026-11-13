"""Rebuild supported game artwork from the installed game's UHD sprite frames.

Uses the native two-color icon layers and robot/spider animation descriptions;
does not alter level geometry or any files in the installed game.
"""
import json
import plistlib
import re
import sys
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageChops

source = Path(sys.argv[1] if len(sys.argv) > 1 else r"C:\Program Files (x86)\Steam\steamapps\common\Geometry Dash\Resources")
assets = Path(__file__).resolve().parents[1] / "assets"
frames = {}
for path in list(source.glob("*-uhd.plist")) + list((source / "icons").glob("*-uhd.plist")):
    # Only the default icon sets are needed by the current local garage.
    if path.parent.name == "icons" and path.stem not in [f"{s}-uhd" for s in ["player_01", "player_02", "ship_01", "dart_01", "player_ball_01", "robot_01", "spider_01"]]:
        continue
    for name, frame in plistlib.loads(path.read_bytes().lstrip()).get("frames", {}).items():
        frames[name] = (path.with_suffix(".png"), frame)


def numbers(s):
    return [float(v) for v in re.findall(r"-?\d+(?:\.\d+)?", s)]


@lru_cache(maxsize=16)
def atlas(path):
    return Image.open(path).convert("RGBA")


@lru_cache(maxsize=256)
def frame(name):
    path, f = frames[name]
    x, y, w, h = map(int, numbers(f["textureRect"]))
    a = atlas(path).crop((x, y, x + (h if f["textureRotated"] else w), y + (w if f["textureRotated"] else h)))
    if f["textureRotated"]:
        a = a.transpose(Image.Transpose.ROTATE_90)
    sw, sh = map(int, numbers(f["spriteSourceSize"]))
    ox, oy = numbers(f["spriteOffset"])
    result = Image.new("RGBA", (sw, sh))
    result.paste(a, (round((sw-w)/2+ox), round((sh-h)/2-oy)))
    return result


def centered(canvas, layer):
    canvas.alpha_composite(layer, (round((canvas.width-layer.width)/2), round((canvas.height-layer.height)/2)))


def icon(name):
    primary = frame(name)
    result = ImageChops.multiply(primary, Image.new("RGBA", primary.size, (125, 255, 0, 255)))
    secondary_name = name.replace("_001.png", "_2_001.png")
    if secondary_name in frames:
        secondary = frame(secondary_name)
        centered(result, ImageChops.multiply(secondary, Image.new("RGBA", secondary.size, (0, 255, 255, 255))))
    extra_name = name.replace("_001.png", "_extra_001.png")
    if extra_name in frames:
        centered(result, frame(extra_name))
    return result


outputs = {}


def save(path, image):
    target = assets / path
    target.parent.mkdir(parents=True, exist_ok=True)
    image.save(target)
    outputs[path] = [image.width/4, image.height/4]


for name, path in [("player_01", "cube/Cube001.png"), ("player_02", "cube/Cube002.png"), ("ship_01", "ship/Ship001.png"), ("dart_01", "wave/Wave001.png"), ("player_ball_01", "ball/Ball001.png")]:
    a = icon(name + "_001.png")
    save(path, a)
    if name.startswith("player_0"):
        save(path.replace(".png", "_small.png"), a)

# Export every native running/jump/fall frame with a common pivot and canvas.
for kind, directory in [("Robot", "robot"), ("Spider", "spider")]:
    animations = plistlib.loads((source / f"{kind}_AnimDesc.plist").read_bytes().lstrip())["animationContainer"]
    for name, parts in animations.items():
        if not any(name.startswith(f"{kind}_{state}_") for state in ["idle", "run", "jump_loop", "fall_loop"]):
            continue
        a = Image.new("RGBA", (192, 192))
        for part in sorted(parts.values(), key=lambda p: int(p["zValue"])):
            layer = icon(part["texture"])
            sx, sy = numbers(part["scale"])
            layer = layer.resize((max(1, round(layer.width*sx)), max(1, round(layer.height*sy))), Image.Resampling.LANCZOS)
            fx, fy = numbers(part["flipped"])
            if fx:
                layer = layer.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
            if fy:
                layer = layer.transpose(Image.Transpose.FLIP_TOP_BOTTOM)
            layer = layer.rotate(-float(part["rotation"]), Image.Resampling.BICUBIC, expand=True)
            x, y = numbers(part["position"])
            a.alpha_composite(layer, (round(96 + x*4 - layer.width/2), round(96 - y*4 - layer.height/2)))
        save(f"{directory}/{name}", a)
        if name == f"{kind}_idle_001.png":
            save(f"{directory}/{kind}001.png", a)

for path, name in {
    "spikes/spike.png": "spike_01_001.png",
    "spikes/minispike.png": "spike_02_001.png",
    "orbs/yellow_orb.png": "ring_01_001.png",
    "checkpoint.png": "checkpoint_01_001.png",
    "normal_level_complete_text.png": "GJ_levelComplete_001.png",
    "practice_level_complete_text.png": "GJ_practiceComplete_001.png",
}.items():
    save(path, frame(name))

# Pads are anchored at the bottom of their logical 30-unit cell. The native
# frame is tightly trimmed; stretching it to the whole cell makes a tall wall.
pad = Image.new("RGBA", (120, 120))
bump = frame("bump_01_001.png")
pad.alpha_composite(bump, (round((120-bump.width)/2), 120-bump.height))
save("jump_pads/jump_pad.png", pad)

for path, name in {
    "m_m_settings_btn": "GJ_optionsBtn_001.png",
    "mm_iconsbtn": "GJ_garageBtn_001.png",
    "mm_playbtn": "GJ_playBtn_001.png",
    "mm_editorbtn": "GJ_creatorBtn_001.png",
    "pm_editorbtn": "GJ_editBtn_001.png",
    "pm_practicebtn": "GJ_practiceBtn_001.png",
    "pm_unpracticebtn": "GJ_normalBtn_001.png",
    "pm_playbtn": "GJ_playBtn2_001.png",
    "pm_menubtn": "GJ_menuBtn_001.png",
    "pm_restartbtn": "GJ_replayBtn_001.png",
}.items():
    save(f"menubuttons/{path}.png", frame(name))
save("ui/star.png", frame("GJ_starsIcon_001.png"))

# Keep each existing block's edge configuration; only replace its raster source.
mapping = {1: "square_01", 2: "square_08", 3: "square_06", 4: "square_03", 5: "square_02", 6: "square_09", 11: "square_07"}
for index, name in mapping.items():
    save(f"blocks/block{index}.png", frame(name + "_001.png"))
for index, angles in {7: [0,270], 8:[0,270,180], 9:[0,270,180,90], 10:[0]}.items():
    a=frame("square_04_001.png")
    for angle in angles[1:]:
        a=ImageChops.lighter(a,frame("square_04_001.png").rotate(angle))
    save(f"blocks/block{index}.png",a)

# Both halves share one native portal canvas and pivot. Keep the split around
# the player, rather than drawing a complete flattened portal above the player.
for stem, number in [("Cube", 3), ("Ship", 4), ("Wave", 13), ("Ball", 7), ("Robot", 14), ("Spider", 17)]:
    back, front = [frame(f"portal_{number:02}_{side}_001.png") for side in ["back", "front"]]
    w, h = max(back.width, front.width), max(back.height, front.height)
    for side, a in [("back", back), ("front", front)]:
        canvas = Image.new("RGBA", (w,h))
        centered(canvas, a)
        save(f"portals/{stem}Portal{side}.png", canvas)
    if stem in ["Ball", "Spider"]:
        canvas = Image.new("RGBA", (w,h))
        centered(canvas, back)
        centered(canvas, front)
        save(f"portals/{stem}Portal.png", canvas)

(assets / "native_ui/game-sprites.json").write_text(json.dumps(outputs, indent=2))
print(f"Imported {len(outputs)} native game sprites and animation frames.")
