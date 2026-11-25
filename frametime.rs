//! A frametime dump (`VICTORY_FRAMETIME_DUMP`): one CSV row per summary
//! interval, with the frame time and the buckets `host.rs` already measures
//! (preparing, copies, textures, GPU, copy-back, heap-buffer passes, texture
//! refreshes, the passes over the card's copies), plus the worst and average
//! frame over the interval, measured at every present.
//!
//! `VICTORY_FRAMETIME_DUMP=<path>` writes it there; `=1` writes
//! `%TEMP%\victory-frametime.csv`. The file is truncated at the first row.
//! This is deliberately release-buildable (no `diagnostics` feature): the
//! frames it is meant to explain are the slow ones.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static ENABLED: OnceLock<bool> = OnceLock::new();
static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static FILE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();

/// When the previous present happened and how long the frames since the
/// last summary took (microseconds, and a count).
static LAST: Mutex<Option<Instant>> = Mutex::new(None);
static MAX_US: AtomicU64 = AtomicU64::new(0);
static SUM_US: AtomicU64 = AtomicU64::new(0);
static COUNT: AtomicU64 = AtomicU64::new(0);

fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        let Some(value) = std::env::var_os("VICTORY_FRAMETIME_DUMP") else {
            return false;
        };
        if value.is_empty() {
            return false;
        }
        let path = if value.to_str() == Some("1") {
            std::env::temp_dir().join("victory-frametime.csv")
        } else {
            PathBuf::from(value)
        };
        let _ = PATH.set(Some(path));
        true
    })
}

fn file() -> Option<&'static Mutex<std::fs::File>> {
    FILE.get_or_init(|| {
        if !enabled() {
            return None;
        }
        let path = PATH.get().and_then(Option::as_ref)?;
        match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
        {
            Ok(mut file) => {
                let _ = file.write_all(HEADER.as_bytes());
                let _ = file.flush();
                tracing::info!(path = %path.display(), "frametime dump");
                Some(Mutex::new(file))
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "no frametime dump");
                None
            }
        }
    })
    .as_ref()
}

const HEADER: &str = "t_ms,frame,frames,fps,ms_per_frame,frame_ms_max,frame_ms_avg,\
cpu_prepare_ms,gpu_ms,gpu_pct,copies_ms,copies_mb,textures_ms,textures_mb,\
copyback_ms,copyback_mb,draws,dropped,making,indirect,heap_passes,heap_ms,\
texture_refresh_ms,card_passes,card_compares,card_sent_mb,card_ranges,\
card_pieces,card_fetched,card_fetched_mb,card_waits,card_wait_ms,\
twins_into_depth,twins_into_colour,depth_planes,ram_mb,cpu_pct\n";

/// Times one presented frame. Called for every present, before the frame
/// summary's own one-second gate, so a slow frame is seen even when the
/// summary is not printed.
pub(crate) fn tick() {
    if !enabled() {
        return;
    }
    let now = Instant::now();
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(previous) = *last {
        let micros = now.duration_since(previous).as_micros() as u64;
        MAX_US.fetch_max(micros, Ordering::Relaxed);
        SUM_US.fetch_add(micros, Ordering::Relaxed);
        COUNT.fetch_add(1, Ordering::Relaxed);
    }
    *last = Some(now);
}

/// The worst and average frame in the interval, in milliseconds, and how
/// many frames were timed; resets them.
pub(crate) fn take_interval() -> (f64, f64, u64) {
    let max = MAX_US.swap(0, Ordering::Relaxed) as f64 / 1000.0;
    let sum = SUM_US.swap(0, Ordering::Relaxed) as f64 / 1000.0;
    let count = COUNT.swap(0, Ordering::Relaxed);
    let avg = if count > 0 {
        sum / count as f64
    } else {
        0.0
    };
    (max, avg, count)
}

