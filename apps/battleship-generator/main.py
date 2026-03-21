import json
import math
import os
import random
from PIL import Image, ImageDraw, ImageFilter, ImageEnhance

# ============================================================================
# Image / dataset settings
# ============================================================================

IMG_W, IMG_H = 160, 120 

NUM_IMAGES = 100000
TRAIN_SPLIT = 0.7
VAL_SPLIT = 0.2
TEST_SPLIT = 0.1
DATASET_DIR = "dataset"
OUTPUT_MODE = "luma" # luma or rgb

assert abs(TRAIN_SPLIT + VAL_SPLIT + TEST_SPLIT - 1.0) < 1e-9

# ============================================================================
# Grid settings
# ============================================================================

GRID_ROWS = 10
GRID_COLS = 10

# Base size of one cell in pixels. This is the main scale control.
BASE_CELL_SIZE = 10

# Small per-image scale jitter so the whole grid changes size slightly.
CELL_SIZE_JITTER_PX = 2

# Randomize the grid position around center.
GRID_OFFSET_JITTER_X_PX = 4
GRID_OFFSET_JITTER_Y_PX = 4

# ============================================================================
# Colors
# ============================================================================

BASE_GRID_LINE = (80, 80, 80)
BASE_BG_COLOR = (90, 160, 220)

# ============================================================================
# Ship settings
# ============================================================================

SPRITE_DIR = "assets/ships"
SHIP_SPECS = {
    "patrol.png": 2,
    "carrier.png": 2,
    "destroyer.png": 3,
    "battleship.png": 4,
}

MIN_BOATS = 0
MAX_BOATS = 4
ROTATIONS = [0, 90, 180, 270]

# ============================================================================
# Augmentation settings
# ============================================================================

DRAW_GRID_PROB = 0
SHIP_BLUR_RADIUS_RANGE = (0.0, 1.2)
IMAGE_BLUR_RADIUS_RANGE = (0.0, 0.8)
BG_BRIGHTNESS_RANGE = (0.94, 1.12)
SHIP_BRIGHTNESS_RANGE = (0.85, 1.15)
GRID_BRIGHTNESS_RANGE = (0.75, 1.25)

SPRITE_SIZE_JITTER_PX = 10
PASTE_OFFSET_JITTER_PX = 1

SPRITE_PAD_X = 2
SPRITE_PAD_Y = 2

# ============================================================================
# Dataset directories
# ============================================================================

IMAGE_TRAIN_DIR = os.path.join(DATASET_DIR, "images", "train")
IMAGE_VAL_DIR = os.path.join(DATASET_DIR, "images", "val")
IMAGE_TEST_DIR = os.path.join(DATASET_DIR, "images", "test")

LABEL_TRAIN_DIR = os.path.join(DATASET_DIR, "labels", "train")
LABEL_VAL_DIR = os.path.join(DATASET_DIR, "labels", "val")
LABEL_TEST_DIR = os.path.join(DATASET_DIR, "labels", "test")

for d in [
    IMAGE_TRAIN_DIR, IMAGE_VAL_DIR, IMAGE_TEST_DIR,
    LABEL_TRAIN_DIR, LABEL_VAL_DIR, LABEL_TEST_DIR,
]:
    os.makedirs(d, exist_ok=True)

# ============================================================================
# Helpers
# ============================================================================

def clamp_u8(x):
    return max(0, min(255, int(round(x))))


def jitter_color(rgb, jitter=12, brightness_scale=1.0):
    r, g, b = rgb
    r = clamp_u8((r + random.randint(-jitter, jitter)) * brightness_scale)
    g = clamp_u8((g + random.randint(-jitter, jitter)) * brightness_scale)
    b = clamp_u8((b + random.randint(-jitter, jitter)) * brightness_scale)
    return (r, g, b)


def get_split(idx, num_train, num_val):
    if idx < num_train:
        return "train"
    if idx < num_train + num_val:
        return "val"
    return "test"


def choose_random_ship_name():
    return random.choice(list(SHIP_SPECS.keys()))


# ============================================================================
# Geometry helpers
# ============================================================================

