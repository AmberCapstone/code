"""
Battleship Boat Detection - Software Verification Suite
========================================================
Compares three detection approaches:
  1. CNN grid classifier (10x10 occupied-cell prediction)
  2. Classical blob detection (threshold + morphology + CCL)
  3. Sobel + threshold pipeline

Label format: JSON with 'grid' and 'objects[].occupied_cells' fields.
Output: per-method precision / recall / F1, plus annotated images.

Usage:
    python battleship_detector.py --data_dir /path/to/dataset \
                                  --output_dir ./results \
                                  [--train] [--epochs 20] [--visualize]

Dependencies:
    pip install opencv-python numpy torch torchvision tqdm matplotlib
"""

import argparse
import json
import os
import glob
import time
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Tuple, Dict, Optional

import cv2
import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from tqdm import tqdm


# ---------------------------------------------------------------------------
# Device selection — CUDA > MPS (Apple Silicon) > CPU
# ---------------------------------------------------------------------------

def get_device() -> torch.device:
    if torch.cuda.is_available():
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

GRID_ROWS = 10
GRID_COLS = 10
IMG_H = 120
IMG_W = 160

YOLO_ANCHORS     = [(11, 22), (11, 44)]   # updated at training time by k-means
YOLO_NUM_ANCHORS = len(YOLO_ANCHORS)
YOLO_NUM_OUTPUTS = 5


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

@dataclass
class GridLabel:
    """Parsed ground-truth: 10x10 binary occupancy grid + raw bboxes."""
    image_path: str
    grid: np.ndarray                        # shape (GRID_ROWS, GRID_COLS) bool
    cell_bboxes: List[Tuple[int,int,int,int]]   # xyxy, one per ship
    grid_top_left: Tuple[int, int]          # (x, y) pixel origin of grid
    cell_size: int


# ---------------------------------------------------------------------------
# Label parsing
# ---------------------------------------------------------------------------

def load_label(json_path: str, image_dir: str) -> Optional[GridLabel]:
    """Parse one JSON label file into a GridLabel."""
    with open(json_path) as f:
        data = json.load(f)

    image_file = data["image"]
    image_path = os.path.join(image_dir, image_file)

    grid_info  = data["grid"]
    cell_size  = grid_info["cell_size"]
    top_left   = tuple(grid_info["grid_top_left"])   # (x, y)

    occupancy  = np.zeros((GRID_ROWS, GRID_COLS), dtype=bool)
    bboxes     = []

    for obj in data.get("objects", []):
        for (row, col) in obj["occupied_cells"]:
            if 0 <= row < GRID_ROWS and 0 <= col < GRID_COLS:
                occupancy[row, col] = True
        x1, y1, x2, y2 = obj["cell_bbox_xyxy"]
        bboxes.append((x1, y1, x2, y2))

    return GridLabel(
        image_path=image_path,
        grid=occupancy,
        cell_bboxes=bboxes,
        grid_top_left=top_left,
        cell_size=cell_size,
    )


def load_split(data_dir: str, split: str) -> List[GridLabel]:
    """
    Load one split (train / val / test) from the canonical layout:

        <data_dir>/
            images/<split>/<name>.png
            labels/<split>/<name>.json
    """
    label_dir = os.path.join(data_dir, "labels", split)
    image_dir = os.path.join(data_dir, "images", split)

    if not os.path.isdir(label_dir):
        raise FileNotFoundError(f"Label directory not found: {label_dir}")
    if not os.path.isdir(image_dir):
        raise FileNotFoundError(f"Image directory not found: {image_dir}")

    json_paths = sorted(glob.glob(os.path.join(label_dir, "*.json")))
    if not json_paths:
        raise FileNotFoundError(f"No JSON files found in {label_dir}")

    labels = []
    missing = 0
    for p in json_paths:
        try:
            label = load_label(p, image_dir)
        except Exception as e:
            print(f"  Warning: could not parse {p}: {e}")
            continue
        if not os.path.exists(label.image_path):
            missing += 1
            continue
        labels.append(label)

    if missing:
        print(f"  Warning: {missing} label(s) in '{split}' had no matching image")
    print(f"  {split:5s}: {len(labels)} samples")
    return labels


def load_dataset(data_dir: str) -> Dict[str, List[GridLabel]]:
    """
    Load all three splits from a dataset structured as:

        <data_dir>/
            images/train/  images/val/  images/test/
            labels/train/  labels/val/  labels/test/

    Returns a dict with keys 'train', 'val', 'test'.
    Missing splits are returned as empty lists with a warning.
    """
    print(f"Loading dataset from {data_dir}")
    splits: Dict[str, List[GridLabel]] = {}
    for split in ("train", "val", "test"):
        try:
            splits[split] = load_split(data_dir, split)
        except FileNotFoundError as e:
            print(f"  Warning: skipping split '{split}' — {e}")
            splits[split] = []

    total = sum(len(v) for v in splits.values())
    print(f"  Total: {total} samples across "
          f"{sum(1 for v in splits.values() if v)} split(s)")
    return splits



# ---------------------------------------------------------------------------
# Helpers: grid <-> pixel conversion
# ---------------------------------------------------------------------------

def cell_to_pixel_bbox(row: int, col: int, top_left: Tuple[int,int],
                        cell_size: int) -> Tuple[int,int,int,int]:
    """Return pixel xyxy bbox for a grid cell."""
    tx, ty = top_left
    x1 = tx + col * cell_size
    y1 = ty + row * cell_size
    x2 = x1 + cell_size
    y2 = y1 + cell_size
    return (x1, y1, x2, y2)


def prediction_grid_to_bboxes(pred_grid: np.ndarray,
                               top_left: Tuple[int,int],
                               cell_size: int,
                               max_ship_cells: int = 5) -> List[Tuple[int,int,int,int]]:
    """Convert a predicted occupancy grid into pixel bboxes.

    Adjacent occupied cells are merged (ships span multiple cells), but any
    connected component larger than max_ship_cells is split back into individual
    cells to prevent separate nearby ships from being merged into one giant box.
    """
    grid_u8 = pred_grid.astype(np.uint8) * 255
    num_labels, labels_im = cv2.connectedComponents(grid_u8, connectivity=4)
    tx, ty = top_left

    bboxes = []
    for lbl in range(1, num_labels):
        ys, xs = np.where(labels_im == lbl)
        n_cells = len(ys)

        if n_cells <= max_ship_cells:
            # Normal case: treat the whole component as one ship
            r_min, r_max = ys.min(), ys.max()
            c_min, c_max = xs.min(), xs.max()
            x1 = tx + c_min * cell_size
            y1 = ty + r_min * cell_size
            x2 = tx + (c_max + 1) * cell_size
            y2 = ty + (r_max + 1) * cell_size
            bboxes.append((x1, y1, x2, y2))
        else:
            # Oversized component: likely two ships touching — emit one bbox
            # per cell so we at least get the positions roughly right
            for r, c in zip(ys, xs):
                x1 = tx + c * cell_size
                y1 = ty + r * cell_size
                bboxes.append((x1, y1, x1 + cell_size, y1 + cell_size))
    return bboxes


