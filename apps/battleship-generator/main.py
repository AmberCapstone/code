import argparse
import json
import math
import os
import random
from PIL import Image, ImageFilter, ImageEnhance

# ============================================================================
# Defaults / settings
# ============================================================================

IMG_W, IMG_H = 160, 120

NUM_IMAGES = 10000
TRAIN_SPLIT = 0.7
VAL_SPLIT = 0.2
TEST_SPLIT = 0.1
DATASET_DIR = "dataset"
OUTPUT_MODE = "luma"  # luma or rgb

assert abs(TRAIN_SPLIT + VAL_SPLIT + TEST_SPLIT - 1.0) < 1e-9

# ============================================================================
# Colors
# ============================================================================

BASE_BG_COLOR = (90, 160, 220)

# ============================================================================
# Ship settings
# ============================================================================

BATTLESHIP_SPRITE_DIR = "assets/battleships"
BATTLESHIP_SPRITE_SPECS = {
    "patrol.png": 2,
    "carrier.png": 2,
    "destroyer.png": 3,
    "battleship.png": 4,
}

NOTBATTLESHIP_SPRITE_DIR = "assets/notbattleships"
NOTBATTLESHIP_SPRITE_SPECS = {
    "pirate-ship-sprite.png": 4,
    "coast-guard-sprite.png": 3,
    "cargo-sprite-2.png": 6,
    "cargo-boat-sprite.png": 5,
}

SPRITE_SETS = {
    "battleship": {
        "dir": BATTLESHIP_SPRITE_DIR,
        "specs": BATTLESHIP_SPRITE_SPECS,
    },
    "notbattleship": {
        "dir": NOTBATTLESHIP_SPRITE_DIR,
        "specs": NOTBATTLESHIP_SPRITE_SPECS,
    },
}

MIN_BOATS = 0
MAX_BOATS = 4

ROTATION_RANGE_DEG = (0.0, 360.0)

# ============================================================================
# Augmentation settings
# ============================================================================

SHIP_BLUR_RADIUS_RANGE = (0.6, 1.5)
IMAGE_BLUR_RADIUS_RANGE = (0.0, 0.8)
BG_BRIGHTNESS_RANGE = (0.75, 0.95)
SHIP_BRIGHTNESS_RANGE = (0.85, 1.15)

SPRITE_SIZE_JITTER_PX = 10
PASTE_OFFSET_JITTER_PX = 1

SPRITE_PAD_X = 2
SPRITE_PAD_Y = 2

BASE_PIXELS_PER_CELL = 10
PIXELS_PER_CELL_JITTER = 2

PLACEMENT_MARGIN_PX = 1
MAX_PLACEMENT_ATTEMPTS = 200

# Ocean texture crop scaling
OCEAN_TEXTURE_SCALE_RANGE = (1.0, 1.5)
OCEAN_TEXTURE_BLUR_RANGE = (0.0, 0.6)

# ============================================================================
# CLI
# ============================================================================

def parse_args():
    parser = argparse.ArgumentParser(description="Generate synthetic boat dataset.")
    parser.add_argument(
        "--boat-set",
        choices=["battleship", "notbattleship"],
        default="notbattleship",
        help="Which sprite family to use.",
    )
    parser.add_argument(
        "--background-mode",
        choices=["color", "ocean-texture"],
        default="color",
        help="Background generation mode.",
    )
    parser.add_argument(
        "--ocean-texture-path",
        default="assets/backgrounds/ocean_texture.png",
        help="Path to ocean texture image used when background-mode=ocean-texture.",
    )
    return parser.parse_args()

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


def choose_pixels_per_cell():
    px = BASE_PIXELS_PER_CELL + random.randint(-PIXELS_PER_CELL_JITTER, PIXELS_PER_CELL_JITTER)
    return max(4, px)

# ============================================================================
# Background helpers
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


def load_ocean_texture(path):
    if not os.path.exists(path):
        raise FileNotFoundError(f"Ocean texture not found: {path}")
    return Image.open(path).convert("RGB")


