# victory-frametime

A small patch for [VIctory](https://github.com/Marvisak/VIctory) and its community forks that makes
the GPU layer write a **frametime CSV**: one row per ~1 s summary interval, with the frame time and
the buckets the runtime already measures, plus the worst and average **single** frame over the
interval.

It contains **no game files or assets**. It is a patch against the open-source runtime (GPL-3.0).

## What it adds

- New module `crates/victory-gpu/src/frametime.rs`.
- `VICTORY_FRAMETIME_DUMP=<path>` writes the CSV there; `=1` writes `%TEMP%\victory-frametime.csv`.
- One `tick()` per present and one `record(...)` at the existing frame summary in `Host::frame`.
- Release-buildable: no `diagnostics` feature needed.

## For an agent (fastest path)

```sh
git clone <your victory fork>
cd <fork>
git am /path/to/victory-frametime.patch      # or: git apply -3 victory-frametime.patch
cargo build --release -p victory
```

Then run the title with:

```bat
set VICTORY_FRAMETIME_DUMP=%TEMP%\frametime.csv
victory run <game>\<title>.exe --execute
```

If `git am` fails because the fork has drifted, the change is four small pieces:

1. Copy `frametime.rs` into `crates/victory-gpu/src/`.
2. In `crates/victory-gpu/src/lib.rs`, add `mod frametime;` next to `mod frame;`.
3. In `crates/victory-gpu/src/queue.rs`, add
   `pub(crate) fn frame_count() -> u64 { FRAMES.load(Ordering::Relaxed) }`.
4. In `crates/victory-gpu/src/host.rs`, inside `fn frame(&mut self, hwnd: HWND)`:
   - call `crate::frametime::tick();` at the top (every present, before the 1 s early return);
   - insert `crate::frametime::record(&crate::frametime::Row { ... })` where the summary already
     computes `fps`, `ms`, `cpu`, `gpu`, `copy_ms`, `copy_mb`, `texture_ms`, `texture_mb`,
     `back_ms`, `back_mb`, `dropped`, `indirect`, `picked`, `picked_ms`, `card`, `twin_to_depth`,
     `twin_to_colour`, `planes`, and the card/texture pass values. `victory-frametime.patch` shows
     the exact insertion.

No `diagnostics` feature: the point is to profile release builds.

## Columns

`t_ms,frame,frames,fps,ms_per_frame,frame_ms_max,frame_ms_avg,cpu_prepare_ms,gpu_ms,gpu_pct,copies_ms,copies_mb,textures_ms,textures_mb,copyback_ms,copyback_mb,draws,dropped,making,indirect,heap_passes,heap_ms,texture_refresh_ms,card_passes,card_compares,card_sent_mb,card_ranges,card_pieces,card_fetched,card_fetched_mb,card_waits,card_wait_ms,twins_into_depth,twins_into_colour,depth_planes,ram_mb,cpu_pct`

| Column | Meaning |
|---|---|
| `frame_ms_max` / `frame_ms_avg` | worst / mean **single** frame between presents (load pauses count as gaps) |
| `cpu_prepare_ms` | CPU time building the frame's command lists |
| `gpu_ms` / `gpu_pct` | GPU time on our work / GPU busy share |
| `copies_ms` / `copies_mb` | CPU time / MB copying game memory for the GPU |
| `textures_ms` / `textures_mb` | CPU time / MB refreshing (copying + detiling) textures |
| `copyback_ms` / `copyback_mb` | time / MB reading GPU results back |
| `draws` / `dropped` | draw & mesh-dispatch calls attempted / skipped for lack of a pipeline |
| `heap_ms` | the by-index resource-table pass (`State::heap_buffers`): walk buffer views, resolve host copies, merge ranges, queue refreshes, pick textures. Bookkeeping, not the byte copy; overlaps `cpu_prepare_ms` |
| `texture_refresh_ms` | the actual texture refresh inside `textures_ms` |
| `card_*` | hash/compare passes over copies in GPU (card) memory, MB sent up/fetched, waits |
| `twins_*` / `depth_planes` | depth-texture/colour-twin copies and depth planes drawn from twins |

## What it found (sample)

RTX 5080 16 GB, Ryzen 7 9800X3D, 32 GB, 1440p, Series X profile, build `3346d57` (merge of
`ZSlayerHQ/victory-world-runtime` `next`). Settled world (last 40% of samples):

| bucket | default | `VICTORY_UNWATCHED_FRAMES=2` |
|---|---:|---:|
| fps | 4.07 | 4.41 |
| ms/frame | 259.6 | 226.9 |
| gpu_ms | 95.5 | 98.0 |
| cpu_prepare_ms | 69.1 | 64.4 |
| copies_ms | 41.3 | 37.5 |
| copyback_ms | 28.4 | 27.5 |
| textures_ms | 4.1 | 3.3 |
| heap_ms | 62.3 (median ~18) | 13.0 (median ~11) |
| draws/frame | 4,833 | 4,462 |
| dropped/frame | 415 | 351 |

The 1–2 s drops were dominated by `heap_ms` (1,000–2,164 ms) and `textures_ms` (up to 1,813 ms)
while the GPU was idle — i.e. the by-index/asset-streaming pass, not the GPU and not the file read.
Separately, 2.5–4.9 s gaps between presents showed no heap/GPU cost and are not attributed yet.

Not a controlled A/B: the two settle points had different draw counts.

## Samples

- `samples/report.md` — the writeup above in longer form.
- `samples/baseline.csv` — default switches, 176 intervals (~4 min).
- `samples/u2.csv` — `VICTORY_UNWATCHED_FRAMES=2`, 271 intervals.
- `analyze.ps1 <csv>` — prints means/medians, the slowest intervals, and the slowest single frames.

## Credits / license

VIctory is by Marvisak and contributors; this patch follows upstream's GPL-3.0. No game files or
assets are included or required.
