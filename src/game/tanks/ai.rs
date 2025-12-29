use rand::rngs::SmallRng;
use raylib::prelude::Vector2;

use crate::config::{BODY_ROT_SPEED, TURRET_ROT_SPEED};
use crate::entities::{Bullet, Tank, Team, TrackMark};
use crate::math::{
    angle_difference, vec2, vec2_add, vec2_angle, vec2_distance, vec2_length,
    vec2_normalize, vec2_scale, vec2_sub, rotate_towards,
};
use crate::world::World;

use super::combat::fire_bullet;
use super::super::constants::{AI_FIRE_RANGE, AI_TARGET_FAR, AI_TARGET_NEAR};
use super::modifiers::{range_multiplier, speed_multiplier};
use super::movement::advance_tank;
use super::spawn::pick_waypoint;

pub(super) fn update_ai_tank(
    tank: &mut Tank,
    dt: f32,
    world: &World,
    snapshot: &[(Team, Vector2, bool)],
    rng: &mut SmallRng,
    new_tracks: &mut Vec<TrackMark>,
    new_bullets: &mut Vec<Bullet>,
) {
    let (target_pos, target_dist) = find_target_snapshot(snapshot, tank.team, tank.pos);
    let desired_dir = if let Some(target) = target_pos {
        if target_dist > AI_TARGET_FAR {
            vec2_normalize(vec2_sub(target, tank.pos))
        } else if target_dist < AI_TARGET_NEAR {
            vec2_normalize(vec2_sub(tank.pos, target))
        } else {
            let to_target = vec2_sub(target, tank.pos);
            vec2_normalize(vec2(-to_target.y, to_target.x))
        }
    } else {
        let to_waypoint = vec2_sub(tank.waypoint, tank.pos);
        if vec2_length(to_waypoint) < 80.0 {
            tank.waypoint = pick_waypoint(world, tank.team, rng);
        }
        vec2_normalize(to_waypoint)
    };

    if let Some(target) = target_pos {
        let target_angle = vec2_angle(vec2_sub(target, tank.pos));
        tank.turret_angle = rotate_towards(tank.turret_angle, target_angle, TURRET_ROT_SPEED * dt);

        if angle_difference(tank.turret_angle, target_angle) < 0.22
            && tank.fire_cooldown <= 0.0
            && target_dist < AI_FIRE_RANGE * range_multiplier(tank)
        {
            fire_bullet(tank, new_bullets);
        }
    } else {
        tank.turret_angle =
            rotate_towards(tank.turret_angle, tank.body_angle, TURRET_ROT_SPEED * dt);
    }

    let avoidance = avoidance_vector(world, tank.team, tank.pos);
    let steer = vec2_normalize(vec2_add(desired_dir, vec2_scale(avoidance, 1.4)));
    if vec2_length(steer) > 0.1 {
        let target_angle = vec2_angle(steer);
        tank.body_angle = rotate_towards(tank.body_angle, target_angle, BODY_ROT_SPEED * dt);
        advance_tank(tank, dt, speed_multiplier(tank), world, new_tracks);
    }
}

fn find_target_snapshot(
    snapshot: &[(Team, Vector2, bool)],
    team: Team,
    pos: Vector2,
) -> (Option<Vector2>, f32) {
    let mut best = None;
    let mut best_dist = f32::MAX;
    for (other_team, other_pos, alive) in snapshot {
        if *alive && *other_team != team {
            let dist = vec2_distance(*other_pos, pos);
            if dist < best_dist {
                best_dist = dist;
                best = Some(*other_pos);
            }
        }
    }
    (best, best_dist)
}

fn avoidance_vector(world: &World, team: Team, pos: Vector2) -> Vector2 {
    let mut steer = vec2(0.0, 0.0);
    for obstacle in &world.obstacles {
        let dist = vec2_distance(pos, obstacle.pos);
        let avoid_radius = obstacle.radius + 70.0;
        if dist < avoid_radius && dist > 0.1 {
            let push = vec2_scale(
                vec2_normalize(vec2_sub(pos, obstacle.pos)),
                (avoid_radius - dist) / avoid_radius,
            );
            steer = vec2_add(steer, push);
        }
    }

    if world.is_inside_enemy_zone(team, pos) {
        if let Some(zone) = world.spawn_zones.iter().find(|zone| zone.team == team.enemy()) {
            let zone_center = vec2(
                zone.rect.x + zone.rect.width * 0.5,
                zone.rect.y + zone.rect.height * 0.5,
            );
            steer = vec2_add(steer, vec2_normalize(vec2_sub(pos, zone_center)));
        }
    }

    steer
}
