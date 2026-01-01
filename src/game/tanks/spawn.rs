use rand::{Rng, rngs::SmallRng};
use raylib::prelude::Vector2;

use crate::config::{FIRE_COOLDOWN, MAX_HEALTH, TANK_SPEED, TANKS_PER_TEAM, TILE_SIZE};
use crate::entities::{Tank, Team};
use crate::math::{random_angle, vec2};
use crate::world::World;

pub(super) fn spawn_tanks(rng: &mut SmallRng, world: &World) -> Vec<Tank> {
    let mut tanks = Vec::new();
    for team in [Team::Red, Team::Blue] {
        for _ in 0..TANKS_PER_TEAM {
            let pos = world.random_point_in_zone(team, rng);
            let angle = random_angle(rng);
            tanks.push(Tank {
                team,
                pos,
                body_angle: angle,
                turret_angle: angle,
                speed: TANK_SPEED + rng.random_range(-12.0..14.0),
                fire_cooldown: rng.random_range(0.0..0.8),
                alive: true,
                respawn_timer: 0.0,
                waypoint: pick_waypoint(world, team, rng),
                track_distance: rng.random_range(0.0..40.0),
                tread_phase: rng.random_range(0.0..3.0),
                health: MAX_HEALTH,
                max_health: MAX_HEALTH,
                health_flash: 0.0,
                invincible_timer: 0.0,
                rapid_timer: 0.0,
            });
        }
    }
    tanks
}

pub(super) fn respawn_tank(tank: &mut Tank, world: &World, rng: &mut SmallRng) {
    tank.alive = true;
    tank.pos = world.random_point_in_zone(tank.team, rng);
    tank.body_angle = random_angle(rng);
    tank.turret_angle = tank.body_angle;
    tank.fire_cooldown = FIRE_COOLDOWN * 0.5;
    tank.health = tank.max_health;
    tank.health_flash = 0.0;
    tank.invincible_timer = 0.0;
    tank.rapid_timer = 0.0;
}

pub(super) fn pick_waypoint(world: &World, team: Team, rng: &mut SmallRng) -> Vector2 {
    for _ in 0..40 {
        let bounds = world.world_bounds();
        let margin = TILE_SIZE * 1.2;
        let pos = vec2(
            rng.random_range(bounds.x + margin..bounds.x + bounds.width - margin),
            rng.random_range(bounds.y + margin..bounds.y + bounds.height - margin),
        );
        if !world.is_inside_enemy_zone(team, pos) {
            return pos;
        }
    }
    world.random_point_in_zone(team, rng)
}
