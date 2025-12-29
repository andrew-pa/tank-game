mod constants;
mod powerups;
mod render;
mod tanks;
mod update;

use rand::{rngs::SmallRng, SeedableRng};
use raylib::prelude::Camera2D;

use crate::config::{
    PLAYER_INTRO_TIME, POWERUP_BASE_SPAWN, ROUND_COUNTDOWN, ROUND_TIME, TILE_SIZE, WINDOW_HEIGHT,
    WINDOW_WIDTH,
};
use crate::entities::{Bullet, Explosion, Powerup, Tank, Team, TrackMark};
use crate::math::{vec2, vec2_add, vec2_scale};
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreenState {
    Title,
    Playing,
    RoundOver,
}

pub struct Game {
    state: ScreenState,
    world: World,
    tanks: Vec<Tank>,
    bullets: Vec<Bullet>,
    tracks: Vec<TrackMark>,
    explosions: Vec<Explosion>,
    powerups: Vec<Powerup>,
    rng: SmallRng,
    round_timer: f32,
    countdown_timer: f32,
    intro_timer: f32,
    powerup_spawn_timer: f32,
    team_kills: [u32; 2],
    last_winner: Option<Team>,
    player_index: usize,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);
        let world = World::new(&mut rng);
        let tanks = tanks::spawn_tanks(&mut rng, &world);
        let mut game = Self {
            state: ScreenState::Title,
            world,
            tanks,
            bullets: Vec::new(),
            tracks: Vec::new(),
            explosions: Vec::new(),
            powerups: Vec::new(),
            rng,
            round_timer: ROUND_TIME,
            countdown_timer: ROUND_COUNTDOWN,
            intro_timer: PLAYER_INTRO_TIME,
            powerup_spawn_timer: POWERUP_BASE_SPAWN,
            team_kills: [0, 0],
            last_winner: None,
            player_index: 0,
        };
        game.reset_round();
        game.state = ScreenState::Title;
        game
    }

    fn reset_round(&mut self) {
        self.world = World::new(&mut self.rng);
        self.tanks = tanks::spawn_tanks(&mut self.rng, &self.world);
        self.bullets.clear();
        self.tracks.clear();
        self.explosions.clear();
        self.powerups.clear();
        self.round_timer = ROUND_TIME;
        self.countdown_timer = ROUND_COUNTDOWN;
        self.intro_timer = PLAYER_INTRO_TIME;
        self.powerup_spawn_timer = POWERUP_BASE_SPAWN;
        self.team_kills = [0, 0];
        self.last_winner = None;
        self.player_index = self
            .tanks
            .iter()
            .position(|tank| tank.team == Team::Red)
            .unwrap_or(0);
    }

    fn camera(&self) -> Camera2D {
        let mut center = self
            .tanks
            .get(self.player_index)
            .filter(|tank| tank.alive)
            .map(|tank| tank.pos);

        if center.is_none() {
            let mut sum = vec2(0.0, 0.0);
            let mut count = 0.0;
            for tank in &self.tanks {
                if tank.alive {
                    sum = vec2_add(sum, tank.pos);
                    count += 1.0;
                }
            }
            if count > 0.0 {
                center = Some(vec2_scale(sum, 1.0 / count));
            } else {
                center = Some(vec2(
                    self.world.width as f32 * TILE_SIZE * 0.5,
                    self.world.height as f32 * TILE_SIZE * 0.5,
                ));
            }
        }

        Camera2D {
            target: center.unwrap_or_else(|| {
                vec2(
                    self.world.width as f32 * TILE_SIZE * 0.5,
                    self.world.height as f32 * TILE_SIZE * 0.5,
                )
            }),
            offset: vec2(WINDOW_WIDTH as f32 * 0.5, WINDOW_HEIGHT as f32 * 0.55),
            rotation: 0.0,
            zoom: 0.55,
        }
    }
}