def random_resized_crop(img, out_w, out_h, min_scale=1.0, max_scale=1.4):
    src_w, src_h = img.size

    scale = random.uniform(min_scale, max_scale)
    crop_w = max(out_w, int(out_w * scale))
    crop_h = max(out_h, int(out_h * scale))

    crop_w = min(crop_w, src_w)
    crop_h = min(crop_h, src_h)

    if crop_w < out_w or crop_h < out_h:
        return img.resize((out_w, out_h), Image.LANCZOS)

    max_x = src_w - crop_w
    max_y = src_h - crop_h

    x0 = random.randint(0, max_x) if max_x > 0 else 0
    y0 = random.randint(0, max_y) if max_y > 0 else 0

    crop = img.crop((x0, y0, x0 + crop_w, y0 + crop_h))
    return crop.resize((out_w, out_h), Image.LANCZOS)


def make_background(img_w, img_h, background_mode, ocean_texture=None):
    if background_mode == "ocean-texture":
        if ocean_texture is None:
            raise ValueError("ocean_texture must be provided for ocean-texture mode")

        bg = random_resized_crop(
            ocean_texture,
            img_w,
            img_h,
            min_scale=OCEAN_TEXTURE_SCALE_RANGE[0],
            max_scale=OCEAN_TEXTURE_SCALE_RANGE[1],
        )

        bg_brightness = random.uniform(*BG_BRIGHTNESS_RANGE)
        bg = ImageEnhance.Brightness(bg).enhance(bg_brightness)

        blur_r = random.uniform(*OCEAN_TEXTURE_BLUR_RANGE)
        if blur_r > 0.01:
            bg = bg.filter(ImageFilter.GaussianBlur(radius=blur_r))

        return bg

    if background_mode == "color":
        bg_brightness = random.uniform(*BG_BRIGHTNESS_RANGE)
        bg_color = jitter_color(BASE_BG_COLOR, jitter=10, brightness_scale=bg_brightness)

        bg = Image.new("RGB", (img_w, img_h), bg_color)
        bg = add_background_texture(bg)
        return bg

    raise ValueError(f"Unsupported background mode: {background_mode}")

# ============================================================================
# Ship helpers
# ============================================================================

def resize_sprite_freeform(path, ship_cells, pixels_per_cell):
    sprite = Image.open(path).convert("RGBA")

    target_w = pixels_per_cell - 2 * SPRITE_PAD_X
    target_h = ship_cells * pixels_per_cell - 2 * SPRITE_PAD_Y

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


def rotate_sprite(sprite, angle_deg):
    return sprite.rotate(angle_deg, expand=True, resample=Image.BICUBIC)


def bbox_intersects(a, b):
    ax0, ay0, ax1, ay1 = a
    bx0, by0, bx1, by1 = b
    return not (ax1 <= bx0 or bx1 <= ax0 or ay1 <= by0 or by1 <= ay0)


def bbox_inside_image(bbox, img_w, img_h, margin=0):
    x0, y0, x1, y1 = bbox
    return (
        x0 >= margin and
        y0 >= margin and
        x1 <= img_w - margin and
        y1 <= img_h - margin
    )


def choose_random_paste_xy(sprite_w, sprite_h, img_w, img_h, margin=0):
    min_x = margin
    min_y = margin
    max_x = img_w - sprite_w - margin
    max_y = img_h - sprite_h - margin

    if max_x < min_x or max_y < min_y:
        return None

    x = random.randint(min_x, max_x)
    y = random.randint(min_y, max_y)
    return x, y


def alpha_bbox_for_pasted_sprite(sprite, paste_x, paste_y):
    alpha = sprite.getchannel("A")
    bbox = alpha.getbbox()
    if bbox is None:
        return None

    x0, y0, x1, y1 = bbox
    return [paste_x + x0, paste_y + y0, paste_x + x1, paste_y + y1]


def paste_sprite(img, sprite, paste_x, paste_y):
    img.alpha_composite(sprite, (paste_x, paste_y))


