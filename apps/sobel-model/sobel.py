import argparse
import sys
from dataclasses import dataclass
from pathlib import Path

import cv2
import numpy as np

RESOLUTIONS = {
    "VGA": (640, 480),
    "QVGA": (320, 240),
    "QQVGA": (160, 120),
}

FORMATS = [
    "AUTO",
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


def infer_format(path: Path) -> str:
    suf = path.suffix.lower()
    match suf:
        case ".png":
            return "PNG"
        case ".pgm":
            return "PGM"
        case ".ppm":
            return "PPM"
        case ".yuv422":
            return "YUV422"
        case _:
            return "AUTO"


def parse():
    p = argparse.ArgumentParser(
        "Sobel", description="FPGA-style streaming Sobel for blurgen images."
    )
    p.add_argument(
        "resolution",
        type=resolution_type,
        help="Resolution. WIDTH,HEIGHT or one of: " + " ".join(RESOLUTIONS),
    )
    p.add_argument("input", type=Path, help="Input image file (PNG/PGM/PPM/YUV422).")

    p.add_argument(
        "-f",
        "--fmt",
        type=str.upper,
        choices=FORMATS,
        default="AUTO",
        help="Input format. Default AUTO (infer from extension).",
    )

    p.add_argument(
        "--dump",
        type=str.upper,
        nargs="+",
        choices=["Y", "GX", "GY", "MAG", "VALID"],
        default=["MAG"],
        help="Planes to print line-by-line (default MAG).",
    )

    p.add_argument(
        "--radix",
        type=str.lower,
        choices=["hex", "dec"],
        default="hex",
        help="Print radix for GX/GY/MAG (default hex).",
    )

    p.add_argument("--gx-bits", type=int, default=12, help="GX bit width for hex print.")
    p.add_argument("--gy-bits", type=int, default=12, help="GY bit width for hex print.")
    p.add_argument(
        "--mag-bits", type=int, default=12, help="MAG bit width for hex print."
    )

    p.add_argument(
        "--threshold",
        type=int,
        default=0,
        help="Optional threshold on MAG for score accumulation (default 0).",
    )

    p.add_argument(
        "--row-start",
        type=int,
        default=0,
        help="First row to print (default 0).",
    )
    p.add_argument(
        "--row-count",
        type=int,
        default=0,
        help="Number of rows to print (0 means all from row-start).",
    )
    p.add_argument(
        "--col-start",
        type=int,
        default=0,
        help="First column to print (default 0).",
    )
    p.add_argument(
        "--col-count",
        type=int,
        default=0,
        help="Number of columns to print (0 means all from col-start).",
    )

    p.add_argument(
        "--per-pixel",
        action="store_true",
        help="Print per-pixel window taps + GX/GY/MAG in streaming order (best for small images).",
    )

    args = p.parse_args(sys.argv[1:] or ["-h"])
    args.width, args.height = args.resolution

    if args.fmt == "AUTO":
        inferred = infer_format(args.input)
        if inferred == "AUTO":
            raise SystemExit(
                "Could not infer format from extension. Use --fmt explicitly."
            )
        args.fmt = inferred

    return args


def load_y_image(path: Path, fmt: str, width: int, height: int) -> np.ndarray:
    if not path.exists():
        raise FileNotFoundError(path)

    match fmt.upper():
        case "PNG" | "PGM" | "PPM":
            img = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
            if img is None:
                raise ValueError(f"Failed to read image {path}")

            # Convert to grayscale Y (uint8)
            if img.ndim == 2:
                y = img
            else:
                # BGR -> Gray
                y = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

            if y.dtype != np.uint8:
                y = np.clip(y, 0, 255).astype(np.uint8)

            if (y.shape[1], y.shape[0]) != (width, height):
                raise ValueError(
                    f"Resolution mismatch: file is {y.shape[1]}x{y.shape[0]}, "
                    f"but you specified {width}x{height}."
                )
            return y

        case "YUV422":
            if width % 2 != 0:
                raise ValueError("YUV422 width must be even (2 pixels per 4 bytes).")

            raw = path.read_bytes()
            expected = width * height * 2  # 2 bytes per pixel in YUV422 packed
            if len(raw) != expected:
                raise ValueError(
                    f"YUV422 size mismatch: expected {expected} bytes, got {len(raw)}."
                )

            data = np.frombuffer(raw, dtype=np.uint8).reshape(height, width * 2)
            y = data[:, 0::2]

            return y.copy()

        case _:
            raise ValueError(f"Invalid format {fmt}")


@dataclass(frozen=True)
class SobelOutputs:
    gx: np.ndarray        # int16, same shape as input, valid only on inner pixels
    gy: np.ndarray        # int16
    mag: np.ndarray       # uint16, |gx|+|gy|
    valid: np.ndarray     # bool mask for pixels where gx/gy/mag are valid (inner pixels)
    score: int            # sum(max(0, mag-threshold)) over valid pixels


def sobel_streaming_fpga_like(y_frame: np.ndarray, threshold: int = 0) -> SobelOutputs:
    """
    Sobel using a streaming FPGA-like implementation:
      - 2 line buffers + 3-tap shift regs per row
      - output pixel corresponds to window center at (x-1, y-1)
      - valid only for inner pixels: x in [1..W-2], y in [1..H-2]
      - mag = |gx| + |gy|
    """
    if y_frame.ndim != 2:
        raise ValueError("Input must be grayscale 2D array")

    H, W = y_frame.shape
    gx_img = np.zeros((H, W), dtype=np.int16)
    gy_img = np.zeros((H, W), dtype=np.int16)
    mag_img = np.zeros((H, W), dtype=np.uint16)
    valid = np.zeros((H, W), dtype=bool)

    score = 0

    prev2 = np.zeros(W, dtype=np.uint8)  # row y-2
    prev1 = np.zeros(W, dtype=np.uint8)  # row y-1
    cur = np.zeros(W, dtype=np.uint8)    # row y

    for y in range(H):
        # Shift registers for each of the 3 rows in the 3x3 window
        s2_0 = s2_1 = s2_2 = 0
        s1_0 = s1_1 = s1_2 = 0
        s0_0 = s0_1 = s0_2 = 0

        for x in range(W):
            p = int(y_frame[y, x])
            cur[x] = p

            # Update window columns
            s2_0, s2_1, s2_2 = s2_1, s2_2, int(prev2[x])
            s1_0, s1_1, s1_2 = s1_1, s1_2, int(prev1[x])
            s0_0, s0_1, s0_2 = s0_1, s0_2, p

            # Need at least 3 cols + 3 rows before producing first output
            if x >= 2 and y >= 2:
                # Window taps:
                # a b c
                # d e f
                # g h i
                a, b, c = s2_0, s2_1, s2_2
                d, e, f = s1_0, s1_1, s1_2
                g, h, i = s0_0, s0_1, s0_2

                gx = (c + 2 * f + i) - (a + 2 * d + g)
                gy = (g + 2 * h + i) - (a + 2 * b + c)
                mag = abs(gx) + abs(gy)

                # Output aligns to window center
                x_out = x - 1
                y_out = y - 1

                valid[y_out, x_out] = True
                gx_img[y_out, x_out] = gx
                gy_img[y_out, x_out] = gy
                mag_img[y_out, x_out] = mag

                if mag > threshold:
                    score += (mag - threshold)

        # Rotate line buffers (reuse arrays to avoid allocs)
        prev2, prev1, cur = prev1, cur, prev2
        cur.fill(0)

    return SobelOutputs(gx=gx_img, gy=gy_img, mag=mag_img, valid=valid, score=int(score))


def hex_digits(bits: int) -> int:
    return (bits + 3) // 4


def fmt_signed(v: int, bits: int, radix: str) -> str:
    if radix == "dec":
        return str(int(v))
    mask = (1 << bits) - 1
    return f"{int(v) & mask:0{hex_digits(bits)}X}"


def fmt_unsigned(v: int, bits: int, radix: str) -> str:
    if radix == "dec":
        return str(int(v))
    mask = (1 << bits) - 1
    return f"{int(v) & mask:0{hex_digits(bits)}X}"


def clamp_window(start: int, count: int, limit: int) -> tuple[int, int]:
    if start < 0:
        start = 0
    if start > limit:
        start = limit

    if count <= 0:
        end = limit
    else:
        end = min(limit, start + count)

    return start, end


def print_plane_u8(name: str, img: np.ndarray, r0: int, r1: int, c0: int, c1: int):
    for y in range(r0, r1):
        line = " ".join(f"{int(img[y, x]) & 0xFF:02X}" for x in range(c0, c1))
        print(f"{name}[{y:03d}]: {line}")


def print_plane_signed(
    name: str,
    img: np.ndarray,
    valid: np.ndarray,
    bits: int,
    radix: str,
    r0: int,
    r1: int,
    c0: int,
    c1: int,
):
    placeholder = "-" * max(3, hex_digits(bits))
    for y in range(r0, r1):
        parts = []
        for x in range(c0, c1):
            if valid[y, x]:
                parts.append(fmt_signed(int(img[y, x]), bits, radix))
            else:
                parts.append(placeholder)
        print(f"{name}[{y:03d}]: " + " ".join(parts))


def print_plane_unsigned(
    name: str,
    img: np.ndarray,
    valid: np.ndarray,
    bits: int,
    radix: str,
    r0: int,
    r1: int,
    c0: int,
    c1: int,
):
    placeholder = "-" * max(3, hex_digits(bits))
    for y in range(r0, r1):
        parts = []
        for x in range(c0, c1):
            if valid[y, x]:
                parts.append(fmt_unsigned(int(img[y, x]), bits, radix))
            else:
                parts.append(placeholder)
        print(f"{name}[{y:03d}]: " + " ".join(parts))


def per_pixel_dump(y_frame: np.ndarray, out: SobelOutputs, r0: int, r1: int, c0: int, c1: int, args):
    """
    Per-pixel dump in raster order for valid outputs, including window taps.
    Best used on very small images (e.g., 8x8) or with small print windows.
    """
    H, W = y_frame.shape

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
                x_out = x - 1
                y_out = y - 1

                if not (r0 <= y_out < r1 and c0 <= x_out < c1):
                    continue

                a, b, c = s2_0, s2_1, s2_2
                d, e, f = s1_0, s1_1, s1_2
                g, h, i = s0_0, s0_1, s0_2

                gx = int(out.gx[y_out, x_out])
                gy = int(out.gy[y_out, x_out])
                mag = int(out.mag[y_out, x_out])

                print(
                    f"OUT y={y_out:03d} x={x_out:03d} "
                    f"win=[{a:02X} {b:02X} {c:02X}; {d:02X} {e:02X} {f:02X}; {g:02X} {h:02X} {i:02X}] "
                    f"gx={fmt_signed(gx, args.gx_bits, args.radix)} "
                    f"gy={fmt_signed(gy, args.gy_bits, args.radix)} "
                    f"mag={fmt_unsigned(mag, args.mag_bits, args.radix)}"
                )

        prev2, prev1, cur = prev1, cur, prev2
        cur.fill(0)


def main():
    args = parse()

    y = load_y_image(
        args.input, args.fmt, args.width, args.height
    )
    out = sobel_streaming_fpga_like(y, threshold=args.threshold)

    H, W = y.shape
    r0, r1 = clamp_window(args.row_start, args.row_count, H)
    c0, c1 = clamp_window(args.col_start, args.col_count, W)

    print(f"Loaded {args.input} as {args.fmt}, {W}x{H}")
    print(f"Sobel valid pixels: x=1..{W-2}, y=1..{H-2}")
    print(f"Score (sum(max(0,mag-threshold)) over valid): {out.score}")
    if args.threshold:
        print(f"Threshold: {args.threshold}")

    if args.per_pixel:
        per_pixel_dump(y, out, r0, r1, c0, c1, args)
        return

    # Line-by-line dumps
    dump_set = set(args.dump)

    if "Y" in dump_set:
        print_plane_u8("Y", y, r0, r1, c0, c1)

    if "GX" in dump_set:
        print_plane_signed(
            "GX", out.gx, out.valid, args.gx_bits, args.radix, r0, r1, c0, c1
        )

    if "GY" in dump_set:
        print_plane_signed(
            "GY", out.gy, out.valid, args.gy_bits, args.radix, r0, r1, c0, c1
        )

    if "MAG" in dump_set:
        print_plane_unsigned(
            "MAG", out.mag, out.valid, args.mag_bits, args.radix, r0, r1, c0, c1
        )

    if "VALID" in dump_set:
        for yy in range(r0, r1):
            line = " ".join("1" if out.valid[yy, xx] else "0" for xx in range(c0, c1))
            print(f"VALID[{yy:03d}]: {line}")


if __name__ == "__main__":
    main()
