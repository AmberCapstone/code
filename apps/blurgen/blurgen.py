import argparse
import math
import os
import sys
from pathlib import Path
import cv2
import numpy as np

RESOLUTIONS = {
    "VGA": (640, 480),
    "QVGA": (320, 240),
    "QQVGA": (160, 120),
}

FORMATS = [
    "PGM",
    "PPM",
    "PNG",
    "YUV422",
]


def resolution_type(value: str) -> tuple[int, int]:
    # Named preset
    value = value.upper()
    if value in RESOLUTIONS:
        return RESOLUTIONS[value]

    # Try parsing w,h
    try:
        w, h = map(int, value.split(","))
        if w <= 0 or h <= 0:
            raise ValueError("Dimensions must be positive")
        return (w, h)
    except Exception:
        raise argparse.ArgumentTypeError(
            "Resolution must be WIDTH,HEIGHT or one of: " + " ".join(RESOLUTIONS)
        )


def parse():
    p = argparse.ArgumentParser(
        "BlurGen", description="Generate blurry images for sharpness adjustment."
    )
    p.add_argument(
        "resolution",
        type=resolution_type,
        help="Resolution. WIDTH,HEIGHT or one of: " + " ".join(RESOLUTIONS),
    )
    p.add_argument(
        "-b",
        "--blurs",
        type=int,
        nargs="+",
        help="Gaussian blur radius. Pass multiple blurs to generate multiple images.",
        default=[0],
    )
    p.add_argument("-s", "--size", type=int, help="Checker pattern size (px)")
    p.add_argument(
        "-f",
        "--fmt",
        type=str.upper,
        choices=FORMATS,
        help="Output format.",
        default="PNG",
    )
    p.add_argument(
        "-o",
        "--output",
        type=Path,
        default="out",
        help="Output filename. Default 'out'",
    )

    args = p.parse_args(sys.argv[1:] or ["-h"])
    args.width, args.height = args.resolution

    if args.size is None:
        args.size = math.ceil(args.resolution[0] / 8)

    suffix = args.output.suffix
    desired_suffix = "." + args.fmt.lower()
    if suffix != "" and suffix != desired_suffix:
        print(f"Output extension will be changed from {suffix} to {desired_suffix}")

    return args


def generate_checker(width: int, height: int, size: int) -> np.ndarray:
    x, y = np.meshgrid(range(width), range(height))
    squares = (x // size + y // size) % 2
    return 255 * squares.astype(np.uint8)


def save_image(img: np.ndarray, filename: Path, detail: str, format: str):
    filename = filename.with_stem(filename.stem + detail).with_suffix(
        "." + format.lower()
    )
    os.makedirs(filename.parent, exist_ok=True)

    match format.upper():
        case "PGM" | "PNG":
            cv2.imwrite(filename, img)

        case "PPM":
            cv2.imwrite(filename, cv2.cvtColor(img, cv2.COLOR_GRAY2BGR))

        case "YUV422":
            # YUV422 is [Y1, U12, Y2, V12]. For grayscale, U=V=0
            # Interleave columns of zeros between the Ys
            yuv = np.dstack((img, np.zeros_like(img))).reshape(img.shape[0], -1)

            with open(filename, "wb") as f:
                f.write(yuv.tobytes())

        case _:
            raise ValueError(f"Invalid format {format}")

    print("Saved", filename)


def main():
    args = parse()

    checker = generate_checker(args.width, args.height, args.size)

    for blur in args.blurs:
        kernel = blur * 2 + 1
        blurred = cv2.GaussianBlur(
            checker,
            ksize=(kernel, kernel),
            sigmaX=blur,
            sigmaY=blur,
            borderType=cv2.BORDER_WRAP,
        )

        # If we're generating multiple images, append the blur amount to the filename
        detail = f"_b{blur}" if len(args.blurs) > 1 else ""

        save_image(blurred, args.output, detail, args.fmt)


if __name__ == "__main__":
    main()
