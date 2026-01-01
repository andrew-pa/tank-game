mod ai;
mod collisions;
mod combat;
mod modifiers;
mod movement;
mod player;
mod spawn;

use raylib::prelude::{RaylibHandle, Vector2};

use crate::entities::{Tank, Team};
use crate::world::World;

use super::Game;

pub(super) fn spawn_tanks(rng: &mut rand::rngs::SmallRng, world: &World) -> Vec<Tank> {
    spawn::spawn_tanks(rng, world)
}

impl Game {
    pub(super) fn update_tanks(&mut self, dt: f32, rl: &RaylibHandle) {
        let snapshot = collect_snapshot(&self.tanks);
        let mut new_bullets = Vec::new();
        let mut new_tracks = Vec::new();
        let world = &self.world;
        let camera = self.camera(rl.get_screen_width(), rl.get_screen_height());
        let mouse_world = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);
        let player_input = self.input_state.player_input(rl);

        for (index, tank) in self.tanks.iter_mut().enumerate() {
            movement::update_tank_timers(tank, dt);

            if !tank.alive {
                tank.respawn_timer -= dt;
                if tank.respawn_timer <= 0.0 {
                    spawn::respawn_tank(tank, world, &mut self.rng);
                }
                continue;
            }

            tank.fire_cooldown = (tank.fire_cooldown - dt).max(0.0);

            if index == self.player_index {
                player::update_player_tank(
                    tank,
                    dt,
                    world,
                    mouse_world,
                    &player_input,
                    &mut new_tracks,
                    &mut new_bullets,
                );
                continue;
            }
            ai::update_ai_tank(
                tank,
                dt,
                world,
                &snapshot,
                &mut self.rng,
                &mut new_tracks,
                &mut new_bullets,
            );
        }

        self.bullets.extend(new_bullets);
        self.tracks.extend(new_tracks);
        self.resolve_tank_collisions();
    }

    fn resolve_tank_collisions(&mut self) {
        collisions::resolve_tank_collisions(&mut self.tanks, &self.world);
    }
}

fn collect_snapshot(tanks: &[Tank]) -> Vec<(Team, Vector2, bool)> {
    tanks
        .iter()
        .map(|tank| (tank.team, tank.pos, tank.alive))
        .collect()
}
