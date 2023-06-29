Flight simulation using the [Bevy](https://bevyengine.org/) game engine and [Rapier](https://rapier.rs/) physics engine.

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

## Flight dynamics

- Roll is rotation around the front-to-back axis, or z-axis.
- Pitch is rotation around the side-to-side axis, or x-axis.
- Yaw is rotation around the vertical axis, or y-axis.

Roll is controlled using the ailerons on the main wings by raising them in opposing directions and creating more lift on one side and less on the other. This changes the direction of lift to point to one side and causes sideways movement of the plane.

Pitch is controlled using the elevators on the horizontal tail wings by raising them to create less lift and dropping the tail, or lowering them to create more lift and raising the tail.

Yaw is controlled using the rudder on the vertical tail fin by creating more lift in a sideways direction and pulling the tail in one direction.

Turning the plane is usually achieved by using the the rudder and the ailerons together.

## Calculating lift

`L = 0.5 * p * v^2 * Sref * CL`

- L denotes lift force.
- V defines the velocity of aircraft expressed in m/s.
- ρ is air density, affected by altitude. Air densitry is 1.225 kg/m^3 at sea level.
- Sref is the reference area or the wing area of an aircraft measured in square metres.
- CL is the coefficient of lift, depending on the angle of attack and the type of aerofoil.
