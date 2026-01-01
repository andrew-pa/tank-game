# Tanks: Dominion

Tanks: Dominion is a top‑down, squad‑based tank skirmish. Two teams (Crimson vs. Azure) battle across a procedurally generated frontier, racing to score the most eliminations before the round timer expires.

<video controls loop muted playsinline width="720">
  <source src="demo.webm" type="video/webm" />
  Your browser does not support the video tag.
</video>

[Watch the gameplay demo](demo.webm)

## Gameplay at a Glance
- **Teams:** 4 tanks per side; you control one Crimson tank, the rest are AI.
- **Rounds:** 120 seconds with a short deploy countdown. Highest kill count wins; ties are a stalemate.
- **Respawns:** Eliminated tanks return after a brief respawn timer.
- **Spawn zones:** You cannot enter the enemy spawn zone, and enemy bullets vanish if they enter it.
- **Procedural map:** Each round spawns a new layout of tiles and obstacles.

## Controls
**Keyboard + Mouse**
- **Move:** `W/S` or `Up/Down`
- **Turn:** `A/D` or `Left/Right`
- **Aim turret:** mouse
- **Fire:** Left Mouse Button or `Space`
- **Start/Continue:** `Enter`

**Gamepad**
- **Move/Turn:** Left Stick or D‑Pad
- **Aim turret:** Right Stick
- **Fire:** Right Trigger (RT) / bottom face button
- **Start/Continue:** Start / A

The game automatically switches to the last active input device.

## Powerups
Powerups spawn periodically near mid‑map (up to 3 active at once). Pick them up by driving over them. Each powerup looks like a floating puff of smoke with a pulsing ring; the color tells you which one it is.
- **Invincible (icy blue/white):** temporary invulnerability plus a small speed boost.
- **Rapid Range (warm orange):** faster fire rate and longer bullet range.
- **Heal (green glow with yellow smoke):** restores the tank to full health instantly.

## Build & Run
Requires Rust (edition 2024) and a working `cargo` toolchain.

```bash
cargo run
```

Helpful flags:
```bash
cargo run -- --seed 123         # deterministic world + spawns
cargo run -- --render-frame     # saves debug_frame.png then exits
```

For an optimized build:
```bash
cargo build --release
```

## Development Notes
- Assets live in `assets/` and are wired in `src/assets.rs`. Thanks [Kenney](https://www.kenney.nl)!
- Gameplay tuning constants are centralized in `src/config.rs`.