# ---------------------------------------------------------------------------
# Evaluation metrics
# ---------------------------------------------------------------------------

def iou(boxA: Tuple, boxB: Tuple) -> float:
    ax1, ay1, ax2, ay2 = boxA
    bx1, by1, bx2, by2 = boxB
    ix1 = max(ax1, bx1); iy1 = max(ay1, by1)
    ix2 = min(ax2, bx2); iy2 = min(ay2, by2)
    iw = max(0, ix2 - ix1); ih = max(0, iy2 - iy1)
    inter = iw * ih
    areaA = (ax2-ax1)*(ay2-ay1)
    areaB = (bx2-bx1)*(by2-by1)
    union = areaA + areaB - inter
    return inter / union if union > 0 else 0.0


def match_detections(pred_boxes: List[Tuple], gt_boxes: List[Tuple],
                     iou_thresh: float = 0.3) -> Tuple[int,int,int]:
    """Greedy match predicted boxes to GT. Returns (TP, FP, FN)."""
    matched_gt = set()
    tp = 0
    for pb in pred_boxes:
        best_iou = 0.0
        best_j   = -1
        for j, gb in enumerate(gt_boxes):
            if j in matched_gt:
                continue
            s = iou(pb, gb)
            if s > best_iou:
                best_iou = s
                best_j   = j
        if best_iou >= iou_thresh and best_j >= 0:
            tp += 1
            matched_gt.add(best_j)
    fp = len(pred_boxes) - tp
    fn = len(gt_boxes)   - tp
    return tp, fp, fn


@dataclass
class Metrics:
    name: str
    tp: int = 0; fp: int = 0; fn: int = 0
    inference_ms: float = 0.0
    n_images: int = 0

    def update(self, tp, fp, fn, elapsed_ms):
        self.tp += tp; self.fp += fp; self.fn += fn
        self.inference_ms += elapsed_ms
        self.n_images += 1

    @property
    def precision(self): return self.tp/(self.tp+self.fp) if (self.tp+self.fp) else 0.0
    @property
    def recall(self):    return self.tp/(self.tp+self.fn) if (self.tp+self.fn) else 0.0
    @property
    def f1(self):
        p, r = self.precision, self.recall
        return 2*p*r/(p+r) if (p+r) else 0.0
    @property
    def avg_ms(self): return self.inference_ms/self.n_images if self.n_images else 0.0

    def __str__(self):
        return (f"{self.name:30s}  P={self.precision:.3f}  R={self.recall:.3f}"
                f"  F1={self.f1:.3f}  avg {self.avg_ms:.1f} ms/img"
                f"  (TP={self.tp} FP={self.fp} FN={self.fn})")


# ---------------------------------------------------------------------------
# Method 1 — CNN
# ---------------------------------------------------------------------------

class BattleshipDataset(Dataset):
    """Loads greyscale QQVGA images + 10x10 occupancy grids."""

    def __init__(self, labels: List[GridLabel], augment: bool = False):
        self.labels  = labels
        self.augment = augment

    def __len__(self): return len(self.labels)

    def __getitem__(self, idx):
        lbl = self.labels[idx]
        img = cv2.imread(lbl.image_path, cv2.IMREAD_GRAYSCALE)
        if img is None:
            img = np.zeros((IMG_H, IMG_W), dtype=np.uint8)
        img = cv2.resize(img, (IMG_W, IMG_H))
        grid = lbl.grid.copy()

        if self.augment:
            # Horizontal flip — mirror grid columns too
            if np.random.rand() < 0.5:
                img  = cv2.flip(img, 1)
                grid = grid[:, ::-1].copy()
            # Vertical flip — mirror grid rows
            if np.random.rand() < 0.5:
                img  = cv2.flip(img, 0)
                grid = grid[::-1, :].copy()
            # Brightness / contrast jitter
            alpha = np.random.uniform(0.8, 1.2)   # contrast
            beta  = np.random.randint(-15, 15)     # brightness
            img   = np.clip(img.astype(np.float32) * alpha + beta, 0, 255).astype(np.uint8)
            # Small amount of Gaussian noise
            noise = np.random.normal(0, 3, img.shape).astype(np.float32)
            img   = np.clip(img.astype(np.float32) + noise, 0, 255).astype(np.uint8)

        x = torch.from_numpy(img).float().unsqueeze(0) / 127.5 - 1.0
        y = torch.from_numpy(grid.astype(np.float32))   # (10, 10)
        return x, y


def _adaptive_pool_compat(pool, f):
    """AdaptiveAvgPool2d on MPS requires divisible spatial dims. CPU fallback."""
    dev = f.device
    return pool(f.cpu()).to(dev)


