"""
Convert template images to RGBA with transparent background.

Uses flood-fill from image corners to detect connected background regions,
avoiding false positives inside the button/icon.

Usage:
    python scripts/make_template_transparent.py <input.png> [output.png] [--threshold 30]

If output is omitted, overwrites the input file.
"""
import sys
import numpy as np
from PIL import Image
from collections import deque


def flood_fill_mask(img_array, threshold=30):
    """Flood fill from corners to find connected background pixels."""
    h, w = img_array.shape[:2]
    visited = np.zeros((h, w), dtype=bool)
    bg_mask = np.zeros((h, w), dtype=bool)

    # Sample background color from corners (3x3 patches)
    corners = []
    for cy, cx in [(0,0), (0,w-1), (h-1,0), (h-1,w-1)]:
        for dy in range(min(3, h)):
            for dx in range(min(3, w)):
                ny = min(cy + dy, h-1) if cy == 0 else max(cy - dy, 0)
                nx = min(cx + dx, w-1) if cx == 0 else max(cx - dx, 0)
                corners.append(img_array[ny, nx])
    bg_color = np.median(corners, axis=0)

    # BFS flood fill from all 4 corners
    queue = deque()
    seeds = [(0,0), (0,w-1), (h-1,0), (h-1,w-1)]
    for sy, sx in seeds:
        if not visited[sy, sx]:
            dist = np.sqrt(np.sum((img_array[sy, sx].astype(float) - bg_color) ** 2))
            if dist <= threshold:
                queue.append((sy, sx))
                visited[sy, sx] = True
                bg_mask[sy, sx] = True

    while queue:
        y, x = queue.popleft()
        for dy, dx in [(-1,0),(1,0),(0,-1),(0,1)]:
            ny, nx = y + dy, x + dx
            if 0 <= ny < h and 0 <= nx < w and not visited[ny, nx]:
                visited[ny, nx] = True
                pixel = img_array[ny, nx].astype(float)
                dist = np.sqrt(np.sum((pixel - bg_color) ** 2))
                if dist <= threshold:
                    bg_mask[ny, nx] = True
                    queue.append((ny, nx))

    return bg_mask, bg_color


def make_transparent(input_path, output_path=None, threshold=30):
    img = Image.open(input_path).convert("RGB")
    arr = np.array(img, dtype=np.uint8)

    bg_mask, bg_color = flood_fill_mask(arr, threshold)

    print(f"Detected background color: RGB({bg_color[0]:.0f}, {bg_color[1]:.0f}, {bg_color[2]:.0f})")

    alpha = np.where(bg_mask, 0, 255).astype(np.uint8)

    rgba = np.dstack([arr, alpha])
    result = Image.fromarray(rgba, "RGBA")

    out = output_path or input_path
    result.save(out)

    opaque_pct = (alpha > 128).sum() / alpha.size * 100
    print(f"Result: {result.size[0]}x{result.size[1]} RGBA, {opaque_pct:.1f}% opaque")
    print(f"Saved to: {out}")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(
        description="Convert template to RGBA with transparent background")
    parser.add_argument("input", help="Input PNG file")
    parser.add_argument("output", nargs="?",
                        help="Output PNG (default: overwrite input)")
    parser.add_argument("--threshold", type=int, default=30,
                        help="Color distance threshold (default: 30)")
    args = parser.parse_args()
    make_transparent(args.input, args.output, args.threshold)
