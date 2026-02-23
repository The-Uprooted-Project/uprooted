"""Generate og.png for uprooted.sh - 1200x630 OG image."""
from PIL import Image, ImageDraw, ImageFont
import os

W, H = 1200, 630
bg = (8, 8, 12)
text_color = (200, 204, 208)
dim_color = (90, 94, 102)
green = (106, 154, 110)
faint = (42, 45, 51)

img = Image.new("RGB", (W, H), bg)
draw = ImageDraw.Draw(img)

# Try to find a good monospace font
font_paths = [
    "C:/Windows/Fonts/CascadiaMono.ttf",
    "C:/Windows/Fonts/cascadiamono.ttf",
    "C:/Windows/Fonts/consola.ttf",     # Consolas
    "C:/Windows/Fonts/cour.ttf",        # Courier New
    "C:/Windows/Fonts/lucon.ttf",       # Lucida Console
]

font_path = None
for p in font_paths:
    if os.path.exists(p):
        font_path = p
        break

if font_path:
    font_title = ImageFont.truetype(font_path, 64)
    font_sub = ImageFont.truetype(font_path, 22)
    font_url = ImageFont.truetype(font_path, 16)
else:
    font_title = ImageFont.load_default()
    font_sub = ImageFont.load_default()
    font_url = ImageFont.load_default()

# Subtle top accent line (green)
draw.rectangle([0, 0, W, 3], fill=green)

# Subtle grid dots in background
import random
random.seed(42)
for _ in range(80):
    x = random.randint(40, W - 40)
    y = random.randint(40, H - 40)
    r = random.randint(1, 2)
    opacity = random.randint(15, 30)
    dot_color = (dim_color[0], dim_color[1], dim_color[2])
    # Approximate opacity by blending with bg
    blended = tuple(int(bg[i] + (dot_color[i] - bg[i]) * opacity / 255) for i in range(3))
    draw.ellipse([x - r, y - r, x + r, y + r], fill=blended)

# Title - "uprooted"
title = "uprooted"
bbox = draw.textbbox((0, 0), title, font=font_title)
tw = bbox[2] - bbox[0]
title_x = (W - tw) // 2
title_y = 210
draw.text((title_x, title_y), title, fill=text_color, font=font_title)

# Accent line under title
line_w = 60
line_y = title_y + (bbox[3] - bbox[1]) + 24
draw.rectangle([(W - line_w) // 2, line_y, (W + line_w) // 2, line_y + 2], fill=green)

# Subtitle
sub = "a client mod framework for root"
bbox_sub = draw.textbbox((0, 0), sub, font=font_sub)
sw = bbox_sub[2] - bbox_sub[0]
sub_y = line_y + 24
draw.text(((W - sw) // 2, sub_y), sub, fill=dim_color, font=font_sub)

# Bottom URL
url = "uprooted.sh"
bbox_url = draw.textbbox((0, 0), url, font=font_url)
uw = bbox_url[2] - bbox_url[0]
draw.text(((W - uw) // 2, H - 60), url, fill=faint, font=font_url)

# Bottom accent line
draw.rectangle([0, H - 3, W, H], fill=green)

out = os.path.join(os.path.dirname(__file__), "public", "og.png")
img.save(out, "PNG", optimize=True)
print(f"Saved {out} ({os.path.getsize(out)} bytes)")