class FCNSmall(nn.Module):
    """
    FCN-Small: 4-block encoder, max 16 filters. FPGA-feasible baseline.
    Same architecture used in previous experiments — now re-evaluated with
    100k training images to isolate data volume as the variable.

    Estimated iCE40UP5K footprint: ~1,200 LUTs, ~6 EBRAMs.
    """
    def __init__(self):
        super().__init__()
        self.encoder = nn.Sequential(
            nn.Conv2d(1,  8,  3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->80x60
            nn.Conv2d(8,  16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->40x30
            nn.Conv2d(16, 16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->20x15
            nn.Conv2d(16, 16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->10x7
        )
        self.pool = nn.AdaptiveAvgPool2d((GRID_ROWS, GRID_COLS))
        self.head = nn.Conv2d(16, 1, 1)

    def forward(self, x):
        f = self.encoder(x)
        f = _adaptive_pool_compat(self.pool, f)
        return self.head(f).squeeze(1)


class FCNDeep(nn.Module):
    """
    FCN-Deep: 4-block encoder with 32 filters + skip connection.
    The skip connection concatenates early features (after block 2) with
    the deep features before the output head, giving the model both
    fine-grained edge detail and high-level semantic context simultaneously.

    This is the key architectural improvement: with the previous FCN-Small
    the output only saw the deeply-pooled 10x7 feature map, discarding the
    spatial detail captured in earlier blocks.

    Estimated iCE40UP5K footprint: ~2,800 LUTs, ~14 EBRAMs — still fits.
    """
    def __init__(self):
        super().__init__()
        # Encoder split into two halves to expose the skip connection point
        self.enc1 = nn.Sequential(
            nn.Conv2d(1,  16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->80x60
            nn.Conv2d(16, 32, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->40x30
        )
        self.enc2 = nn.Sequential(
            nn.Conv2d(32, 32, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->20x15
            nn.Conv2d(32, 32, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),  # ->10x7
        )
        self.pool = nn.AdaptiveAvgPool2d((GRID_ROWS, GRID_COLS))
        # Skip projection: enc1 output (32ch @40x30) -> 10x10, projected to 16ch
        self.skip_pool = nn.AdaptiveAvgPool2d((GRID_ROWS, GRID_COLS))
        self.skip_proj = nn.Conv2d(32, 16, 1)
        # Head receives 32 (deep) + 16 (skip) = 48 channels
        self.head = nn.Sequential(
            nn.Conv2d(48, 16, 1), nn.ReLU(),
            nn.Conv2d(16,  1, 1),
        )

    def forward(self, x):
        s    = self.enc1(x)                                   # (B, 32, 40, 30)
        f    = self.enc2(s)                                   # (B, 32, 10,  7)
        f    = _adaptive_pool_compat(self.pool, f)            # (B, 32, 10, 10)
        skip = _adaptive_pool_compat(self.skip_pool, s)       # (B, 32, 10, 10)
        skip = self.skip_proj(skip)                           # (B, 16, 10, 10)
        out  = self.head(torch.cat([f, skip], dim=1))         # (B,  1, 10, 10)
        return out.squeeze(1)                                 # (B, 10, 10)


# Keep TinyGridCNN as an alias so saved weights from earlier runs still load
TinyGridCNN = FCNSmall


def compute_pos_weight(labels: List[GridLabel]) -> float:
    """
    Compute the negative/positive cell ratio from the training labels.
    Used to set BCEWithLogitsLoss pos_weight accurately rather than guessing.
    A random 2,000-sample subset is used for speed on large datasets.
    """
    sample = labels[:2000] if len(labels) > 2000 else labels
    total  = len(sample) * GRID_ROWS * GRID_COLS
    pos    = sum(lbl.grid.sum() for lbl in sample)
    neg    = total - pos
    ratio  = neg / max(pos, 1)
    print(f"  pos_weight computed from {len(sample)} samples: "
          f"{pos}/{total} occupied cells  →  pos_weight={ratio:.1f}")
    return float(ratio)


def train_one_cnn(model: nn.Module,
                  train_labels: List[GridLabel],
                  val_labels: List[GridLabel],
                  epochs: int,
                  model_path: str,
                  pos_weight: float) -> float:
    """
    Train a single model variant. Returns best validation loss.
    Shared by both FCN-Small and FCN-Deep to keep training identical.
    """
    # With 100k images augmentation is unnecessary — real variety covers it.
    train_ds = BattleshipDataset(train_labels, augment=False)
    val_ds   = BattleshipDataset(val_labels,   augment=False)

    # Larger batch size: better gradient estimates, faster epochs on MPS.
    train_loader = DataLoader(train_ds, batch_size=64, shuffle=True,  num_workers=0)
    val_loader   = DataLoader(val_ds,   batch_size=64, shuffle=False, num_workers=0)

    device = get_device()
    model  = model.to(device)
    n_params = sum(p.numel() for p in model.parameters() if p.requires_grad)
    print(f"  Parameters: {n_params:,}")

    # LR warmup for 2 epochs then cosine annealing — better than fixed StepLR
    # for large datasets where the loss landscape is well-defined.
    optimizer  = optim.Adam(model.parameters(), lr=1e-4, weight_decay=1e-4)
    warmup     = optim.lr_scheduler.LinearLR(
                     optimizer, start_factor=0.1, end_factor=1.0, total_iters=2)
    cosine     = optim.lr_scheduler.CosineAnnealingLR(
                     optimizer, T_max=max(epochs - 2, 1), eta_min=1e-5)
    scheduler  = optim.lr_scheduler.SequentialLR(
                     optimizer, schedulers=[warmup, cosine], milestones=[2])

    pw        = torch.tensor([pos_weight]).to(device)
    criterion = nn.BCEWithLogitsLoss(pos_weight=pw)

    best_val_loss     = float("inf")
    patience_counter  = 0
    EARLY_STOP        = 8

    for epoch in range(1, epochs + 1):
        model.train()
        train_loss = 0.0
        for x, y in tqdm(train_loader, desc=f"    Epoch {epoch}/{epochs}", leave=False):
            x, y = x.to(device), y.to(device)
            optimizer.zero_grad()
            loss = criterion(model(x), y)
            loss.backward()
            optimizer.step()
            train_loss += loss.item() * x.size(0)
        train_loss /= len(train_ds)

        model.eval()
        val_loss = 0.0
        with torch.no_grad():
            for x, y in val_loader:
                x, y = x.to(device), y.to(device)
                val_loss += criterion(model(x), y).item() * x.size(0)
        val_loss /= len(val_ds)
        scheduler.step()

        lr = optimizer.param_groups[0]["lr"]
        print(f"    Epoch {epoch:3d}  train={train_loss:.4f}  val={val_loss:.4f}"
              f"  lr={lr:.2e}")

        if val_loss < best_val_loss:
            best_val_loss = val_loss
            patience_counter = 0
            torch.save(model.state_dict(), model_path)
        else:
            patience_counter += 1
            if patience_counter >= EARLY_STOP:
                print(f"    Early stop at epoch {epoch}")
                break

    print(f"  Best val loss: {best_val_loss:.4f}  →  {model_path}")
    ckpt = torch.load(model_path, map_location=device, weights_only=False)
    if isinstance(ckpt, dict) and "state_dict" in ckpt:
        model.load_state_dict(ckpt["state_dict"])
        YOLO_ANCHORS[:] = ckpt.get("anchors", YOLO_ANCHORS)
    else:
        model.load_state_dict(ckpt)
    return best_val_loss


# ---------------------------------------------------------------------------
# YOLO anchor k-means + dataset, model, loss, inference
# ---------------------------------------------------------------------------

def compute_yolo_anchors(labels: List[GridLabel],
                          n_anchors: int = 2,
                          n_iter: int = 100) -> List[Tuple[int, int]]:
    """
    K-means clustering on GT bbox dimensions to find optimal anchor sizes.
    Uses IoU distance (1 - IoU) rather than Euclidean distance, which
    clusters by shape rather than absolute size — standard YOLO practice.
    """
    # Collect all (w, h) pairs from cell bboxes
    dims = []
    for lbl in labels:
        for (x1, y1, x2, y2) in lbl.cell_bboxes:
            w = float(x2 - x1); h = float(y2 - y1)
            if w > 0 and h > 0:
                dims.append([w, h])
    if len(dims) < n_anchors:
        return YOLO_ANCHORS

    dims = np.array(dims, dtype=np.float32)

    # IoU between zero-centred boxes of size (w,h) and each centroid
    def iou_dist(boxes, centroids):
        # boxes: (N,2), centroids: (K,2)
        inter_w = np.minimum(boxes[:,0:1], centroids[:,0])   # (N,K)
        inter_h = np.minimum(boxes[:,1:2], centroids[:,1])
        inter   = inter_w * inter_h
        union   = (boxes[:,0:1] * boxes[:,1:2] +
                   centroids[:,0] * centroids[:,1] - inter)
        return 1.0 - inter / np.maximum(union, 1e-6)   # (N,K) distance

    # Initialise centroids by picking n_anchors samples spread across size range
    idx = np.linspace(0, len(dims)-1, n_anchors, dtype=int)
    centroids = dims[idx].copy()

    for _ in range(n_iter):
        dist     = iou_dist(dims, centroids)           # (N, K)
        assign   = dist.argmin(axis=1)                 # (N,)
        new_cent = np.array([
            dims[assign == k].mean(axis=0) if (assign == k).any() else centroids[k]
            for k in range(n_anchors)
        ])
        if np.allclose(new_cent, centroids, atol=0.5):
            break
        centroids = new_cent

    # Sort anchors by area ascending (small first)
    centroids = centroids[np.argsort(centroids[:,0] * centroids[:,1])]

    # Enforce minimum aspect ratio: ships are elongated (min 1.5:1).
    # If k-means converged to near-square centroids (common when cell_bbox
    # padding dominates), force the longer dimension to be at least 1.5x
    # the shorter one, preserving the area.
    anchors = []
    for w, h in centroids:
        area = w * h
        long_side  = max(w, h)
        short_side = min(w, h)
        if long_side / max(short_side, 1) < 1.5:
            # Rescale: keep area, set ratio to 1.5:1
            short_side = float(np.sqrt(area / 1.5))
            long_side  = 1.5 * short_side
            w, h = (short_side, long_side) if h >= w else (long_side, short_side)
        anchors.append((max(1, int(round(w))), max(1, int(round(h)))))

    print(f"  K-means anchors ({n_anchors}): {anchors}")
    return anchors


class YOLODataset(Dataset):
    """
    Produces YOLO-format targets: (GRID_ROWS, GRID_COLS, NUM_ANCHORS, 5)
    where the last dim is [tx, ty, tw, th, objectness].

    tx, ty: centre offset within the cell, normalised to [0,1].
    tw, th: log-space ratio of bbox size to anchor size.
    obj:    1.0 if this anchor is responsible for a ship, else 0.0.
    """

    def __init__(self, labels: List[GridLabel]):
        self.labels = labels

    def __len__(self): return len(self.labels)

    def __getitem__(self, idx):
        lbl = self.labels[idx]
        img = cv2.imread(lbl.image_path, cv2.IMREAD_GRAYSCALE)
        if img is None:
            img = np.zeros((IMG_H, IMG_W), dtype=np.uint8)
        img = cv2.resize(img, (IMG_W, IMG_H))
        x = torch.from_numpy(img).float().unsqueeze(0) / 127.5 - 1.0

        target  = torch.zeros(GRID_ROWS, GRID_COLS, YOLO_NUM_ANCHORS, YOLO_NUM_OUTPUTS)
        cell_w  = IMG_W / GRID_COLS
        cell_h  = IMG_H / GRID_ROWS

        for (x1, y1, x2, y2) in lbl.cell_bboxes:
            bw = float(x2 - x1); bh = float(y2 - y1)
            cx = (x1 + x2) / 2.0; cy = (y1 + y2) / 2.0
            col = min(int(cx / cell_w), GRID_COLS - 1)
            row = min(int(cy / cell_h), GRID_ROWS - 1)
            tx  = (cx / cell_w) - col
            ty  = (cy / cell_h) - row

            best_iou = -1.0; best_a = 0
            for a, (aw, ah) in enumerate(YOLO_ANCHORS):
                inter = min(bw, aw) * min(bh, ah)
                union = bw * bh + aw * ah - inter
                a_iou = inter / union if union > 0 else 0.0
                if a_iou > best_iou:
                    best_iou = a_iou; best_a = a

            aw, ah = YOLO_ANCHORS[best_a]
            tw = float(np.log(max(bw, 1e-4) / aw))
            th = float(np.log(max(bh, 1e-4) / ah))
            target[row, col, best_a] = torch.tensor([tx, ty, tw, th, 1.0])

        return x, target


class TinyYOLO(nn.Module):
    """
    FPGA-constrained YOLO detector for QQVGA presence/absence detection.

    Shares the FCN-Small encoder (4 blocks, max 16 filters, ~23 KB INT8).
    Head outputs NUM_ANCHORS*5 channels via 1x1 conv, reshaped to
    (B, GRID_ROWS, GRID_COLS, NUM_ANCHORS, NUM_OUTPUTS).

    Total weight budget: ~24 KB — fits in iCE40UP5K BRAM (157 KB available).
    """
    def __init__(self):
        super().__init__()
        self.encoder = nn.Sequential(
            nn.Conv2d(1,  8,  3, padding=1), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(8,  16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(16, 16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(16, 16, 3, padding=1), nn.ReLU(), nn.MaxPool2d(2),
        )
        self.pool = nn.AdaptiveAvgPool2d((GRID_ROWS, GRID_COLS))
        self.head = nn.Conv2d(16, YOLO_NUM_ANCHORS * YOLO_NUM_OUTPUTS, 1)

    def forward(self, x):
        f   = self.encoder(x)
        f   = _adaptive_pool_compat(self.pool, f)        # (B, 16, 10, 10)
        out = self.head(f)                               # (B, A*5, 10, 10)
        B   = out.shape[0]
        out = out.permute(0, 2, 3, 1)                   # (B, 10, 10, A*5)
        return out.reshape(B, GRID_ROWS, GRID_COLS,
                           YOLO_NUM_ANCHORS, YOLO_NUM_OUTPUTS)


def yolo_loss(pred: torch.Tensor, target: torch.Tensor,
              lambda_coord: float = 5.0,
              lambda_noobj: float = 0.3,
              focal_gamma:  float = 2.0) -> torch.Tensor:
    """
    YOLO-style loss with focal modulation on objectness.

    Focal weighting (Lin et al. 2017) down-weights the easy negatives that
    dominate the gradient when ~95% of cells are empty. Without it the model
    learns to suppress everything to minimise the noobj term.

    lambda_noobj reduced 0.5→0.3 to further reduce empty-cell pressure.
    """
    obj_mask   = target[..., 4]
    noobj_mask = 1.0 - obj_mask

    # Coordinate regression — only on positive cells
    coord_loss = (obj_mask.unsqueeze(-1) *
                  (pred[..., :4] - target[..., :4]) ** 2).sum()

    # Focal-weighted objectness BCE
    obj_logits = pred[..., 4]
    bce        = nn.functional.binary_cross_entropy_with_logits(
                     obj_logits, obj_mask, reduction="none")
    prob       = torch.sigmoid(obj_logits).detach()
    # For positives: focal = (1-p)^gamma; for negatives: focal = p^gamma
    focal      = torch.where(obj_mask > 0.5,
                              (1.0 - prob) ** focal_gamma,
                              prob          ** focal_gamma)
    obj_loss   = (obj_mask   * focal * bce).sum()
    noobj_loss = (noobj_mask * focal * bce).sum()

    return (lambda_coord * coord_loss + obj_loss + lambda_noobj * noobj_loss) / pred.shape[0]


def infer_yolo(model: TinyYOLO, img_gray: np.ndarray,
               obj_thresh: float = 0.35) -> List[Tuple[int,int,int,int]]:
    """Decode TinyYOLO output to xyxy pixel bboxes."""
    device = next(model.parameters()).device
    img = cv2.resize(img_gray, (IMG_W, IMG_H))
    x   = torch.from_numpy(img).float().unsqueeze(0).unsqueeze(0) / 127.5 - 1.0
    x   = x.to(device)

    cell_w = IMG_W / GRID_COLS
    cell_h = IMG_H / GRID_ROWS

    with torch.no_grad():
        out = model(x).squeeze(0).cpu().numpy()   # (R, C, A, 5)

    boxes = []
    for row in range(GRID_ROWS):
        for col in range(GRID_COLS):
            for a_idx, (aw, ah) in enumerate(YOLO_ANCHORS):
                tx, ty, tw, th, obj_logit = out[row, col, a_idx]
                obj_conf = float(torch.sigmoid(torch.tensor(obj_logit)))
                if obj_conf < obj_thresh:
                    continue
                cx = (col + float(torch.sigmoid(torch.tensor(tx)))) * cell_w
                cy = (row + float(torch.sigmoid(torch.tensor(ty)))) * cell_h
                bw = aw * float(np.exp(np.clip(tw, -4, 4)))
                bh = ah * float(np.exp(np.clip(th, -4, 4)))
                x1 = max(0,      int(cx - bw / 2))
                y1 = max(0,      int(cy - bh / 2))
                x2 = min(IMG_W,  int(cx + bw / 2))
                y2 = min(IMG_H,  int(cy + bh / 2))
                if x2 > x1 and y2 > y1:
                    boxes.append((x1, y1, x2, y2))

    return nms_bboxes(boxes, iou_thresh=0.4)


def train_yolo(model: TinyYOLO,
               train_labels: List[GridLabel],
               val_labels: List[GridLabel],
               epochs: int,
               model_path: str) -> None:
    """Train TinyYOLO with YOLO loss. Saves best checkpoint to model_path."""
    # Recompute anchors from training data and update the global
    global YOLO_ANCHORS
    YOLO_ANCHORS = compute_yolo_anchors(train_labels, n_anchors=YOLO_NUM_ANCHORS)

    train_ds = YOLODataset(train_labels)
    val_ds   = YOLODataset(val_labels)
    train_loader = DataLoader(train_ds, batch_size=64, shuffle=True,  num_workers=0)
    val_loader   = DataLoader(val_ds,   batch_size=64, shuffle=False, num_workers=0)

    device    = next(model.parameters()).device
    n_params  = sum(p.numel() for p in model.parameters() if p.requires_grad)
    print(f"  Parameters: {n_params:,}")

    optimizer = optim.Adam(model.parameters(), lr=1e-3, weight_decay=1e-4)
    warmup    = optim.lr_scheduler.LinearLR(
                    optimizer, start_factor=0.1, end_factor=1.0, total_iters=2)
    cosine    = optim.lr_scheduler.CosineAnnealingLR(
                    optimizer, T_max=max(epochs - 2, 1), eta_min=1e-5)
    scheduler = optim.lr_scheduler.SequentialLR(
                    optimizer, schedulers=[warmup, cosine], milestones=[2])

    best_val  = float("inf")
    patience  = 0
    PATIENCE  = 8

    for epoch in range(1, epochs + 1):
        model.train()
        train_loss = 0.0
        for x, y in tqdm(train_loader, desc=f"    Epoch {epoch}/{epochs}", leave=False):
            x, y = x.to(device), y.to(device)
            optimizer.zero_grad()
            loss = yolo_loss(model(x), y)
            loss.backward()
            optimizer.step()
            train_loss += loss.item() * x.size(0)
        train_loss /= len(train_ds)

        model.eval()
        val_loss = 0.0
        with torch.no_grad():
            for x, y in val_loader:
                x, y = x.to(device), y.to(device)
                val_loss += yolo_loss(model(x), y).item() * x.size(0)
        val_loss /= len(val_ds)
        scheduler.step()

        lr = optimizer.param_groups[0]["lr"]
        print(f"    Epoch {epoch:3d}  train={train_loss:.4f}  val={val_loss:.4f}"
              f"  lr={lr:.2e}")

        if val_loss < best_val:
            best_val = val_loss
            patience = 0
            torch.save({
                "state_dict": model.state_dict(),
                "anchors":    YOLO_ANCHORS,
            }, model_path)
        else:
            patience += 1
            if patience >= PATIENCE:
                print(f"    Early stop at epoch {epoch}")
                break

    print(f"  Best val loss: {best_val:.4f}  ->  {model_path}")
    ckpt = torch.load(model_path, map_location=device, weights_only=False)
    if isinstance(ckpt, dict) and "state_dict" in ckpt:
        model.load_state_dict(ckpt["state_dict"])
        YOLO_ANCHORS[:] = ckpt.get("anchors", YOLO_ANCHORS)
    else:
        model.load_state_dict(ckpt)


def train_cnn(train_labels: List[GridLabel], val_labels: List[GridLabel],
              epochs: int = 40,
              model_path: str = "cnn_model.pt") -> Dict[str, nn.Module]:
    """
    Train both FCN-Small and FCN-Deep and return both models in a dict.
    Weights are saved as <model_path> (Small) and <stem>_deep.<ext> (Deep).
    """
    device = get_device()
    print(f"Training on {device}  "
          f"({len(train_labels):,} train / {len(val_labels):,} val)")

    pw = compute_pos_weight(train_labels)

    stem, ext = os.path.splitext(model_path)
    path_small = model_path
    path_deep  = f"{stem}_deep{ext}"

    print("\n── FCN-Small ──────────────────────────────────────────────────")
    small = FCNSmall().to(device)
    train_one_cnn(small, train_labels, val_labels, epochs, path_small, pw)
    small.eval()

    print("\n── FCN-Deep ───────────────────────────────────────────────────")
    deep = FCNDeep().to(device)
    train_one_cnn(deep, train_labels, val_labels, epochs, path_deep, pw)
    deep.eval()

    print("\n── Tiny YOLO ──────────────────────────────────────────────────")
    path_yolo = f"{stem}_yolo{ext}"
    yolo = TinyYOLO().to(device)
    train_yolo(yolo, train_labels, val_labels, epochs, path_yolo)
    yolo.eval()

    return {"FCN-Small": small, "FCN-Deep": deep, "Tiny YOLO": yolo}


def infer_cnn(model: nn.Module, img_gray: np.ndarray,
              threshold: float = 0.5) -> np.ndarray:
    """Run any FCN model on a single greyscale image. Returns boolean 10x10 grid."""
    device = next(model.parameters()).device
    img = cv2.resize(img_gray, (IMG_W, IMG_H))
    x   = torch.from_numpy(img).float().unsqueeze(0).unsqueeze(0) / 127.5 - 1.0
    x   = x.to(device)
    with torch.no_grad():
        pred = torch.sigmoid(model(x).squeeze(0)).cpu().numpy()
    return pred > threshold


# ---------------------------------------------------------------------------
# Method 2 — Classical blob detection
# ---------------------------------------------------------------------------

def infer_blob(img_gray: np.ndarray,
               morph_kernel: int = 3) -> np.ndarray:
    """
    CLAHE → Otsu threshold (both polarities) → morphological open/close → CCL.

    Uses adaptive contrast enhancement + automatic Otsu thresholding instead of
    a fixed value, so it works across the full range of image brightnesses in
    the dataset (light-background and dark-background frames).
    """
    img = cv2.resize(img_gray, (IMG_W, IMG_H))

    # Blank-frame guard: if the image has almost no contrast, there are no
    # ships — return an empty mask immediately to avoid Otsu hallucinations.
    if img.std() < 4.0:
        return np.zeros_like(img)

    # Enhance local contrast so low-contrast boats become detectable
    clahe = cv2.createCLAHE(clipLimit=2.0, tileGridSize=(8, 8))
    enhanced = clahe.apply(img)

    def _detect(binary):
        kernel = cv2.getStructuringElement(
            cv2.MORPH_ELLIPSE, (morph_kernel, morph_kernel))
        opened = cv2.morphologyEx(binary, cv2.MORPH_OPEN,  kernel, iterations=1)
        closed = cv2.morphologyEx(opened,  cv2.MORPH_CLOSE, kernel, iterations=2)
        return closed

    # Otsu automatically finds the best threshold for this specific image
    _, dark_on_light = cv2.threshold(enhanced, 0, 255,
                                     cv2.THRESH_BINARY_INV | cv2.THRESH_OTSU)
    _, light_on_dark = cv2.threshold(enhanced, 0, 255,
                                     cv2.THRESH_BINARY     | cv2.THRESH_OTSU)

    mask_dol = _detect(dark_on_light)
    mask_lod = _detect(light_on_dark)

    n_dol = cv2.connectedComponents(mask_dol)[0] - 1
    n_lod = cv2.connectedComponents(mask_lod)[0] - 1
    MAX_SHIPS = 6

    # Prefer the polarity that gives 1–MAX_SHIPS blobs; break ties toward dol
    dol_ok = 0 < n_dol <= MAX_SHIPS
    lod_ok = 0 < n_lod <= MAX_SHIPS
    if dol_ok and (not lod_ok or n_dol >= n_lod):
        return mask_dol
    if lod_ok:
        return mask_lod
    return mask_dol


def nms_bboxes(bboxes: List[Tuple[int,int,int,int]],
               iou_thresh: float = 0.4) -> List[Tuple[int,int,int,int]]:
    """Greedy IoU-based NMS: suppress smaller box when two overlap heavily."""
    if not bboxes:
        return []
    # Sort by area descending so we keep the larger (likely correct) box
    sorted_boxes = sorted(bboxes,
                          key=lambda b: (b[2]-b[0])*(b[3]-b[1]),
                          reverse=True)
    kept = []
    for box in sorted_boxes:
        if all(iou(box, k) < iou_thresh for k in kept):
            kept.append(box)
    return kept


def ensemble_bboxes(blob_boxes: List[Tuple[int,int,int,int]],
                    sobel_boxes: List[Tuple[int,int,int,int]],
                    iou_thresh: float = 0.2) -> List[Tuple[int,int,int,int]]:
    """
    Combine blob and Sobel detections:
      - Any box agreed upon by both methods (IoU >= iou_thresh) → keep once
      - Sobel-only boxes → keep (high precision anchor)
      - Blob-only boxes  → keep (catches what Sobel misses)
    Then run NMS on the merged set to remove any remaining duplicates.

    This gives us Sobel's precision floor with Blob's recall ceiling.
    """
    all_boxes = list(blob_boxes) + list(sobel_boxes)
    return nms_bboxes(all_boxes, iou_thresh=iou_thresh)


def mask_to_bboxes(mask: np.ndarray,
                   min_area: int = 30,
                   max_area: int = 2000,
                   min_aspect: float = 1.3) -> List[Tuple[int,int,int,int]]:
    """Extract xyxy bboxes from a binary mask via CCL.

    Filters:
      min_area   — ignore tiny noise blobs
      max_area   — ignore oversized merged regions (split them instead)
      min_aspect — ships are elongated; reject near-square blobs
                   (aspect = max(w,h)/min(w,h), so 1.0 = perfect square)
    """
    num_labels, labels_im, stats, _ = cv2.connectedComponentsWithStats(
        mask, connectivity=8)
    bboxes = []
    for i in range(1, num_labels):
        area = stats[i, cv2.CC_STAT_AREA]
        if area < min_area:
            continue
        x = stats[i, cv2.CC_STAT_LEFT]
        y = stats[i, cv2.CC_STAT_TOP]
        w = stats[i, cv2.CC_STAT_WIDTH]
        h = stats[i, cv2.CC_STAT_HEIGHT]

        if w == 0 or h == 0:
            continue

        # Split oversized blobs (two ships merged) by eroding and re-running CCL
        if area > max_area:
            sub = (labels_im == i).astype(np.uint8) * 255
            kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (5, 5))
            eroded = cv2.erode(sub, kernel, iterations=2)
            sub_bboxes = mask_to_bboxes(eroded, min_area=min_area//4,
                                         max_area=max_area, min_aspect=min_aspect)
            bboxes.extend(sub_bboxes)
            continue

        aspect = max(w, h) / min(w, h)
        if aspect < min_aspect:
            continue

        bboxes.append((x, y, x+w, y+h))

    return nms_bboxes(bboxes)


# ---------------------------------------------------------------------------
# Method 3 — Sobel + threshold
# ---------------------------------------------------------------------------

def infer_sobel(img_gray: np.ndarray,
                edge_thresh: int = 30,
                morph_kernel: int = 5) -> np.ndarray:
    """
    Sobel magnitude → threshold → dilate to close contours → flood-fill
    interior → return binary mask.
    """
    img = cv2.resize(img_gray, (IMG_W, IMG_H))

    # Blank-frame guard
    if img.std() < 4.0:
        return np.zeros_like(img)

    blurred = cv2.GaussianBlur(img, (3, 3), 0)

    sx = cv2.Sobel(blurred, cv2.CV_16S, 1, 0, ksize=3)
    sy = cv2.Sobel(blurred, cv2.CV_16S, 0, 1, ksize=3)
    mag = cv2.magnitude(sx.astype(np.float32), sy.astype(np.float32))
    mag = np.clip(mag, 0, 255).astype(np.uint8)

    _, edge_bin = cv2.threshold(mag, edge_thresh, 255, cv2.THRESH_BINARY)

    kernel = cv2.getStructuringElement(
        cv2.MORPH_ELLIPSE, (morph_kernel, morph_kernel))
    dilated = cv2.dilate(edge_bin, kernel, iterations=1)

    # Zero the image border so edge artifacts at the frame margin don't create
    # closed contours that the flood-fill will treat as ship interiors
    border = 2
    edge_bin[:border, :]  = 0; edge_bin[-border:, :] = 0
    edge_bin[:, :border]  = 0; edge_bin[:, -border:] = 0
    dilated[:border, :]   = 0; dilated[-border:, :]  = 0
    dilated[:, :border]   = 0; dilated[:, -border:]  = 0

    # Flood-fill from border to find background, invert to get foreground
    flood = dilated.copy()
    h, w  = flood.shape
    mask  = np.zeros((h+2, w+2), dtype=np.uint8)
    cv2.floodFill(flood, mask, (0, 0), 255)
    interior = cv2.bitwise_not(flood)

    # Combine edges + interior
    combined = cv2.bitwise_or(edge_bin, interior)
    closed   = cv2.morphologyEx(combined, cv2.MORPH_CLOSE, kernel, iterations=1)
    return closed


# ---------------------------------------------------------------------------
# Grid-based detection evaluation helper
# ---------------------------------------------------------------------------

def grid_to_pixel_bboxes(grid: np.ndarray, label: GridLabel
                          ) -> List[Tuple[int,int,int,int]]:
    """Convert a boolean occupancy grid prediction back to pixel bboxes."""
    return prediction_grid_to_bboxes(grid, label.grid_top_left, label.cell_size)


def mask_to_grid(mask: np.ndarray, label: GridLabel) -> np.ndarray:
    """
    Map a pixel-space binary mask onto the label's grid by checking how much
    of each cell's area is covered.
    """
    grid = np.zeros((GRID_ROWS, GRID_COLS), dtype=bool)
    tx, ty = label.grid_top_left
    cs = label.cell_size
    for r in range(GRID_ROWS):
        for c in range(GRID_COLS):
            x1 = tx + c*cs; y1 = ty + r*cs
            x2 = min(x1+cs, IMG_W); y2 = min(y1+cs, IMG_H)
            if x2 <= x1 or y2 <= y1:
                continue
            cell_mask = mask[y1:y2, x1:x2]
            coverage  = cell_mask.mean() / 255.0
            grid[r, c] = coverage > 0.25
    return grid


# ---------------------------------------------------------------------------
# Visualisation
# ---------------------------------------------------------------------------

def draw_detections(img_gray: np.ndarray, label: GridLabel,
                    pred_bboxes: List[Tuple], method_name: str) -> np.ndarray:
    """Draw GT (green) and predicted (red) bboxes on a BGR image."""
    vis = cv2.cvtColor(cv2.resize(img_gray, (IMG_W, IMG_H)), cv2.COLOR_GRAY2BGR)

    for (x1, y1, x2, y2) in label.cell_bboxes:
        cv2.rectangle(vis, (x1,y1), (x2,y2), (0,200,0), 1)

    for (x1, y1, x2, y2) in pred_bboxes:
        cv2.rectangle(vis, (x1,y1), (x2,y2), (0,0,220), 1)

    cv2.putText(vis, method_name, (3, 10),
                cv2.FONT_HERSHEY_SIMPLEX, 0.35, (255,255,200), 1)
    return vis


def save_comparison_figure(label: GridLabel,
                            panels: List[Tuple[str, np.ndarray]],
                            out_path: str):
    """Save a grid comparison figure. >4 panels → 2 rows for readability."""
    n = len(panels)
    if n <= 4:
        nrows, ncols = 1, n
    else:
        ncols = (n + 1) // 2   # ceil(n/2) columns
        nrows = 2

    fig, axes = plt.subplots(nrows, ncols,
                             figsize=(4 * ncols, 3.5 * nrows),
                             facecolor="#1a1a2e")
    # Flatten axes to a 1-D list and hide any unused slots
    ax_flat = np.array(axes).flatten()
    for i, (title, img_bgr) in enumerate(panels):
        ax_flat[i].imshow(cv2.cvtColor(img_bgr, cv2.COLOR_BGR2RGB))
        ax_flat[i].set_title(title, color="white", fontsize=9, pad=4)
        ax_flat[i].axis("off")
    for j in range(len(panels), len(ax_flat)):
        ax_flat[j].axis("off")

    fig.suptitle(os.path.basename(label.image_path),
                 color="#e0e0ff", fontsize=10, y=1.01)
    plt.tight_layout()
    plt.savefig(out_path, dpi=120, bbox_inches="tight",
                facecolor=fig.get_facecolor())
    plt.close(fig)


# ---------------------------------------------------------------------------
# Main evaluation loop
# ---------------------------------------------------------------------------

def evaluate(labels: List[GridLabel],
             cnn_models: Optional[Dict[str, nn.Module]],
             output_dir: str,
             visualize: bool = True,
             n_vis: int = 10,
             iou_thresh: float = 0.3):

    os.makedirs(output_dir, exist_ok=True)
    vis_dir = os.path.join(output_dir, "visualisations")
    os.makedirs(vis_dir, exist_ok=True)

    cnn_metrics: Dict[str, Metrics] = {}
    if cnn_models:
        for name in cnn_models:
            cnn_metrics[name] = Metrics(f"CNN {name}")
    m_blob     = Metrics("Blob detection")
    m_sobel    = Metrics("Sobel + threshold")
    m_ensemble = Metrics("Ensemble (Blob + Sobel)")

    for idx, label in enumerate(tqdm(labels, desc="Evaluating")):
        img = cv2.imread(label.image_path, cv2.IMREAD_GRAYSCALE)
        if img is None:
            continue
        img = cv2.resize(img, (IMG_W, IMG_H))
        gt_bboxes = label.cell_bboxes

        panels = []

        # --- GT panel ---
        gt_vis = cv2.cvtColor(img, cv2.COLOR_GRAY2BGR)
        for (x1,y1,x2,y2) in gt_bboxes:
            cv2.rectangle(gt_vis, (x1,y1), (x2,y2), (0,200,0), 1)
        cv2.putText(gt_vis, "Ground Truth", (3,10),
                    cv2.FONT_HERSHEY_SIMPLEX, 0.35, (100,255,100), 1)
        panels.append(("Ground Truth", gt_vis))

        # --- CNN models + Tiny YOLO ---
        if cnn_models:
            for name, model in cnn_models.items():
                t0 = time.perf_counter()
                if isinstance(model, TinyYOLO):
                    boxes = infer_yolo(model, img)
                else:
                    grid  = infer_cnn(model, img)
                    boxes = grid_to_pixel_bboxes(grid, label)
                elapsed = (time.perf_counter() - t0) * 1000
                tp, fp, fn = match_detections(boxes, gt_bboxes, iou_thresh)
                cnn_metrics[name].update(tp, fp, fn, elapsed)
                panels.append((name, draw_detections(img, label, boxes, name)))

        # --- Blob ---
        t0   = time.perf_counter()
        blob_mask  = infer_blob(img)
        blob_boxes = mask_to_bboxes(blob_mask)
        elapsed    = (time.perf_counter() - t0) * 1000
        tp, fp, fn = match_detections(blob_boxes, gt_bboxes, iou_thresh)
        m_blob.update(tp, fp, fn, elapsed)
        panels.append(("Blob", draw_detections(img, label, blob_boxes, "Blob")))

        # --- Sobel ---
        t0    = time.perf_counter()
        sobel_mask  = infer_sobel(img)
        sobel_boxes = mask_to_bboxes(sobel_mask)
        elapsed     = (time.perf_counter() - t0) * 1000
        tp, fp, fn  = match_detections(sobel_boxes, gt_bboxes, iou_thresh)
        m_sobel.update(tp, fp, fn, elapsed)
        panels.append(("Sobel", draw_detections(img, label, sobel_boxes, "Sobel")))

        # --- Ensemble ---
        t0 = time.perf_counter()
        ens_boxes  = ensemble_bboxes(blob_boxes, sobel_boxes)
        elapsed    = (time.perf_counter() - t0) * 1000
        tp, fp, fn = match_detections(ens_boxes, gt_bboxes, iou_thresh)
        m_ensemble.update(tp, fp, fn, elapsed)
        panels.append(("Ensemble", draw_detections(img, label, ens_boxes, "Ensemble")))

        # --- Visualise first n_vis images ---
        if visualize and idx < n_vis:
            out_img = os.path.join(
                vis_dir, Path(label.image_path).stem + "_compare.png")
            save_comparison_figure(label, panels, out_img)

    # --- Print results ---
    print("\n" + "="*72)
    print("DETECTION RESULTS")
    print("="*72)
    for m in list(cnn_metrics.values()) + [m_blob, m_sobel, m_ensemble]:
        if m.n_images > 0:
            print(m)
    print("="*72)

    # --- Summary bar chart ---
    active = [m for m in list(cnn_metrics.values()) + [m_blob, m_sobel, m_ensemble]
              if m.n_images > 0]
    names  = [m.name.split("(")[0].strip() for m in active]
    prec   = [m.precision for m in active]
    rec    = [m.recall    for m in active]
    f1     = [m.f1        for m in active]

    x = np.arange(len(names))
    w = 0.25
    fig, ax = plt.subplots(figsize=(8, 4), facecolor="#1a1a2e")
    ax.set_facecolor("#1a1a2e")
    ax.bar(x - w, prec, w, label="Precision", color="#4fc3f7")
    ax.bar(x,     rec,  w, label="Recall",    color="#81c784")
    ax.bar(x + w, f1,   w, label="F1",        color="#ffb74d")
    ax.set_xticks(x); ax.set_xticklabels(names, color="white")
    ax.set_ylim(0, 1.05)
    ax.set_ylabel("Score", color="white")
    ax.set_title("Detection Method Comparison", color="white")
    ax.tick_params(colors="white")
    ax.legend(facecolor="#2a2a4e", labelcolor="white")
    for spine in ax.spines.values():
        spine.set_edgecolor("#444466")
    plt.tight_layout()
    chart_path = os.path.join(output_dir, "comparison_chart.png")
    plt.savefig(chart_path, dpi=120, facecolor=fig.get_facecolor())
    plt.close(fig)
    print(f"\nChart saved → {chart_path}")
    print(f"Visualisations → {vis_dir}/")


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="Battleship boat detection verification suite")
    parser.add_argument("--data-dir", "--data_dir", required=True, dest="data_dir",
                        help="Root directory containing images and JSON labels")
    parser.add_argument("--output-dir", "--output_dir", default="./results", dest="output_dir",
                        help="Where to write results and visualisations")
    parser.add_argument("--train",  action="store_true",
                        help="Train any model whose weights file is missing (FCN weights are reused if present)")
    parser.add_argument("--epochs", type=int, default=20,
                        help="Number of CNN training epochs (default 20)")
    parser.add_argument("--model-path", "--model_path", default="cnn_model.pt", dest="model_path",
                        help="Path to save / load CNN weights")
    parser.add_argument("--iou-thresh", "--iou_thresh", type=float, default=0.3, dest="iou_thresh",
                        help="IoU threshold for a true-positive match (default 0.3)")
    parser.add_argument("--visualize",  action="store_true",
                        help="Save annotated comparison images")
    parser.add_argument("--n-vis", "--n_vis", type=int, default=10, dest="n_vis",
                        help="Number of images to visualise (default 10)")
    parser.add_argument("--no-cnn", "--no_cnn", action="store_true", dest="no_cnn",
                        help="Skip CNN (evaluate classical methods only)")
    args = parser.parse_args()

    # Load splits
    splits = load_dataset(args.data_dir)
    train_labels = splits["train"]
    val_labels   = splits["val"]
    test_labels  = splits["test"]

    if not any([train_labels, val_labels, test_labels]):
        print("No valid labelled images found. Check --data_dir.")
        return

    # CNN — trained on train, validated on val, evaluated on test
    cnn_models = None
    stem, ext  = os.path.splitext(args.model_path)
    path_small = args.model_path
    path_deep  = f"{stem}_deep{ext}"

    if not args.no_cnn:
        if not train_labels:
            print("Warning: no training labels found — skipping CNN.")
        else:
            device    = get_device()
            path_yolo = f"{stem}_yolo{ext}"
            models    = {}

            vl = val_labels or train_labels[int(0.8*len(train_labels)):]
            tl = train_labels[:int(0.8*len(train_labels))] if not val_labels else train_labels

            # FCN-Small — load if weights exist, train only if missing or --train
            if not os.path.exists(path_small):
                print("\n── FCN-Small — training ───────────────────────────────────────")
                small = FCNSmall().to(device)
                pw    = compute_pos_weight(tl)
                train_one_cnn(small, tl, vl, args.epochs, path_small, pw)
                small.eval()
            else:
                print(f"Loading FCN-Small from {path_small}")
                small = FCNSmall().to(device)
                small.load_state_dict(torch.load(path_small, map_location=device,
                                                 weights_only=True))
                small.eval()
            models["FCN-Small"] = small

            # FCN-Deep — load if weights exist, train only if missing or --train
            if not os.path.exists(path_deep):
                print("\n── FCN-Deep — training ────────────────────────────────────────")
                deep = FCNDeep().to(device)
                pw   = compute_pos_weight(tl)
                train_one_cnn(deep, tl, vl, args.epochs, path_deep, pw)
                deep.eval()
            else:
                print(f"Loading FCN-Deep  from {path_deep}")
                deep = FCNDeep().to(device)
                deep.load_state_dict(torch.load(path_deep, map_location=device,
                                                weights_only=True))
                deep.eval()
            models["FCN-Deep"] = deep

            # Tiny YOLO — load if weights exist, train only if missing or --train
            if not os.path.exists(path_yolo):
                print("\n── Tiny YOLO — training ───────────────────────────────────────")
                yolo = TinyYOLO().to(device)
                train_yolo(yolo, tl, vl, args.epochs, path_yolo)
                yolo.eval()
            else:
                print(f"Loading Tiny YOLO from {path_yolo}")
                yolo = TinyYOLO().to(device)
                ckpt = torch.load(path_yolo, map_location=device, weights_only=False)
                if isinstance(ckpt, dict) and "state_dict" in ckpt:
                    yolo.load_state_dict(ckpt["state_dict"])
                    global YOLO_ANCHORS
                    YOLO_ANCHORS = ckpt.get("anchors", YOLO_ANCHORS)
                    print(f"  Loaded anchors: {YOLO_ANCHORS}")
                else:
                    yolo.load_state_dict(ckpt)
                yolo.eval()
            models["Tiny YOLO"] = yolo

            cnn_models = models

    # Evaluate on test split (fall back to val, then train if needed)
    eval_labels = test_labels or val_labels or train_labels
    split_name  = "test" if test_labels else ("val" if val_labels else "train")
    print(f"\nEvaluating on '{split_name}' split ({len(eval_labels):,} samples)")

    evaluate(eval_labels, cnn_models, args.output_dir,
             visualize=args.visualize,
             n_vis=args.n_vis,
             iou_thresh=args.iou_thresh)


if __name__ == "__main__":
    main()