def compute_grid_geometry(img_w, img_h):
    """
    Compute grid placement and scale for a single image.

    The grid is centered, then jittered slightly in x/y.
    Cell size is jittered slightly to vary overall scale.
    """
    cell_size = BASE_CELL_SIZE + random.randint(-CELL_SIZE_JITTER_PX, CELL_SIZE_JITTER_PX)
    cell_size = max(4, cell_size)

    grid_w = GRID_COLS * cell_size
    grid_h = GRID_ROWS * cell_size

    # If image size changes and grid would not fit, automatically shrink it.
    if grid_w > img_w or grid_h > img_h:
        max_cell_w = img_w // GRID_COLS
        max_cell_h = img_h // GRID_ROWS
        cell_size = max(4, min(max_cell_w, max_cell_h))
        grid_w = GRID_COLS * cell_size
        grid_h = GRID_ROWS * cell_size

    center_x0 = (img_w - grid_w) // 2
    center_y0 = (img_h - grid_h) // 2

    x_jitter = random.randint(-GRID_OFFSET_JITTER_X_PX, GRID_OFFSET_JITTER_X_PX)
    y_jitter = random.randint(-GRID_OFFSET_JITTER_Y_PX, GRID_OFFSET_JITTER_Y_PX)

    grid_x0 = center_x0 + x_jitter
    grid_y0 = center_y0 + y_jitter

    # Clamp so the whole grid remains visible
    grid_x0 = max(0, min(grid_x0, img_w - grid_w))
    grid_y0 = max(0, min(grid_y0, img_h - grid_h))

    grid_x1 = grid_x0 + grid_w
    grid_y1 = grid_y0 + grid_h

    return {
        "cell_size": cell_size,
        "grid_w": grid_w,
        "grid_h": grid_h,
        "grid_x0": grid_x0,
        "grid_y0": grid_y0,
        "grid_x1": grid_x1,
        "grid_y1": grid_y1,
    }


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


def cell_block_pixel_bbox(cells, grid_x0, grid_y0, cell_size):
    rows = [r for r, _ in cells]
    cols = [c for _, c in cells]

    min_r, max_r = min(rows), max(rows)
    min_c, max_c = min(cols), max(cols)

    x0 = grid_x0 + min_c * cell_size
    y0 = grid_y0 + min_r * cell_size
    x1 = grid_x0 + (max_c + 1) * cell_size
    y1 = grid_y0 + (max_r + 1) * cell_size

    return [x0, y0, x1, y1]


# ============================================================================
# Background texture
# ============================================================================

def add_ocean_gradient(img, max_delta=18):
    w, h = img.size

    angle = random.uniform(0, 2 * math.pi)
    dx = math.cos(angle)
    dy = math.sin(angle)

    grad = Image.new("L", (w, h))
    pixels = []

    cx = (w - 1) / 2.0
    cy = (h - 1) / 2.0
    denom = max(w, h) / 2.0

    for y in range(h):
        for x in range(w):
            nx = (x - cx) / denom
            ny = (y - cy) / denom
            t = nx * dx + ny * dy
            val = int(round(128 + max_delta * t))
            pixels.append(clamp_u8(val))

    grad.putdata(pixels)
    grad_rgb = Image.merge("RGB", (grad, grad, grad))

    return Image.blend(img, grad_rgb, alpha=0.12)


def add_soft_noise(img, strength=18, blur_radius=2.5):
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

            factor = (mod[x, y] - 128) / 128.0
            factor = 1.0 + 0.22 * factor

            dst[x, y] = (
                clamp_u8(r * factor),
                clamp_u8(g * factor),
                clamp_u8(b * factor),
            )

    return out


def add_background_texture(img):
    img = add_ocean_gradient(img, max_delta=random.randint(16, 28))
    img = add_soft_noise(
        img,
        strength=random.randint(16, 28),
        blur_radius=random.uniform(1.5, 3.0),
    )
    return img


# ============================================================================
# Ship helpers
# ============================================================================

def resize_sprite_to_cells(path, ship_cells, cell_size):
    sprite = Image.open(path).convert("RGBA")

    target_w = cell_size - 2 * SPRITE_PAD_X
    target_h = ship_cells * cell_size - 2 * SPRITE_PAD_Y

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


def paste_sprite_centered_on_cells(img, sprite, cells, grid_x0, grid_y0, cell_size):
    x0, y0, x1, y1 = cell_block_pixel_bbox(cells, grid_x0, grid_y0, cell_size)

    block_cx = (x0 + x1) // 2
    block_cy = (y0 + y1) // 2

    sprite_w, sprite_h = sprite.size
    paste_x = block_cx - sprite_w // 2 + random.randint(-PASTE_OFFSET_JITTER_PX, PASTE_OFFSET_JITTER_PX)
    paste_y = block_cy - sprite_h // 2 + random.randint(-PASTE_OFFSET_JITTER_PX, PASTE_OFFSET_JITTER_PX)

    img.alpha_composite(sprite, (paste_x, paste_y))

    return [paste_x, paste_y, paste_x + sprite_w, paste_y + sprite_h]


