import argparse
from dataclasses import dataclass
from pathlib import Path

import cv2
import numpy as np

IMAGE_EXTENSIONS = {".jpg", ".jpeg", ".png", ".bmp", ".tiff", ".tif", ".webp"}


@dataclass(frozen=True)
class SobelOutputs:
    gx: np.ndarray
    gy: np.ndarray
    mag: np.ndarray
    valid: np.ndarray


def load_grayscale_image(path: Path) -> np.ndarray:
    if not path.exists():
        raise FileNotFoundError(f"Input image not found: {path}")

    img = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
    if img is None:
        raise ValueError(f"Failed to read image: {path}")

    if img.ndim == 2:
        gray = img
    else:
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    if gray.dtype != np.uint8:
        gray = np.clip(gray, 0, 255).astype(np.uint8)

    return gray


def sobel_streaming_fpga_like(y_frame: np.ndarray) -> SobelOutputs:
    """
    FPGA-style streaming Sobel:
      - 2 line buffers
      - 3 shift registers per row
      - output aligns to window center
      - valid only for inner pixels
      - gx = (c + 2f + i) - (a + 2d + g)
      - gy = (g + 2h + i) - (a + 2b + c)
      - mag = |gx| + |gy|
    """
    if y_frame.ndim != 2:
        raise ValueError("Input must be a 2D grayscale image")

    H, W = y_frame.shape

    gx_img = np.zeros((H, W), dtype=np.int16)
    gy_img = np.zeros((H, W), dtype=np.int16)
    mag_img = np.zeros((H, W), dtype=np.uint16)
    valid = np.zeros((H, W), dtype=bool)

    prev2 = np.zeros(W, dtype=np.uint8)
    prev1 = np.zeros(W, dtype=np.uint8)
    cur = np.zeros(W, dtype=np.uint8)

    for y in range(H):
        s2_0 = s2_1 = s2_2 = 0
        s1_0 = s1_1 = s1_2 = 0
        s0_0 = s0_1 = s0_2 = 0

        for x in range(W):
            p = int(y_frame[y, x])
            cur[x] = p

            s2_0, s2_1, s2_2 = s2_1, s2_2, int(prev2[x])
            s1_0, s1_1, s1_2 = s1_1, s1_2, int(prev1[x])
            s0_0, s0_1, s0_2 = s0_1, s0_2, p

            if x >= 2 and y >= 2:
                # Window:
                # a b c
                # d e f
                # g h i
                a, b, c = s2_0, s2_1, s2_2
                d, e, f = s1_0, s1_1, s1_2
                g, h, i = s0_0, s0_1, s0_2

                gx = (c + 2 * f + i) - (a + 2 * d + g)
                gy = (g + 2 * h + i) - (a + 2 * b + c)
                mag = abs(gx) + abs(gy)

                x_out = x - 1
                y_out = y - 1

                gx_img[y_out, x_out] = gx
                gy_img[y_out, x_out] = gy
                mag_img[y_out, x_out] = mag
                valid[y_out, x_out] = True

        prev2, prev1, cur = prev1, cur, prev2
        cur.fill(0)

    return SobelOutputs(gx=gx_img, gy=gy_img, mag=mag_img, valid=valid)


def signed_to_u8(img_signed: np.ndarray, valid: np.ndarray) -> np.ndarray:
    """
    Visualize signed Sobel output:
      128 = zero
      >128 = positive
      <128 = negative
    """
    out = np.full(img_signed.shape, 0, dtype=np.uint8)

    if not np.any(valid):
        return out

    valid_vals = img_signed[valid].astype(np.float32)
    max_abs = np.max(np.abs(valid_vals))
    if max_abs == 0:
        out[valid] = 128
        return out

    scaled = 128.0 + 127.0 * (img_signed.astype(np.float32) / max_abs)
    scaled = np.clip(scaled, 0, 255).astype(np.uint8)
    out[valid] = scaled[valid]
    return out


def unsigned_to_u8(img_unsigned: np.ndarray, valid: np.ndarray) -> np.ndarray:
    """
    Normalize nonnegative Sobel-derived image to 0..255 for display.
    """
    out = np.zeros(img_unsigned.shape, dtype=np.uint8)

    if not np.any(valid):
        return out

    valid_vals = img_unsigned[valid].astype(np.float32)
    vmax = np.max(valid_vals)
    if vmax == 0:
        return out

    scaled = 255.0 * (img_unsigned.astype(np.float32) / vmax)
    scaled = np.clip(scaled, 0, 255).astype(np.uint8)
    out[valid] = scaled[valid]
    return out


