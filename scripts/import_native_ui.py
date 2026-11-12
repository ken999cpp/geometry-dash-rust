"""Import the UI references from the user's installed Geometry Dash resources.

Usage: python scripts/import_native_ui.py [Resources-directory]
Only named UI frames/fonts are copied; installed game files are never changed.
"""
import json
import plistlib
import re
import shutil
import sys
from pathlib import Path
from PIL import Image

source = Path(sys.argv[1] if len(sys.argv) > 1 else r"C:\Program Files (x86)\Steam\steamapps\common\Geometry Dash\Resources")
destination = Path(__file__).resolve().parents[1] / "assets/native_ui"
destination.mkdir(parents=True, exist_ok=True)
names = """
GJ_logo_001.png GJ_playBtn_001.png GJ_garageBtn_001.png GJ_creatorBtn_001.png
GJ_optionsBtn_001.png GJ_statsBtn_001.png GJ_achBtn_001.png GJ_moreGamesBtn_001.png
GJ_musicLibraryBtn_001.png GJ_closeBtn_001.png GJ_dailyRewardBtn_001.png GJ_dailyBtn_001.png GJ_newsBtn_001.png
GJ_ngBtn_001.png
gj_fbIcon_001.png gj_twIcon_001.png gj_ytIcon_001.png gj_twitchIcon_001.png gj_discordIcon_001.png robtoplogo_small.png
GJ_backBtn_001.png GJ_arrow_01_001.png GJ_arrow_03_001.png GJ_topBar_001.png GJ_sideArt_001.png
difficulty_01_btn_001.png difficulty_02_btn_001.png difficulty_03_btn_001.png GJ_starsIcon_001.png
secretCoinUI_001.png secretCoinUI2_001.png currencyOrbIcon_001.png GJ_infoIcon_001.png GJ_deleteBtn_001.png
GJ_levelLeaderboardBtn_001.png GJ_editBtn_001.png GJ_playBtn2_001.png GJ_practiceBtn_001.png
GJ_normalBtn_001.png GJ_menuBtn_001.png GJ_replayBtn_001.png GJ_audioOnBtn_001.png GJ_musicOnBtn_001.png
GJ_sfxIcon_001.png GJ_musicIcon_001.png GJ_pauseBtn_001.png GJ_checkpointBtn_001.png GJ_removeCheckBtn_001.png
GJ_checkOn_001.png GJ_checkOff_001.png hideBtn_001.png edit_eStartPosBtn_001.png
diffIcon_01_btn_001.png diffIcon_02_btn_001.png diffIcon_03_btn_001.png secretCoin_b_01_001.png
GJ_arrow_02_001.png navArrowBtn_001.png sideBtn_death_001.png sideBtn_hide_001.png sideBtn_quests_001.png
gjCoinUI_01_off_001.png gjCoinUI_02_off_001.png
""".split()
frames = {}
for plist in source.glob("*-uhd.plist"):
    data = plistlib.loads(plist.read_bytes())
    for name, frame in data.get("frames", {}).items():
        if name in names:
            frames[name] = (plist.with_suffix(".png"), frame)

sizes = {}
def numbers(value):
    return [int(v) for v in re.findall(r"-?\d+", value)]
for name in names:
    if name not in frames:
        print("Missing frame:", name)
        continue
    atlas_path, frame = frames[name]
    atlas = Image.open(atlas_path).convert("RGBA")
    x, y, width, height = numbers(frame["textureRect"])
    if frame["textureRotated"]:
        crop = atlas.crop((x, y, x + height, y + width)).transpose(Image.Transpose.ROTATE_90)
    else:
        crop = atlas.crop((x, y, x + width, y + height))
    source_width, source_height = numbers(frame["spriteSourceSize"])
    offset_x, offset_y = numbers(frame["spriteOffset"])
    canvas = Image.new("RGBA", (source_width, source_height))
    canvas.paste(crop, (round((source_width - width) / 2 + offset_x), round((source_height - height) / 2 - offset_y)))
    canvas.save(destination / name)
    sizes[name] = [source_width / 4, source_height / 4]

fonts = {}
for name in ["bigFont", "goldFont"]:
    fnt = (source / f"{name}-uhd.fnt").read_text()
    common = dict(re.findall(r"(\w+)=(-?\d+)", next(line for line in fnt.splitlines() if line.startswith("common "))))
    glyphs = {}
    kerning = {}
    for line in fnt.splitlines():
        fields = {key: int(value) for key, value in re.findall(r"(\w+)=(-?\d+)", line)}
        if line.startswith("char "):
            glyphs[str(fields.pop("id"))] = fields
        elif line.startswith("kerning "):
            kerning[f"{fields['first']}:{fields['second']}"] = fields["amount"]
    fonts[name] = {"line_height": int(common["lineHeight"]), "glyphs": glyphs, "kerning": kerning}
    shutil.copyfile(source / f"{name}-uhd.png", destination / f"{name}.png")
(destination / "fonts.json").write_text(json.dumps(fonts, separators=(",", ":")))
(destination / "sprites.json").write_text(json.dumps(sizes, indent=2))
for name, target in [("game_bg_01_001-uhd.png", "background.png"), ("groundSquare_01_001-uhd.png", "ground.png"), ("GJ_gradientBG-hd.png", "gradient.png"), ("slidergroove-uhd.png", "slidergroove.png"), ("sliderthumb-uhd.png", "sliderthumb.png"), ("sliderBar-uhd.png", "sliderBar.png")]:
    imported=Image.open(source / name).convert("RGBA")
    if target.startswith("slider"):
        alpha=imported.getchannel("A").point(lambda a: 0 if a < 8 else a)
        imported.putalpha(alpha)
    imported.save(destination / target)
print(f"Imported {len(sizes)} sprites and two bitmap fonts.")
