# Changelog

All notable changes to **Chorus** are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The project is pre-release and not yet versioned, so everything to date lives
under **Unreleased**; it will be split into a tagged `0.1.0` at the first release.

## [Unreleased](https://github.com/arihant-jain8/chorus) — 2026-11-05

### Added
- **Multi-output audio engine.** WASAPI loopback capture (`CaptureSource`) of the
  default output device, fanned out into one ring buffer per sink and played on
  every other device (`OutputSink`), conducted by `Router`.
- **Actor / headless core (`Chorus`).** A dedicated audio thread owns the `!Send`
  `Router`; the UI holds a `Send + Sync + Clone` handle built from an mpsc `Command`
  channel plus an `Arc<Mutex<State>>` snapshot.
- **Tauri v2 desktop app.** `get_state` command plus `set_volume`, `set_sink_mute`,
  `set_source_volume`, `set_source_mute`; Control view rendering the source card
  and one card per sink.
- **Per-sink software volume and mute**, applied in the output callback via
  `Arc<AtomicU32>` (f32 bits) and `Arc<AtomicBool>` — lock-free, instant.
- **Source volume and mute** on the default device through the OS endpoint
  (`volumecontrol`).
- **Source-follower.** The audio thread re-taps automatically when the Windows
  default output device changes.
- **Events rung (push UI updates).** The audio thread announces changes over a
  second `Event` channel; a Tauri `.setup` bridge thread `emit`s `state-changed`
  to the webview, which `listen`s and repaints. Replaces the old 1.5 s polling.
  `launch()` now returns `(Self, Receiver<Event>)`.
- **Dark color palette** matching the UI mockup.
- **Architecture map** at `docs/architecture.html` — a self-contained, expandable
  page documenting every struct, its fields and methods, and the audio, control,
  and event flows.
- **Roadmap** at `docs/ROADMAP.md`.
- **This changelog.**

### Changed
- Renamed `Router::set_volume` → `Router::set_sink_volume` (and updated callers).
- `serde` dependency now enables the `derive` feature explicitly, so the `chorus`
  crate builds standalone (previously relied on workspace feature unification).

### Fixed
- Tauri window not appearing — the engine loop ran inline in `launch()` so it
  never returned; moved `build()` + loop into a spawned thread.
- Several `main.js` render bugs in the sink loop (event-handler casing, element
  variable names, missing `appendChild` calls).
- Command registration / name mismatches between the frontend `invoke` names and
  the registered `#[tauri::command]` functions.

### Planned
See `docs/ROADMAP.md` for the full list. Next up:
- Smooth the UI repaint (update cards in place instead of a full rebuild).
- Live add/remove of sinks without a full rebuild.
- VU meters (needs events + RMS — now unblocked).
- Start/Stop control and a device picker.

[Unreleased]: https://github.com/arihant-jain8/chorus
