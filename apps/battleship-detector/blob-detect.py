import argparse
import cv2
import numpy as np
from pathlib import Path

# --- Command-line arguments ---
def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description='Detect boats using threshold + morphology and save debug images.'
    )
    parser.add_argument('--input-image', type=Path, required=True, help='Path to input grayscale image')
    parser.add_argument('--output-dir', type=Path, required=True, help='Directory for saved stage images')
    parser.add_argument('--threshold', type=int, default=110, help='Threshold value (default: 110)')
    parser.add_argument('--kernel-size', type=int, default=3, help='Morphology kernel size (default: 3)')
    parser.add_argument('--open-iters', type=int, default=1, help='Open iterations (default: 1)')
    parser.add_argument('--close-iters', type=int, default=1, help='Close iterations (default: 1)')
    parser.add_argument('--min-area', type=int, default=8, help='Minimum blob area to keep (default: 8)')
    return parser.parse_args()


args = parse_args()
INPUT_IMAGE = args.input_image
OUTPUT_DIR = args.output_dir
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

THRESHOLD_VALUE = args.threshold

KERNEL_SIZE = args.kernel_size
OPEN_ITERS = args.open_iters
CLOSE_ITERS = args.close_iters
MIN_AREA = args.min_area


def save_image(name: str, image: np.ndarray) -> None:
    out_path = OUTPUT_DIR / name
    cv2.imwrite(str(out_path), image)
    print(f'Saved: {out_path}')


img = cv2.imread(str(INPUT_IMAGE), cv2.IMREAD_GRAYSCALE)
if img is None:
    raise FileNotFoundError(f'Could not read image: {INPUT_IMAGE}')

save_image('01_gray.png', img)


blur = cv2.GaussianBlur(img, (3, 3), 0)
save_image('02_blur.png', blur)

otsu_value, thresh = cv2.threshold(
    blur, 0, 255, cv2.THRESH_BINARY_INV | cv2.THRESH_OTSU
)
print(f'Otsu threshold = {otsu_value}')
save_image('03_threshold.png', thresh)


kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (KERNEL_SIZE, KERNEL_SIZE))

opened = cv2.morphologyEx(thresh, cv2.MORPH_OPEN, kernel, iterations=OPEN_ITERS)
save_image('04_open.png', opened)

closed = cv2.morphologyEx(opened, cv2.MORPH_CLOSE, kernel, iterations=CLOSE_ITERS)
save_image('05_close.png', closed)


num_labels, labels, stats, centroids = cv2.connectedComponentsWithStats(closed, connectivity=8)

vis = cv2.cvtColor(img, cv2.COLOR_GRAY2BGR)

print('\nDetections:')
for label in range(1, num_labels):
    x = stats[label, cv2.CC_STAT_LEFT]
    y = stats[label, cv2.CC_STAT_TOP]
    w = stats[label, cv2.CC_STAT_WIDTH]
    h = stats[label, cv2.CC_STAT_HEIGHT]
    area = stats[label, cv2.CC_STAT_AREA]
    cx, cy = centroids[label]

    if area < MIN_AREA:
        continue

    print(f'label={label:2d} area={area:3d} bbox=({x},{y},{w},{h}) centroid=({cx:.2f},{cy:.2f})')

    cv2.rectangle(vis, (x, y), (x + w - 1, y + h - 1), (0, 255, 0), 1)
    # cv2.circle(vis, (int(round(cx)), int(round(cy))), 1, (0, 0, 255), -1)

save_image('06_detections.png', vis)
print(f'\nDone. Outputs are in: {OUTPUT_DIR}')
