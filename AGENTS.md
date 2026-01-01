# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds all Rust code. The entry point is `src/main.rs`.
- `src/game/` contains gameplay logic (input, update loop, constants, powerups).
- `src/world/` handles map generation, tiles, and obstacles.
- `src/assets.rs` and `assets/` define and store textures/audio used by the game.
- `src/config.rs` centralizes tunable gameplay constants.
- `target/` is build output (generated, do not edit).

## Build, Test, and Development Commands
- `cargo run` runs the game locally.
- `cargo run -- --seed 123` runs with a deterministic RNG seed.
- `cargo run -- --render-frame` renders one frame and saves `debug_frame.png`.
- `cargo build` produces a debug build in `target/`.
- `cargo test` runs tests (none are present yet).
- `cargo fmt` formats Rust code using rustfmt.
- `cargo clippy` runs lint checks and suggestions.

## Coding Style & Naming Conventions
- Use rustfmt defaults (4-space indentation, standard formatting).
- Modules, functions, and variables use `snake_case`.
- Types and structs use `PascalCase`.
- Constants use `UPPER_SNAKE_CASE` (see `src/config.rs`).
- Keep modules small and focused; place gameplay changes in `src/game/` and world changes in `src/world/`.

## Testing Guidelines
- There are no automated tests yet; prefer adding `#[test]` unit tests in the relevant module.
- For integration tests, create files under `tests/` (e.g., `tests/world_generation.rs`).
- Use `cargo test` to run all tests.

## Commit & Pull Request Guidelines
- Commit history uses Conventional Commits (e.g., `feat: ...`, `refactor(input): ...`), with occasional short one-word maintenance commits.
- Keep subjects imperative and under ~72 characters.
- PRs should include a concise summary, testing notes (commands run), and screenshots for visual changes.
- Link related issues or tasks when applicable.

## Configuration & Assets
- Adjust gameplay constants in `src/config.rs` and keep changes minimal and justified.
- Keep asset filenames stable; update references in `src/assets.rs` when adding or renaming files.
