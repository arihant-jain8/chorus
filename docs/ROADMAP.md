# Chorus Roadmap

Where the project is and where it's going. Shipped work is tracked in
`../CHANGELOG.md`; this file is the forward-looking plan.

_Last updated: 2026-11-05_

## Done

- Multi-output engine: WASAPI loopback capture → fan-out → N sinks, conducted by `Router`.
- Actor core (`Chorus`): audio thread owns the `!Send` `Router`; UI holds a `Send + Sync + Clone` handle.
- Tauri v2 Control view: source + per-sink **volume** and **mute**.
- Source-follower: re-taps automatically on default-device change.
- **Events rung**: Rust→JS `emit`/`listen`, replacing UI polling.

## Near-term polish

- **Smooth the UI repaint.** `renderState` currently wipes and rebuilds every card
  (`sinksEl.innerHTML = ""`), so an event causes a visible flash and resets a slider
  mid-drag. Update cards in place — diff the new `State` against what's drawn and
  touch only what changed. _Unblocked now that events drive re-renders._
- **Volume ramping.** Gain jumps to the new value in one callback, which can click
  ("zipper noise"). Step the gain toward the target over a few samples in the output
  callback.

## Core features

Each needs one new piece of plumbing:

- **Start / Stop** — stop and restart routing without quitting. Needs a stop path in
  the actor: `Command::Stop` drops the `Router` (killing all streams), `Command::Start`
  rebuilds it; wire `running` to the UI.
- **Device picker** — choose which outputs to mirror instead of "all non-default".
  `State.devices` already exists; add UI selection + a `Command` to rebuild with a
  chosen sink set.
- **Live add / remove a sink** while running — the ownership problem: `producers` is
  moved into the capture closure and sealed. Either keep the rebuild-on-change path, or
  add `Command::AddSink(device)` with a shared / lock-free producer list so the capture
  callback can learn about new rings without a full teardown.
- **VU meters** — live per-device level bars. Compute **RMS** in the callback and push
  it up via a lightweight event. _Unblocked by the events rung._

## Bigger tracks

- **Track B / RES — Resampling & drift.** Each device has its own clock; over time they
  drift and the ring buffer under/overruns. Proper fix is per-sink resampling to keep
  each output fed regardless of its sample rate.
- **Track C — Per-app routing.** Capture and route a specific application's audio (e.g.
  Spotify to one device, Discord to another) rather than the whole system mix — a deeper
  WASAPI session-enumeration topic.
- **NET — Phone over LAN.** Stream a sink's audio to a phone or another machine over the
  network, as another virtual device.
