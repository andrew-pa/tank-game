use rand::Rng;
use raylib::prelude::Vector2;

use crate::config::{
    HEALTH_FLASH_TIME, POWERUP_BASE_SPAWN, POWERUP_DURATION, POWERUP_MAX_COUNT, POWERUP_MAX_SPAWN,
    POWERUP_MIN_SPAWN, ROUND_TIME, TANK_RADIUS,
};
use crate::entities::{Powerup, PowerupKind, Tank};
use crate::math::{vec2, vec2_distance};

use super::Game;

impl Game {
    pub(super) fn update_powerups(&mut self, dt: f32) {
        for powerup in &mut self.powerups {
            powerup.age += dt;
        }

        self.powerup_spawn_timer -= dt;
        if self.powerup_spawn_timer <= 0.0 && self.powerups.len() < POWERUP_MAX_COUNT {
            if let Some(pos) = self.find_powerup_spawn() {
                let kind = match self.rng.random_range(0..3) {
                    0 => PowerupKind::Invincible,
                    1 => PowerupKind::RapidRange,
                    _ => PowerupKind::Heal,
                };
                self.powerups.push(Powerup {
                    kind,
                    pos,
                    age: 0.0,
                });
            }
            self.powerup_spawn_timer = self.next_powerup_spawn_delay();
        } else if self.powerup_spawn_timer <= 0.0 {
            self.powerup_spawn_timer = self.next_powerup_spawn_delay();
        }

        self.collect_powerups();
    }

    fn collect_powerups(&mut self) {
        if self.powerups.is_empty() {
            return;
        }
        let mut remaining = Vec::with_capacity(self.powerups.len());
        'outer: for powerup in self.powerups.drain(..) {
            for tank in &mut self.tanks {
                if tank.alive && vec2_distance(powerup.pos, tank.pos) < TANK_RADIUS + 22.0 {
                    apply_powerup(tank, powerup.kind);
                    continue 'outer;
                }
            }
            remaining.push(powerup);
        }
        self.powerups = remaining;
    }

    fn next_powerup_spawn_delay(&mut self) -> f32 {
        let progress = (1.0 - self.round_timer / ROUND_TIME).clamp(0.0, 1.0);
        let edge = (progress - 0.5).abs() * 2.0;
        let boost = 0.7 + edge * 0.8;
        let target = (POWERUP_BASE_SPAWN / boost).clamp(POWERUP_MIN_SPAWN, POWERUP_MAX_SPAWN);
        self.rng.random_range(target * 0.85..target * 1.15)
    }

    fn find_powerup_spawn(&mut self) -> Option<Vector2> {
        let bounds = self.world.world_bounds();
        let min_x = bounds.x + bounds.width * 0.2;
        let max_x = bounds.x + bounds.width * 0.8;
        let min_y = bounds.y + bounds.height * 0.2;
        let max_y = bounds.y + bounds.height * 0.8;
        for _ in 0..60 {
            let pos = vec2(
                self.rng.random_range(min_x..max_x),
                self.rng.random_range(min_y..max_y),
            );
            if self.world.is_inside_spawn_zone(pos) {
                continue;
            }
            if self
                .world
                .obstacles
                .iter()
                .any(|obs| vec2_distance(obs.pos, pos) < obs.radius + 36.0)
            {
                continue;
            }
            if self
                .tanks
                .iter()
                .any(|tank| vec2_distance(tank.pos, pos) < TANK_RADIUS + 50.0)
            {
                continue;
            }
            return Some(pos);
        }
        None
    }
}

fn apply_powerup(tank: &mut Tank, kind: PowerupKind) {
    match kind {
        PowerupKind::Invincible => {
            tank.invincible_timer = POWERUP_DURATION;
        }
        PowerupKind::RapidRange => {
            tank.rapid_timer = POWERUP_DURATION;
        }
        PowerupKind::Heal => {
            tank.health = tank.max_health;
            tank.health_flash = HEALTH_FLASH_TIME;
        }
    }
}
