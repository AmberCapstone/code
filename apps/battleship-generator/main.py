import json
import os
import random
from PIL import Image, ImageDraw, ImageFilter, ImageEnhance, ImageChops
import math

# Image size: QVGA
IMG_W, IMG_H = 320, 240

# Grid settings
GRID_ROWS = 10
GRID_COLS = 10
CELL_SIZE = 20

# Base colors
BASE_GRID_LINE = (80, 80, 80)
BASE_BG_COLOR = (90, 160, 220)

# Output settings
NUM_IMAGES = 1000
TRAIN_SPLIT = 0.7
VAL_SPLIT = 0.2
TEST_SPLIT = 0.1
DATASET_DIR = "dataset"

assert abs(TRAIN_SPLIT + VAL_SPLIT + TEST_SPLIT - 1.0) < 1e-9

# Sprite settings
SPRITE_DIR = "assets/ships"
SHIP_SPECS = {
    "patrol.png": 2,
    "carrier.png": 2,
    "destroyer.png": 3,
    "battleship.png": 4,
}

# Boat count settings
MIN_BOATS = 0
MAX_BOATS = 6

# Allowed rotations
ROTATIONS = [0, 90, 180, 270]

# Augmentation settings
DRAW_GRID_PROB = 0.5
SHIP_BLUR_RADIUS_RANGE = (0.0, 1.2)
IMAGE_BLUR_RADIUS_RANGE = (0.0, 0.8)
BG_BRIGHTNESS_RANGE = (0.94, 1.12)
SHIP_BRIGHTNESS_RANGE = (0.85, 1.15)
GRID_BRIGHTNESS_RANGE = (0.75, 1.25)
SPRITE_SIZE_JITTER_PX = 10
PASTE_OFFSET_JITTER_PX = 2

# Padding
SPRITE_PAD_X = 2
SPRITE_PAD_Y = 2

# Derived grid size
GRID_W = GRID_COLS * CELL_SIZE
GRID_H = GRID_ROWS * CELL_SIZE

# Center grid in image
GRID_X0 = (IMG_W - GRID_W) // 2
GRID_Y0 = (IMG_H - GRID_H) // 2
GRID_X1 = GRID_X0 + GRID_W
GRID_Y1 = GRID_Y0 + GRID_H

# Standard dataset folders
IMAGE_TRAIN_DIR = os.path.join(DATASET_DIR, "images", "train")
IMAGE_VAL_DIR = os.path.join(DATASET_DIR, "images", "val")
IMAGE_TEST_DIR = os.path.join(DATASET_DIR, "images", "test")

LABEL_TRAIN_DIR = os.path.join(DATASET_DIR, "labels", "train")
LABEL_VAL_DIR = os.path.join(DATASET_DIR, "labels", "val")
LABEL_TEST_DIR = os.path.join(DATASET_DIR, "labels", "test")

for d in [
    IMAGE_TRAIN_DIR, IMAGE_VAL_DIR, IMAGE_TEST_DIR,
    LABEL_TRAIN_DIR, LABEL_VAL_DIR, LABEL_TEST_DIR
]:
    os.makedirs(d, exist_ok=True)


def clamp_u8(x):
    return max(0, min(255, int(round(x))))

def add_ocean_gradient(img, max_delta=18):
    """
    Add a broad smooth lighting gradient to the background.
    Positive and negative variation is centered so it does not
    systematically darken the image.
    """
    w, h = img.size

    # Pick a random gradient direction
    angle = random.uniform(0, 2 * math.pi)
    dx = math.cos(angle)
    dy = math.sin(angle)

    # Gradient layer centered around mid-gray (128)
    grad = Image.new("L", (w, h))
    pixels = []

    # Normalize coordinates around image center
    cx = (w - 1) / 2.0
    cy = (h - 1) / 2.0
    denom = max(w, h) / 2.0

    for y in range(h):
        for x in range(w):
            nx = (x - cx) / denom
            ny = (y - cy) / denom
            t = nx * dx + ny * dy   # roughly in [-1, 1]
            val = int(round(128 + max_delta * t))
            val = max(0, min(255, val))
            pixels.append(val)

    grad.putdata(pixels)
    grad_rgb = Image.merge("RGB", (grad, grad, grad))

    # Blend gently instead of add/subtract shifting
    return Image.blend(img, grad_rgb, alpha=0.12)


def add_soft_noise(img, strength=18, blur_radius=2.5):
    """
    Add visible but smooth low-frequency brightness variation.
    This modulates the existing ocean instead of blending toward gray.
    """
    w, h = img.size

    noise = Image.new("L", (w, h))
    pixels = [128 + random.randint(-strength, strength) for _ in range(w * h)]
    noise.putdata(pixels)
    noise = noise.filter(ImageFilter.GaussianBlur(radius=blur_radius))

    src = img.load()
    mod = noise.load()

    out = Image.new("RGB", (w, h))
    dst = out.load()

    for y in range(h):
        for x in range(w):
            r, g, b = src[x, y]

            # Convert 0..255 noise into roughly 0.85..1.15 scale
            factor = (mod[x, y] - 128) / 128.0
            factor = 1.0 + 0.22 * factor

            rr = clamp_u8(r * factor)
            gg = clamp_u8(g * factor)
            bb = clamp_u8(b * factor)

            dst[x, y] = (rr, gg, bb)

    return out


