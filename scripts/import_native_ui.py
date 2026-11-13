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
GJ_table_top_001.png GJ_table_bottom_001.png GJ_table_side_001.png chain_01_001.png
GJ_longBtn01_001.png GJ_longBtn02_001.png GJ_levelComplete_001.png GJ_practiceComplete_001.png
GJ_undoBtn_001.png GJ_redoBtn_001.png GJ_resetBtn_001.png GJ_zoomInBtn_001.png GJ_zoomOutBtn_001.png
GJ_playMusicBtn_001.png GJ_playEditorBtn_001.png GJ_pauseEditorBtn_001.png GJ_optionsBtn02_001.png
GJ_copyBtn2_001.png GJ_pasteBtn2_001.png GJ_duplicateObjectBtn2_001.png
GJ_editObjBtn4_001.png GJ_groupIDBtn_001.png GJ_editObjBtn3_001.png
GJ_copyStateBtn_001.png GJ_pasteStateBtn_001.png GJ_pasteColorBtn_001.png GJ_editHSVBtn2_001.png
GJ_groupIDBtn2_001.png GJ_deSelBtn2_001.png
GJ_tabOff_001.png GJ_tabOn_001.png GJ_updateBtn_001.png GJ_viewLevelsBtn_001.png
edit_gridBtn_001.png edit_buildBtn_001.png edit_editBtn_001.png edit_deleteBtn_001.png edit_viewBtn_001.png
edit_swipeBtn_001.png edit_enableRotateBtn_001.png edit_freeMoveBtn_001.png edit_snapBtn_001.png
edit_flipXBtn_001.png edit_flipYBtn_001.png edit_cwBtn_001.png edit_ccwBtn_001.png
edit_rotate45lBtn_001.png edit_rotate45rBtn_001.png edit_freeRotateBtn_001.png edit_rotateSnapBtn_001.png
edit_upBtn_001.png edit_upBtn2_001.png edit_upBtn3_001.png edit_upBtn5_001.png
edit_downBtn_001.png edit_downBtn2_001.png edit_downBtn3_001.png edit_downBtn5_001.png
edit_leftBtn_001.png edit_rightBtn_001.png edit_scaleBtn_001.png edit_scaleXYBtn_001.png edit_warpBtn_001.png
edit_vLine_001.png edit_eChangeBG_001.png edit_groundBtn_001.png edit_objInfoBtn_001.png
GJ_colorBtn_001.png GJ_plainBtn_001.png GJ_audioOffBtn_001.png GJ_helpBtn_001.png
GJ_goToLayerBtn_001.png GJ_sSettingsIcon_001.png GJ_sExportIcon_001.png GJ_sSFXIcon_001.png
GJ_sSaveIcon_001.png GJ_sAudioDLIcon_001.png GJ_sHelpIcon_001.png GJ_deleteIcon_001.png
edit_delBtn_001.png edit_findBtn_001.png GJ_deleteAllIcon_001.png
deleteFilter_none_001.png deleteFilter_static_001.png deleteFilter_details_001.png deleteFilter_custom_001.png
boost_01_001.png
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
for name in ["bigFont", "goldFont", "chatFont"]:
    fnt = (source / f"{name}-uhd.fnt").read_text(encoding="utf-8")
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
standalone = [("game_bg_01_001-uhd.png", "background.png"), ("groundSquare_01_001-uhd.png", "ground.png"), ("GJ_gradientBG-hd.png", "gradient.png"), ("slidergroove-uhd.png", "slidergroove.png"), ("sliderthumb-uhd.png", "sliderthumb.png"), ("sliderBar-uhd.png", "sliderBar.png")]
standalone += [(f"GJ_button_{n:02}-uhd.png", f"GJ_button_{n:02}.png") for n in [1, 2, 4, 5]]
standalone += [(f"GJ_square{n:02}-uhd.png", f"GJ_square_{n:02}.png") for n in [1, 7]]
standalone += [("square01_001-uhd.png", "dialog_panel.png")]
for name, target in standalone:
    imported=Image.open(source / name).convert("RGBA")
    if target.startswith("slider"):
        alpha=imported.getchannel("A").point(lambda a: 0 if a < 8 else a)
        imported.putalpha(alpha)
    imported.save(destination / target)
    sizes[target] = [imported.width / 4, imported.height / 4]
def nine_slice(image, size, border):
    """Preserve the native border when making the editor's long surfaces."""
    result=Image.new("RGBA",size)
    sx=[0,border,image.width-border,image.width]
    sy=[0,border,image.height-border,image.height]
    dx=[0,border,size[0]-border,size[0]]
    dy=[0,border,size[1]-border,size[1]]
    for y in range(3):
        for x in range(3):
            part=image.crop((sx[x],sy[y],sx[x+1],sy[y+1]))
            part=part.resize((dx[x+1]-dx[x],dy[y+1]-dy[y]),Image.Resampling.LANCZOS)
            result.paste(part,(dx[x],dy[y]))
    return result

for name,source_name,size,border in [("editor_green.png","GJ_button_01.png",(780,112),40),("editor_gray.png","GJ_button_05.png",(320,104),40),("editor_time.png","GJ_square_01.png",(140,580),40)]:
    nine_slice(Image.open(destination/source_name),size,border).save(destination/name)
    sizes[name]=[size[0]/4,size[1]/4]
(destination / "sprites.json").write_text(json.dumps(sizes, indent=2))
print(f"Imported {len(sizes)} sprites and {len(fonts)} bitmap fonts.")