def place_one_random_ship(img, occupied_bboxes, annotations, boat_set):
    sprite_dir = SPRITE_SETS[boat_set]["dir"]
    sprite_specs = SPRITE_SETS[boat_set]["specs"]

    ship_name = random.choice(list(sprite_specs.keys()))
    ship_len = sprite_specs[ship_name]
    sprite_path = os.path.join(sprite_dir, ship_name)

    for _ in range(MAX_PLACEMENT_ATTEMPTS):
        pixels_per_cell = choose_pixels_per_cell()

        sprite = resize_sprite_freeform(sprite_path, ship_len, pixels_per_cell)
        angle = random.uniform(*ROTATION_RANGE_DEG)
        sprite = rotate_sprite(sprite, angle)
        sprite = augment_ship(sprite)

        sprite_w, sprite_h = sprite.size
        xy = choose_random_paste_xy(
            sprite_w, sprite_h, img.width, img.height, margin=PLACEMENT_MARGIN_PX
        )
        if xy is None:
            continue

        paste_x, paste_y = xy
        paste_x += random.randint(-PASTE_OFFSET_JITTER_PX, PASTE_OFFSET_JITTER_PX)
        paste_y += random.randint(-PASTE_OFFSET_JITTER_PX, PASTE_OFFSET_JITTER_PX)

        bbox = alpha_bbox_for_pasted_sprite(sprite, paste_x, paste_y)
        if bbox is None:
            continue

        if not bbox_inside_image(bbox, img.width, img.height, margin=PLACEMENT_MARGIN_PX):
            continue

        overlaps = any(bbox_intersects(bbox, other) for other in occupied_bboxes)
        if overlaps:
            continue

        paste_sprite(img, sprite, paste_x, paste_y)
        occupied_bboxes.append(bbox)

        center_x = (bbox[0] + bbox[2]) / 2.0
        center_y = (bbox[1] + bbox[3]) / 2.0

        annotations.append({
            "ship": os.path.splitext(ship_name)[0],
            "sprite_file": ship_name,
            "boat_set": boat_set,
            "length_cells": ship_len,
            "rotation_degrees": angle,
            "pixels_per_cell": pixels_per_cell,
            "center_xy": [center_x, center_y],
            "sprite_bbox_xyxy": bbox,
        })

        return True

    return False


def finalize_output_image(img, output_mode="rgb"):
    rgb_img = img.convert("RGB")

    if output_mode == "rgb":
        return rgb_img

    if output_mode == "luma":
        y, _, _ = rgb_img.convert("YCbCr").split()
        return y

    raise ValueError(f"Unsupported OUTPUT_MODE: {output_mode}")

# ============================================================================
# Main
# ============================================================================

def main():
    args = parse_args()

    if args.boat_set not in SPRITE_SETS:
        raise ValueError(f"Unsupported boat set: {args.boat_set}")

    if args.background_mode == "ocean-texture":
        ocean_texture = load_ocean_texture(args.ocean_texture_path)
    else:
        ocean_texture = None

    image_train_dir = os.path.join(DATASET_DIR, "images", "train")
    image_val_dir = os.path.join(DATASET_DIR, "images", "val")
    image_test_dir = os.path.join(DATASET_DIR, "images", "test")

    label_train_dir = os.path.join(DATASET_DIR, "labels", "train")
    label_val_dir = os.path.join(DATASET_DIR, "labels", "val")
    label_test_dir = os.path.join(DATASET_DIR, "labels", "test")

    for d in [
        image_train_dir, image_val_dir, image_test_dir,
        label_train_dir, label_val_dir, label_test_dir,
    ]:
        os.makedirs(d, exist_ok=True)

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
            image_dir = image_train_dir
            label_dir = label_train_dir
        elif split == "val":
            image_dir = image_val_dir
            label_dir = label_val_dir
        else:
            image_dir = image_test_dir
            label_dir = label_test_dir

        stem = f"battleship_{img_idx:06d}"
        img_name = f"{stem}.png"
        json_name = f"{stem}.json"

        img_path = os.path.join(image_dir, img_name)
        json_path = os.path.join(label_dir, json_name)

        img = make_background(
            IMG_W,
            IMG_H,
            background_mode=args.background_mode,
            ocean_texture=ocean_texture,
        )
        img = img.convert("RGBA")

        occupied_bboxes = []
        annotations = []

        target_num_boats = random.randint(MIN_BOATS, MAX_BOATS)

        for _ in range(target_num_boats):
            placed = place_one_random_ship(
                img,
                occupied_bboxes,
                annotations,
                boat_set=args.boat_set,
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
            "boat_set": args.boat_set,
            "background_mode": args.background_mode,
            "image_size": [IMG_W, IMG_H],
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
    print(f"boat_set={args.boat_set}")
    print(f"background_mode={args.background_mode}")
    if args.background_mode == "ocean-texture":
        print(f"ocean_texture_path={args.ocean_texture_path}")
    print(f"train={num_train}, val={num_val}, test={num_test}")


if __name__ == "__main__":
    main()