def add_background_texture(img):
    img = add_ocean_gradient(img, max_delta=random.randint(16, 28))
    img = add_soft_noise(
        img,
        strength=random.randint(16, 28),
        blur_radius=random.uniform(1.5, 3.0),
    )
    return img

def jitter_color(rgb, jitter=12, brightness_scale=1.0):
    r, g, b = rgb
    r = clamp_u8((r + random.randint(-jitter, jitter)) * brightness_scale)
    g = clamp_u8((g + random.randint(-jitter, jitter)) * brightness_scale)
    b = clamp_u8((b + random.randint(-jitter, jitter)) * brightness_scale)
    return (r, g, b)


def occupied_cells_for_placement(top_r, left_c, ship_len, angle):
    if angle in (0, 180):
        return [(top_r + i, left_c) for i in range(ship_len)]
    return [(top_r, left_c + i) for i in range(ship_len)]


def placement_fits(cells, occupied):
    for r, c in cells:
        if not (0 <= r < GRID_ROWS and 0 <= c < GRID_COLS):
            return False
        if (r, c) in occupied:
            return False
    return True


def find_valid_placements(ship_len, angle, occupied):
    placements = []

    if angle in (0, 180):
        max_start_r = GRID_ROWS - ship_len
        max_start_c = GRID_COLS - 1
    else:
        max_start_r = GRID_ROWS - 1
        max_start_c = GRID_COLS - ship_len

    for r in range(max_start_r + 1):
        for c in range(max_start_c + 1):
            cells = occupied_cells_for_placement(r, c, ship_len, angle)
            if placement_fits(cells, occupied):
                placements.append((r, c, cells))

    return placements


def cell_block_pixel_bbox(cells):
    rows = [r for r, _ in cells]
    cols = [c for _, c in cells]

    min_r, max_r = min(rows), max(rows)
    min_c, max_c = min(cols), max(cols)

    x0 = GRID_X0 + min_c * CELL_SIZE
    y0 = GRID_Y0 + min_r * CELL_SIZE
    x1 = GRID_X0 + (max_c + 1) * CELL_SIZE
    y1 = GRID_Y0 + (max_r + 1) * CELL_SIZE
    return [x0, y0, x1, y1]


def resize_sprite_to_cells(path, ship_cells):
    sprite = Image.open(path).convert("RGBA")

    target_w = CELL_SIZE - 2 * SPRITE_PAD_X
    target_h = ship_cells * CELL_SIZE - 2 * SPRITE_PAD_Y

    jitter_w = random.randint(0, SPRITE_SIZE_JITTER_PX)
    jitter_h = random.randint(0, SPRITE_SIZE_JITTER_PX)

    target_w = max(4, target_w + jitter_w)
    target_h = max(4, target_h + jitter_h)

    sprite.thumbnail((target_w, target_h), Image.LANCZOS)
    return sprite


def augment_ship(sprite):
    brightness = random.uniform(*SHIP_BRIGHTNESS_RANGE)
    sprite = ImageEnhance.Brightness(sprite).enhance(brightness)

    blur_r = random.uniform(*SHIP_BLUR_RADIUS_RANGE)
    if blur_r > 0.01:
        sprite = sprite.filter(ImageFilter.GaussianBlur(radius=blur_r))

    return sprite


def paste_sprite_centered_on_cells(img, sprite, cells):
    x0, y0, x1, y1 = cell_block_pixel_bbox(cells)

    block_cx = (x0 + x1) // 2
    block_cy = (y0 + y1) // 2

    sprite_w, sprite_h = sprite.size
    paste_x = block_cx - sprite_w // 2 + random.randint(-PASTE_OFFSET_JITTER_PX, PASTE_OFFSET_JITTER_PX)
    paste_y = block_cy - sprite_h // 2 + random.randint(-PASTE_OFFSET_JITTER_PX, PASTE_OFFSET_JITTER_PX)

    img.alpha_composite(sprite, (paste_x, paste_y))
    return [paste_x, paste_y, paste_x + sprite_w, paste_y + sprite_h]


def get_split(idx, num_train, num_val):
    if idx < num_train:
        return "train"
    elif idx < num_train + num_val:
        return "val"
    return "test"


def choose_random_ship_name():
    return random.choice(list(SHIP_SPECS.keys()))


