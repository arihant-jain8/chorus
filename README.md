# Chorus

Chorus is a desktop audio router. It captures audio from the default output
device and mirrors it to other available output devices, with per-device
volume and mute controls.

The project is organized around a reusable Rust audio engine and a Tauri
desktop frontend.

![Chorus desktop screenshot](chorus_ss.jfif)

## Features

- Mirror audio to available output devices.
- Adjust source and output volumes.
- Mute the source or individual outputs.
- Rebuild the audio router when the default output device changes.

## Project structure

```text
crates/
  chorus/       Core audio engine and command-line examples
  chorus-egui/  Additional eframe/egui frontend
desktop/
  src/          Tauri frontend (HTML, CSS, and JavaScript)
  src-tauri/    Tauri application and Rust-to-frontend commands
Cargo.toml      Rust workspace manifest
chorus_ss.jfif  Application screenshot
```

The `chorus` crate contains the audio capture, routing, and output logic. The
Tauri application in `desktop/` uses that crate and exposes the engine state
and controls to the frontend.

## Requirements

- Rust and Cargo (the workspace uses Rust edition 2024).
- Bun, to install the desktop frontend's Tauri CLI dependency.
- The platform prerequisites for [Tauri 2](https://v2.tauri.app/start/prerequisites/).
- Audio input/output devices supported by [CPAL](https://github.com/RustAudio/cpal).

## Run the desktop app

From the repository root:

```sh
cd desktop
bun install
bun run tauri dev
```

To create a desktop bundle:

```sh
bun run tauri build
```

## Work with the Rust engine

Check the core crate:

```sh
cargo check -p chorus
```

Run the interactive command-line example:

```sh
cargo run -p chorus --example cli
```

The CLI lists active output devices. Enter an output index and volume between
`0.0` and `1.0` to adjust an output, or enter `s` and a volume to adjust the
source. Enter `q` to quit.

## License

No license has been specified yet.
