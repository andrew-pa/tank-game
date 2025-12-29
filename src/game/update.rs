use raylib::prelude::RaylibHandle;

use crate::config::{
    BULLET_DAMAGE, BULLET_RADIUS, HEALTH_FLASH_TIME, RESPAWN_TIME, TANK_RADIUS, TRACK_LIFE,
};
use crate::entities::{Explosion, SmokeColor, Team};
use crate::math::{point_in_bounds, vec2, vec2_add, vec2_distance, vec2_scale};

use super::{Game, ScreenState};

impl Game {
    pub fn update(&mut self, dt: f32, rl: &RaylibHandle) {
        match self.state {
            ScreenState::Title => {
                if is_start_pressed(rl) {
                    self.reset_round();
                    self.state = ScreenState::Playing;
                }
            }
            ScreenState::Playing => {
                self.intro_timer = (self.intro_timer - dt).max(0.0);
                self.update_powerups(dt);
                if self.countdown_timer > 0.0 {
                    self.countdown_timer = (self.countdown_timer - dt).max(0.0);
                    return;
                }

                self.round_timer -= dt;
                if self.round_timer <= 0.0 {
                    self.round_timer = 0.0;
                    self.state = ScreenState::RoundOver;
                    self.last_winner = match self.team_kills[0].cmp(&self.team_kills[1]) {
                        std::cmp::Ordering::Greater => Some(Team::Red),
                        std::cmp::Ordering::Less => Some(Team::Blue),
                        std::cmp::Ordering::Equal => None,
                    };
                }
                self.update_tanks(dt, rl);
                self.update_bullets(dt);
                self.update_tracks(dt);
                self.update_explosions(dt);
            }
            ScreenState::RoundOver => {
                if is_start_pressed(rl) {
                    self.reset_round();
                    self.state = ScreenState::Playing;
                }
            }
        }
    }

    fn update_bullets(&mut self, dt: f32) {
        let mut survivors = Vec::with_capacity(self.bullets.len());
        for mut bullet in self.bullets.drain(..) {
            bullet.life -= dt;
            if bullet.life <= 0.0 {
                continue;
            }
            bullet.pos = vec2_add(bullet.pos, vec2_scale(bullet.vel, dt));

            if !point_in_bounds(bullet.pos, &self.world.world_bounds()) {
                continue;
            }
            if self
                .world
                .spawn_zones
                .iter()
                .any(|zone| zone.contains(bullet.pos) && zone.team != bullet.team)
            {
                continue;
            }

            let mut hit = false;
            for obstacle in &self.world.obstacles {
                if vec2_distance(bullet.pos, obstacle.pos) < obstacle.radius + BULLET_RADIUS {
                    self.explosions.push(Explosion {
                        pos: bullet.pos,
                        color: SmokeColor::Grey,
                        age: 0.0,
                    });
                    self.explosions.push(Explosion {
                        pos: vec2_add(bullet.pos, vec2(12.0, -8.0)),
                        color: SmokeColor::White,
                        age: 0.0,
                    });
                    hit = true;
                    break;
                }
            }
            if hit {
                continue;
            }

            for tank in &mut self.tanks {
                if tank.alive && tank.team != bullet.team {
                    if vec2_distance(bullet.pos, tank.pos) < TANK_RADIUS + BULLET_RADIUS {
                        if tank.invincible_timer <= 0.0 {
                            tank.health = (tank.health - BULLET_DAMAGE).max(0.0);
                            tank.health_flash = HEALTH_FLASH_TIME;
                            if tank.health <= 0.0 {
                                tank.alive = false;
                                tank.respawn_timer = RESPAWN_TIME;
                                self.team_kills[bullet.team.index()] += 1;
                                self.explosions.push(Explosion {
                                    pos: tank.pos,
                                    color: SmokeColor::Orange,
                                    age: 0.0,
                                });
                                self.explosions.push(Explosion {
                                    pos: vec2_add(tank.pos, vec2(-12.0, 10.0)),
                                    color: SmokeColor::Yellow,
                                    age: 0.0,
                                });
                            }
                        } else {
                            self.explosions.push(Explosion {
                                pos: bullet.pos,
                                color: SmokeColor::White,
                                age: 0.0,
                            });
                        }
                        hit = true;
                        break;
                    }
                }
            }
            if hit {
                continue;
            }

            survivors.push(bullet);
        }
        self.bullets = survivors;
    }

    fn update_tracks(&mut self, dt: f32) {
        self.tracks.retain_mut(|track| {
            track.age += dt;
            track.age < TRACK_LIFE
        });
    }

    fn update_explosions(&mut self, dt: f32) {
        self.explosions.retain_mut(|explosion| {
            explosion.age += dt;
            explosion.age < 0.48
        });
    }
}

fn is_start_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(raylib::prelude::KeyboardKey::KEY_ENTER)
        || rl.is_key_pressed(raylib::prelude::KeyboardKey::KEY_SPACE)
}