def place_one_random_ship(img, occupied, annotations):
    """
    Try to place one randomly chosen ship with random rotation.
    Returns True if a ship was placed, otherwise False.
    """
    ship_name = choose_random_ship_name()
    ship_len = SHIP_SPECS[ship_name]

    angles = ROTATIONS[:]
    random.shuffle(angles)

    valid_options = []
    for angle in angles:
        placements = find_valid_placements(ship_len, angle, occupied)
        for start_r, start_c, cells in placements:
            valid_options.append((angle, start_r, start_c, cells))

    if not valid_options:
        return False

    angle, start_r, start_c, cells = random.choice(valid_options)

    for cell in cells:
        occupied.add(cell)

    sprite_path = os.path.join(SPRITE_DIR, ship_name)
    sprite = resize_sprite_to_cells(sprite_path, ship_len)
    sprite = sprite.rotate(angle, expand=True)
    sprite = augment_ship(sprite)

    sprite_bbox = paste_sprite_centered_on_cells(img, sprite, cells)
    cell_bbox = cell_block_pixel_bbox(cells)

    annotations.append({
        "ship": os.path.splitext(ship_name)[0],
        "sprite_file": ship_name,
        "length_cells": ship_len,
        "rotation_degrees": angle,
        "start_cell": [start_r, start_c],
        "occupied_cells": [[r, c] for r, c in cells],
        "cell_bbox_xyxy": cell_bbox,
        "sprite_bbox_xyxy": sprite_bbox
    })

    return True


dataset_index = {
    "train": [],
    "val": [],
    "test": []
}

num_train = int(NUM_IMAGES * TRAIN_SPLIT)
num_val = int(NUM_IMAGES * VAL_SPLIT)
num_test = NUM_IMAGES - num_train - num_val

for img_idx in range(NUM_IMAGES):
    split = get_split(img_idx, num_train, num_val)

    if split == "train":
        image_dir = IMAGE_TRAIN_DIR
        label_dir = LABEL_TRAIN_DIR
    elif split == "val":
        image_dir = IMAGE_VAL_DIR
        label_dir = LABEL_VAL_DIR
    else:
        image_dir = IMAGE_TEST_DIR
        label_dir = LABEL_TEST_DIR

    stem = f"battleship_{img_idx:06d}"
    img_name = f"{stem}.png"
    json_name = f"{stem}.json"

    img_path = os.path.join(image_dir, img_name)
    json_path = os.path.join(label_dir, json_name)

    bg_brightness = random.uniform(*BG_BRIGHTNESS_RANGE)
    bg_color = jitter_color(BASE_BG_COLOR, jitter=10, brightness_scale=bg_brightness)

    img = Image.new("RGB", (IMG_W, IMG_H), bg_color)
    img = add_background_texture(img)
    draw = ImageDraw.Draw(img)

    draw_grid = random.random() < DRAW_GRID_PROB
    grid_brightness = random.uniform(*GRID_BRIGHTNESS_RANGE)
    grid_line_color = jitter_color(BASE_GRID_LINE, jitter=8, brightness_scale=grid_brightness)

    if draw_grid:
        for c in range(GRID_COLS + 1):
            x = GRID_X0 + c * CELL_SIZE
            draw.line([(x, GRID_Y0), (x, GRID_Y1)], fill=grid_line_color, width=1)

        for r in range(GRID_ROWS + 1):
            y = GRID_Y0 + r * CELL_SIZE
            draw.line([(GRID_X0, y), (GRID_X1, y)], fill=grid_line_color, width=1)

    img = img.convert("RGBA")

    occupied = set()
    annotations = []

    target_num_boats = random.randint(MIN_BOATS, MAX_BOATS)

    for _ in range(target_num_boats):
        placed = place_one_random_ship(img, occupied, annotations)
        if not placed:
            break

    image_blur = random.uniform(*IMAGE_BLUR_RADIUS_RANGE)
    if image_blur > 0.01:
        img = img.filter(ImageFilter.GaussianBlur(radius=image_blur))

    img.convert("RGB").save(img_path)

    label_record = {
        "image": img_name,
        "split": split,
        "image_size": [IMG_W, IMG_H],
        "grid": {
            "rows": GRID_ROWS,
            "cols": GRID_COLS,
            "cell_size": CELL_SIZE,
            "grid_top_left": [GRID_X0, GRID_Y0],
            "grid_bottom_right": [GRID_X1, GRID_Y1],
            "grid_drawn": draw_grid
        },
        "objects": annotations,
        "num_objects": len(annotations)
    }

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(label_record, f, indent=2)

    dataset_index[split].append({
        "image": os.path.join("images", split, img_name),
        "label": os.path.join("labels", split, json_name)
    })

with open(os.path.join(DATASET_DIR, "dataset_index.json"), "w", encoding="utf-8") as f:
    json.dump(dataset_index, f, indent=2)

print(f"Saved dataset to '{DATASET_DIR}/'")
print(f"train={num_train}, val={num_val}, test={num_test}")