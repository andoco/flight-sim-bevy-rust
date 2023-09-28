[![Continuous integration](https://github.com/andoco/flight-sim-bevy-rust/actions/workflows/continuous_integration.yml/badge.svg)](https://github.com/andoco/flight-sim-bevy-rust/actions/workflows/continuous_integration.yml)

Flight simulation using the [Bevy](https://bevyengine.org/) game engine and [Rapier](https://rapier.rs/) physics engine.

[Play online](https://andoco.github.io/flight-sim-bevy-rust/)

## Run

```sh
cargo run
```

As WASM website:

```sh
cargo run --target wasm32-unknown-unknown
```

or

```sh
trunk serve --release
```

## Controls

### Keyboard

- Throttle: A/Z
- Elevators: Up/Down
- Rudder: Q/W
- Ailerons: Left/Right
- Rear view: F1
- Top view: F2
- Side view: F3
- Cockpit view: F4

### Gamepad

- Throttle: Right stick Y axis
- Elevators: Left stick Y axis
- Rudder: Right stick X axis
- Ailerons: Left stick X axis
- Rear view: DPad down
- Top view: DPad up
- Side view: Dpad right
- Cockpit view: Dpad left

## Useful resources

- [Fundamentals Of Aircraft Design](https://aerotoolbox.com/category/intro-aircraft-design/)
