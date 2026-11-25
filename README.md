# victory-frametime — dev-only dump patch

The frametime dump from [skornuta/victory-fork](https://github.com/skornuta/victory-fork), packaged
as a patch any VIctory fork can apply. **This repo is the dev/tool repo**; the playable fork is the
main one. No game files or assets are included.

## What it adds

- `VICTORY_FRAMETIME_DUMP=<path>` writes a CSV; `=1` writes `<cache>\frametime.csv` (default
  `%LOCALAPPDATA%\victory\cache\frametime.csv`).
- One row per ~1 s summary interval with the frame time and the buckets the runtime already
  measures (`gpu_ms`, `cpu_prepare_ms`, `copies_ms`, `textures_ms`, `copyback_ms`, `heap_ms`,
  `card_*`, …), plus the worst/average **single** frame.
- A `#` header block with CPU, GPU + VRAM, **NVIDIA driver**, RAM, OS, board, display, the
  `VICTORY_*` switches and the title-build hash.
- Release-buildable; no `diagnostics` feature.

## For an agent (fastest)

```sh
git clone <your victory fork>
cd <fork>
git am /path/to/victory-frametime.patch      # or: git apply -3 victory-frametime.patch
cargo build --release -p victory
```

Then run the title with:

```bat
set VICTORY_FRAMETIME_DUMP=1
target\release\victory.exe run "D:\game\game.exe" --execute
```

or `run-frametime.cmd "D:\game\game.exe"` from the repo root.

Confirm the log prints `frametime dump path=...` when the first frame lands. The file is at
`%LOCALAPPDATA%\victory\cache\frametime.csv` (`victory config` prints `cache_dir`).

## Manual integration (if the patch doesn't apply)

The patch touches five places; use it as the exact reference.

1. Copy `frametime.rs` into `crates/victory-gpu/src/`.
2. `crates/victory-gpu/src/lib.rs`: add `pub mod frametime;` next to `pub mod frame;`.
3. `crates/victory-gpu/src/queue.rs`: add
   `pub(crate) fn frame_count() -> u64 { FRAMES.load(Ordering::Relaxed) }`.
4. `crates/victory-gpu/src/host.rs`:
   - add `pub(crate) fn adapter_summary()` (the DXGI adapter's name + dedicated VRAM);
   - call `crate::frametime::tick();` at the top of `fn frame(&mut self, hwnd)`;
   - hoist `card_passes`/`card_compares` and `texture_refresh_ms` out of their blocks;
   - insert `crate::frametime::record(&crate::frametime::Row { ... })` before the periodic
     `if self.logged.elapsed() >= 5 s` block, using the values already computed there.
5. `crates/victory-gpu/Cargo.toml`: add `"Win32_System_Registry"` to the `windows-sys` features.
6. `apps/victory/Cargo.toml`: add `victory-gpu.workspace = true`.
7. `apps/victory/src/run.rs`: after `Config::load()`, if `VICTORY_FRAMETIME_DUMP` is `1`, call
   `victory_gpu::frametime::use_dump_path(cfg.cache_dir.join("frametime.csv"))`; pass the detected
   build label with `victory_gpu::frametime::use_build_label(label)`.

## If you merge this into your fork

Add a short section to **your own README** (and to your agent instructions) saying how to enable
the dump and where it goes. Copy this:

```markdown
## Frametime dump
Build first: `cargo build --release -p victory`.
Run with `VICTORY_FRAMETIME_DUMP=1` (or `run-frametime.cmd "<title.exe>"`).
The CSV is at `%LOCALAPPDATA%\victory\cache\frametime.csv` (`victory config` prints `cache_dir`);
old patches wrote `%TEMP%\victory-frametime.csv`. The log prints `frametime dump path=...`.
```

Keep it in your README when the dump changes, so people who find your fork do not have to come back
here.

## Where the dump goes

| `VICTORY_FRAMETIME_DUMP` | File |
|---|---|
| unset or empty | no dump |
| `1` | `%LOCALAPPDATA%\victory\cache\frametime.csv` (follows `cache_dir`) |
| any other value | exactly that path (its folder must already exist) |

Created on the first presented frame (~1 s), truncated each run. The log prints
`frametime dump path=...` when it opens.

## If no file appears

- `git pull` without `cargo build --release -p victory` → still the old exe.
- Running a `victory.exe` from another folder or profile → run `target\release\victory.exe`.
- No `frametime dump path=...` in the log → that build has no dump, or it never presented a frame.
- Pre-`646f5cf` builds wrote `=1` to `%TEMP%\victory-frametime.csv`.
- PowerShell does not expand `%LOCALAPPDATA%`; use `$env:LOCALAPPDATA` or the `.cmd`.

## Keeping it in sync

This patch is generated from the main fork over the dump paths. After changing the dump in the main
fork, regenerate it there:

```sh
git diff --output=../victory-frametime/victory-frametime.patch 3346d57..HEAD -- \
  crates/victory-gpu/src/frametime.rs crates/victory-gpu/src/lib.rs \
  crates/victory-gpu/src/queue.rs crates/victory-gpu/src/host.rs \
  crates/victory-gpu/Cargo.toml apps/victory/Cargo.toml apps/victory/src/run.rs
```

Copy `crates/victory-gpu/src/frametime.rs` and `run-frametime.cmd` over too, then commit both repos.

## Samples

- `samples/baseline.csv` — pre-fold world, 176 intervals.
- `samples/u2.csv` — `VICTORY_UNWATCHED_FRAMES=2`, 271 intervals.
- `samples/frametime_5080_9800x3d_fold_2026-11-25_world.csv` — fold on, 628 intervals (~12 min).
- `samples/report.md` — the longer writeup.
- `analyze.ps1 <csv>` — means/medians, slowest intervals, slowest single frames (skips `#`).

## Credits / licence

VIctory is by Marvisak and contributors; this patch follows upstream's GPL-3.0. No game files.
