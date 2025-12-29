mod assets;
mod config;
mod entities;
mod game;
mod math;
mod world;

use std::time::{SystemTime, UNIX_EPOCH};

use assets::Assets;
use config::{WINDOW_HEIGHT, WINDOW_WIDTH};
use game::Game;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug_frame = args.iter().any(|arg| arg == "--render-frame");
    let seed_override = parse_seed(&args);

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tanks: Dominion")
        .resizable()
        .build();

    rl.set_target_fps(60);

    let seed = seed_override.unwrap_or_else(system_seed);
    let assets = Assets::load(&mut rl, &thread);
    let mut game = Game::new(seed);

    if debug_frame {
        game.update(1.0 / 60.0, &rl);
        {
            let mut d = rl.begin_drawing(&thread);
            game.draw(&mut d, &assets);
        }
        rl.take_screenshot(&thread, "debug_frame.png");
        return;
    }

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        game.update(dt, &rl);
        let mut d = rl.begin_drawing(&thread);
        game.draw(&mut d, &assets);
    }
}

fn parse_seed(args: &[String]) -> Option<u64> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--seed" {
            if let Some(value) = iter.next() {
                if let Ok(parsed) = value.parse::<u64>() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

fn system_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
