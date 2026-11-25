# Frametime sample — RTX 5080 / 9800X3D, fold on

Capture: 628 intervals (~735 s), world, fold buffers on (`VICTORY_FOLD_BUFFERS` default).
Build `931bee3` (main fork merge + fold + dump metadata). Driver 621.24.

The header of the CSV records CPU, GPU + VRAM, driver, RAM, OS, board, display, switches and the
game hash, so this file is self-describing.

## Settled tail (last 251 intervals)

| | pre-fold prologue | this run (fold on) |
|---|---:|---:|
| draws/frame | 4,833 | **5,636** |
| fps | 4.07 | **4.71** |
| ms/frame | 259.6 | **226.4** |
| `gpu_ms` | 95.5 | 100.2 |
| `cpu_prepare_ms` | 69.1 | 83.5 |
| `copies_ms` | 41.3 | 43.3 |
| `copyback_ms` | 28.4 | **167.5** |
| `textures_ms` | 4.1 | 9.9 |
| `heap_ms` | 62.3 | **41.6** |
| dropped/frame | 415 | 392 |

Not a controlled A/B: the scenes had different draw counts (pre-fold = prologue, this = a later
area). The direction matches the fold commit (4090: 179 → 160 ms, settles 4.5 → 1.0).

## Reading it

- **Fold is a net win here:** ~4.07 → **4.71 fps**, 260 → **226 ms**, while drawing ~17% more.
- `copyback_ms` rising to 167 ms is expected: folding removed duplicate-buffer settles, so the
  relay thread does the copy-back **in parallel** instead of the submitting thread waiting. It is
  thread time, not additive to the 226 ms.
- **New bottleneck:** the relay is at ~167 ms of a 226 ms frame (nearly saturated), then
  `gpu_ms` ~100 and `cpu_prepare_ms` ~83.
- Streaming transitions still spike: `heap_ms` 1.2–1.8 s, `textures_ms` 325–418 ms in the worst
  intervals.

## Slowest intervals

```
frame 4175: 1513 ms  heap  763  tex 418  copies  61  gpu  20  back  69
frame 4158: 1322 ms  heap 1236  tex  44  copies  42  gpu   0  back   6
frame 5760: 1314 ms  heap 1772  tex 325  copies  94  gpu  24  back 132
frame 4335: 1274 ms  heap   99  tex   8  copies  45  gpu 104  back 214
frame 4226: 1118 ms  heap 1365  tex  48  copies 135  gpu  39  back 131
```

Next candidates, both small ports from Trex's fork: `VICTORY_COPY_COMPARE` (only changed 4 KiB
pieces) and `VICTORY_CARD_BATCH` (one wait for copies settled together).