/// One summary interval's figures: times per frame in ms, sizes per frame in
/// MB, counts per frame.
#[derive(Default)]
pub(crate) struct Row {
    pub frame: u64,
    pub frames: f64,
    pub fps: f64,
    pub ms: f64,
    pub frame_ms_max: f64,
    pub frame_ms_avg: f64,
    pub cpu_prepare_ms: f64,
    pub gpu_ms: f64,
    pub gpu_pct: f64,
    pub copies_ms: f64,
    pub copies_mb: f64,
    pub textures_ms: f64,
    pub textures_mb: f64,
    pub copyback_ms: f64,
    pub copyback_mb: f64,
    pub draws: f64,
    pub dropped: f64,
    pub making: f64,
    pub indirect: f64,
    pub heap_passes: f64,
    pub heap_ms: f64,
    pub texture_refresh_ms: f64,
    pub card_passes: f64,
    pub card_compares: f64,
    pub card_sent_mb: f64,
    pub card_ranges: f64,
    pub card_pieces: f64,
    pub card_fetched: f64,
    pub card_fetched_mb: f64,
    pub card_waits: f64,
    pub card_wait_ms: f64,
    pub twins_into_depth: f64,
    pub twins_into_colour: f64,
    pub depth_planes: f64,
    pub ram_mb: f64,
    pub cpu_pct: f64,
}

/// Appends `r` when the dump is on.
pub(crate) fn record(r: &Row) {
    let Some(file) = file() else {
        return;
    };
    let t_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let mut file = file.lock().unwrap_or_else(|e| e.into_inner());
    let _ = writeln!(
        file,
        "{t_ms},{frame},{frames:.3},{fps:.3},{ms:.3},{max:.3},{avg:.3},\
         {cpu:.3},{gpu:.3},{gpu_pct:.3},{copies_ms:.3},{copies_mb:.3},\
         {textures_ms:.3},{textures_mb:.3},{copyback_ms:.3},{copyback_mb:.3},\
         {draws:.3},{dropped:.3},{making:.3},{indirect:.3},{heap_passes:.3},{heap_ms:.3},\
         {texture_refresh_ms:.3},{card_passes:.3},{card_compares:.3},{card_sent_mb:.3},\
         {card_ranges:.3},{card_pieces:.3},{card_fetched:.3},{card_fetched_mb:.3},\
         {card_waits:.3},{card_wait_ms:.3},{twins_into_depth:.3},{twins_into_colour:.3},\
         {depth_planes:.3},{ram_mb:.3},{cpu_pct:.3}",
        frame = r.frame,
        frames = r.frames,
        fps = r.fps,
        ms = r.ms,
        max = r.frame_ms_max,
        avg = r.frame_ms_avg,
        cpu = r.cpu_prepare_ms,
        gpu = r.gpu_ms,
        gpu_pct = r.gpu_pct,
        copies_ms = r.copies_ms,
        copies_mb = r.copies_mb,
        textures_ms = r.textures_ms,
        textures_mb = r.textures_mb,
        copyback_ms = r.copyback_ms,
        copyback_mb = r.copyback_mb,
        draws = r.draws,
        dropped = r.dropped,
        making = r.making,
        indirect = r.indirect,
        heap_passes = r.heap_passes,
        heap_ms = r.heap_ms,
        texture_refresh_ms = r.texture_refresh_ms,
        card_passes = r.card_passes,
        card_compares = r.card_compares,
        card_sent_mb = r.card_sent_mb,
        card_ranges = r.card_ranges,
        card_pieces = r.card_pieces,
        card_fetched = r.card_fetched,
        card_fetched_mb = r.card_fetched_mb,
        card_waits = r.card_waits,
        card_wait_ms = r.card_wait_ms,
        twins_into_depth = r.twins_into_depth,
        twins_into_colour = r.twins_into_colour,
        depth_planes = r.depth_planes,
        ram_mb = r.ram_mb,
        cpu_pct = r.cpu_pct,
    );
    let _ = file.flush();
}
