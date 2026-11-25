# Frametime dump — GTA6 on RTX 5080 (VIctory fork)

## What it is
A CSV written once per summary interval (~1 s) by `victory-gpu`, with the frame time and the
buckets the runtime already measures, plus the worst/mean **single** frame over the interval
(so a 3 s hitch is visible even when the interval average looks fine).

`VICTORY_FRAMETIME_DUMP=<path>` writes it; `=1` writes `%TEMP%\victory-frametime.csv`.
Release-buildable (no `diagnostics` feature). One line per second; negligible overhead.

Build: `main` @ `3346d57` = merge of `ZSlayerHQ/victory-world-runtime` branch `next` (`99f8834`)
+ the frametime patch. GPU: RTX 5080 16 GB. CPU: Ryzen 7 9800X3D. 32 GB RAM. Windows 11.
Series X profile, 1440p.

## Reproduce
```
victory warm <game>\shaders.rpf
set VICTORY_SHADER_LOOP_LIMIT=1024
set VICTORY_GROUP_SYNC=2
set VICTORY_RELAY=2
set VICTORY_LATE_WAIT=3
set VICTORY_WIDE_BACK=1
set VICTORY_CARD_BUFFERS_MIN_MB=1
set VICTORY_CARD_BUFFERS_MB=1024
set VICTORY_CARD_REHASH_RUNS=1
set VICTORY_JOIN_COPIES=1
set VICTORY_FRAMETIME_DUMP=%TEMP%\victory-frametime.csv
victory run <game>\game_scarlett_master.exe --execute
```

## Schema
```
t_ms,frame,frames,fps,ms_per_frame,frame_ms_max,frame_ms_avg,cpu_prepare_ms,gpu_ms,gpu_pct,
copies_ms,copies_mb,textures_ms,textures_mb,copyback_ms,copyback_mb,draws,dropped,making,indirect,
heap_passes,heap_ms,texture_refresh_ms,card_passes,card_compares,card_sent_mb,card_ranges,
card_pieces,card_fetched,card_fetched_mb,card_waits,card_wait_ms,twins_into_depth,
twins_into_colour,depth_planes,ram_mb,cpu_pct
```
- `frame_ms_max`/`frame_ms_avg`: worst/mean single frame between presents (load pauses count as gaps).
- `cpu_prepare_ms`: building the frame's command lists.
- `gpu_ms`: GPU time on our work.
- `copies_ms`/`copies_mb`: CPU time/MB copying game memory for the GPU.
- `textures_ms`: refreshing (copying + detiling) textures.
- `copyback_ms`: reading GPU results back into game memory.
- `heap_ms`: the by-index resource-table pass (`State::heap_buffers`): walks buffer views, resolves
  host copies, merges ranges, queues refreshes, and picks textures by index. Bookkeeping, not the
  byte copy. Overlaps `cpu_prepare_ms`; don't sum the columns.
- `dropped`: draws/mesh dispatches skipped for lack of a host pipeline.

## Settled world (last 40% of samples)
| bucket | baseline | `VICTORY_UNWATCHED_FRAMES=2` |
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

Caveat: not a controlled A/B — the settle points had different draw counts.

## Where the FPS drops are (baseline, slowest intervals)
```
frame 1588: 1993 ms  heap 1388  tex 335  copies  85  gpu 23  dropped 2547
frame 1661: 1812 ms  heap 2164  tex 220  copies 108  gpu 38  dropped  576
frame 1594: 1304 ms  heap 1599  tex  41  copies  64  gpu 46  dropped 1353
frame 1570: 1235 ms  heap  997  tex  88  copies  53  gpu 27  dropped 1085
```
The 1–2 s drops line up with `heap_ms` (by-index table pass) and `textures_ms`, while the GPU is idle.

## Separate: long gaps between presents
```
frame 1475: 3418 ms gap  heap 0  tex 179  copies 1  gpu 0
frame 1436: 3270 ms gap  heap 0  tex   7  copies 1  gpu 0
frame 1387: 3191 ms gap  heap 0  tex   8  copies 1  gpu 0
```
2.5–4.9 s gaps with no heap/GPU cost: these look like loading/preemption stalls the current buckets
cannot attribute.

## Asks for the devs
1. Is the by-index heap pass expected to be the top spike? It already skips unchanged slots but does a
   full sweep every 16 passes and walks every changed slot.
2. `VICTORY_UNWATCHED_FRAMES=2` cut `heap_ms` from ~62 to ~13 in the settled sample. Is a higher
   interval safe, or does it bring back the stale geometry / device loss it was set to avoid?
3. Can the dump get a bucket for the 2.5–4.9 s gaps between presents (file read / pipeline / page-in)?
4. DirectStorage reads are still synchronous, buffered `seek_read` in `dstorage.rs`; `copies_ms` +
   `copyback_ms` are ~27% of a frame. Is a host DirectStorage / overlapped read path planned?
   (RTX IO can't decode Xbox BCPack; the fork decodes it in software.)