def place_one_random_ship(img, occupied, annotations, grid_x0, grid_y0, cell_size):
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
    sprite = resize_sprite_to_cells(sprite_path, ship_len, cell_size)
    sprite = sprite.rotate(angle, expand=True)
    sprite = augment_ship(sprite)

    sprite_bbox = paste_sprite_centered_on_cells(
        img, sprite, cells, grid_x0, grid_y0, cell_size
    )
    cell_bbox = cell_block_pixel_bbox(cells, grid_x0, grid_y0, cell_size)

    annotations.append({
        "ship": os.path.splitext(ship_name)[0],
        "sprite_file": ship_name,
        "length_cells": ship_len,
        "rotation_degrees": angle,
        "start_cell": [start_r, start_c],
        "occupied_cells": [[r, c] for r, c in cells],
        "cell_bbox_xyxy": cell_bbox,
        "sprite_bbox_xyxy": sprite_bbox,
    })

    return True

def finalize_output_image(img, output_mode="rgb"):
    """
    Convert final image to the requested output mode.

    rgb  -> standard RGB image
    luma -> extract Y channel from YCbCr and save as single-channel grayscale
    """
    rgb_img = img.convert("RGB")

    if output_mode == "rgb":
        return rgb_img

    if output_mode == "luma":
        y, _, _ = rgb_img.convert("YCbCr").split()
        return y

    raise ValueError(f"Unsupported OUTPUT_MODE: {output_mode}")



# ============================================================================
# Main generation loop
# ============================================================================

dataset_index = {
    "train": [],
    "val": [],
    "test": [],
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

    # Per-image grid geometry
    grid_geom = compute_grid_geometry(IMG_W, IMG_H)
    cell_size = grid_geom["cell_size"]
    grid_x0 = grid_geom["grid_x0"]
    grid_y0 = grid_geom["grid_y0"]
    grid_x1 = grid_geom["grid_x1"]
    grid_y1 = grid_geom["grid_y1"]

    # Background
    bg_brightness = random.uniform(*BG_BRIGHTNESS_RANGE)
    bg_color = jitter_color(BASE_BG_COLOR, jitter=10, brightness_scale=bg_brightness)

    img = Image.new("RGB", (IMG_W, IMG_H), bg_color)
    img = add_background_texture(img)
    draw = ImageDraw.Draw(img)

    # Grid
    draw_grid = random.random() < DRAW_GRID_PROB
    grid_brightness = random.uniform(*GRID_BRIGHTNESS_RANGE)
    grid_line_color = jitter_color(BASE_GRID_LINE, jitter=8, brightness_scale=grid_brightness)

    if draw_grid:
        for c in range(GRID_COLS + 1):
            x = grid_x0 + c * cell_size
            draw.line([(x, grid_y0), (x, grid_y1)], fill=grid_line_color, width=1)

        for r in range(GRID_ROWS + 1):
            y = grid_y0 + r * cell_size
            draw.line([(grid_x0, y), (grid_x1, y)], fill=grid_line_color, width=1)

    img = img.convert("RGBA")

    occupied = set()
    annotations = []

    target_num_boats = random.randint(MIN_BOATS, MAX_BOATS)

    for _ in range(target_num_boats):
        placed = place_one_random_ship(
            img,
            occupied,
            annotations,
            grid_x0,
            grid_y0,
            cell_size,
        )
        if not placed:
            break

    image_blur = random.uniform(*IMAGE_BLUR_RADIUS_RANGE)
    if image_blur > 0.01:
        img = img.filter(ImageFilter.GaussianBlur(radius=image_blur))

    final_img = finalize_output_image(img, OUTPUT_MODE)
    final_img.save(img_path)

    label_record = {
        "image": img_name,
        "split": split,
        "image_size": [IMG_W, IMG_H],
        "grid": {
            "rows": GRID_ROWS,
            "cols": GRID_COLS,
            "cell_size": cell_size,
            "grid_top_left": [grid_x0, grid_y0],
            "grid_bottom_right": [grid_x1, grid_y1],
            "grid_drawn": draw_grid,
        },
        "objects": annotations,
        "num_objects": len(annotations),
    }

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(label_record, f, indent=2)

    dataset_index[split].append({
        "image": os.path.join("images", split, img_name),
        "label": os.path.join("labels", split, json_name),
    })

with open(os.path.join(DATASET_DIR, "dataset_index.json"), "w", encoding="utf-8") as f:
    json.dump(dataset_index, f, indent=2)

print(f"Saved dataset to '{DATASET_DIR}/'")
print(f"train={num_train}, val={num_val}, test={num_test}")