def abs_signed_to_u8(img_signed: np.ndarray, valid: np.ndarray) -> np.ndarray:
    return unsigned_to_u8(np.abs(img_signed).astype(np.uint16), valid)


def build_output_image(sobel: SobelOutputs, mode: str) -> np.ndarray:
    mode = mode.lower()

    if mode == "gx":
        return signed_to_u8(sobel.gx, sobel.valid)
    if mode == "gy":
        return signed_to_u8(sobel.gy, sobel.valid)
    if mode == "abs_gx":
        return abs_signed_to_u8(sobel.gx, sobel.valid)
    if mode == "abs_gy":
        return abs_signed_to_u8(sobel.gy, sobel.valid)
    if mode == "mag":
        return unsigned_to_u8(sobel.mag, sobel.valid)

    raise ValueError(f"Unsupported mode: {mode}")


def process_single(
    input_path: Path,
    output_path: Path,
    mode: str,
    dump_npy: bool,
) -> None:
    gray = load_grayscale_image(input_path)
    sobel = sobel_streaming_fpga_like(gray)
    out_img = build_output_image(sobel, mode)

    ok = cv2.imwrite(str(output_path), out_img)
    if not ok:
        raise RuntimeError(f"Failed to write output image: {output_path}")

    if dump_npy:
        stem = output_path.with_suffix("")
        np.save(str(stem) + "_gx.npy", sobel.gx)
        np.save(str(stem) + "_gy.npy", sobel.gy)
        np.save(str(stem) + "_mag.npy", sobel.mag)
        np.save(str(stem) + "_valid.npy", sobel.valid)

    print(f"  {input_path.name} -> {output_path.name}  [{gray.shape[1]}x{gray.shape[0]}]")


def collect_images(directory: Path) -> list[Path]:
    return sorted(
        p for p in directory.iterdir()
        if p.is_file() and p.suffix.lower() in IMAGE_EXTENSIONS
    )


def parse_args():
    p = argparse.ArgumentParser(description="FPGA-like streaming Sobel image exporter")
    p.add_argument(
        "input",
        type=Path,
        help="Input image file, or a directory of images to process in batch",
    )
    p.add_argument(
        "output",
        type=Path,
        nargs="?",
        help=(
            "Output path. For a single file: output image path (e.g. out.png). "
            "For a directory: output directory (default: <input_dir>/sobel_out/)"
        ),
    )
    p.add_argument(
        "--mode",
        choices=["gx", "gy", "abs_gx", "abs_gy", "mag"],
        default="mag",
        help="Which Sobel result to save (default: mag)",
    )
    p.add_argument(
        "--dump-npy",
        action="store_true",
        help="Also save raw gx/gy/mag arrays as .npy next to each output image",
    )
    return p.parse_args()


def main():
    args = parse_args()

    # ── Single-file mode ──────────────────────────────────────────────────────
    if args.input.is_file():
        output_path = args.output
        if output_path is None:
            output_path = args.input.with_name(args.input.stem + "_sobel" + args.input.suffix)

        output_path.parent.mkdir(parents=True, exist_ok=True)
        print(f"Input : {args.input}")
        print(f"Output: {output_path}")
        print(f"Mode  : {args.mode}")
        process_single(args.input, output_path, args.mode, args.dump_npy)
        return

    # ── Directory (batch) mode ────────────────────────────────────────────────
    if args.input.is_dir():
        out_dir = args.output if args.output is not None else args.input / "sobel_out"
        out_dir.mkdir(parents=True, exist_ok=True)

        images = collect_images(args.input)
        if not images:
            print(f"No image files found in {args.input}")
            return

        print(f"Input dir : {args.input}")
        print(f"Output dir: {out_dir}")
        print(f"Mode      : {args.mode}")
        print(f"Images    : {len(images)}\n")

        succeeded, failed = 0, 0
        for img_path in images:
            out_path = out_dir / img_path.name
            try:
                process_single(img_path, out_path, args.mode, args.dump_npy)
                succeeded += 1
            except Exception as exc:
                print(f"  WARNING: skipping {img_path.name} — {exc}")
                failed += 1

        print(f"\nDone. {succeeded} succeeded, {failed} failed.")
        return

    raise FileNotFoundError(f"Input path not found or not a file/directory: {args.input}")


if __name__ == "__main__":
    main()