use raylib::prelude::Vector2;

use crate::config::TANK_RADIUS;
use crate::entities::{Tank, Team, TrackMark};
use crate::math::{vec2_add, vec2_distance, vec2_from_angle, vec2_scale};
use crate::world::World;

use super::super::constants::{TRACK_OFFSET, TRACK_STEP_DISTANCE};

pub(super) fn update_tank_timers(tank: &mut Tank, dt: f32) {
    tank.health_flash = (tank.health_flash - dt).max(0.0);
    tank.invincible_timer = (tank.invincible_timer - dt).max(0.0);
    tank.rapid_timer = (tank.rapid_timer - dt).max(0.0);
}

pub(super) fn try_move_tank(
    tank: &mut Tank,
    world: &World,
    new_pos: Vector2,
    new_tracks: &mut Vec<TrackMark>,
) {
    if position_clear(world, tank.team, new_pos, TANK_RADIUS) {
        move_tank_with_tracks(tank, new_pos, new_tracks);
    }
}

pub(super) fn advance_tank(
    tank: &mut Tank,
    dt: f32,
    speed_factor: f32,
    world: &World,
    new_tracks: &mut Vec<TrackMark>,
) {
    let velocity = vec2_scale(vec2_from_angle(tank.body_angle), tank.speed * speed_factor);
    let new_pos = vec2_add(tank.pos, vec2_scale(velocity, dt));
    try_move_tank(tank, world, new_pos, new_tracks);
}

pub(super) fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(std::f32::consts::PI * 2.0)
}

fn move_tank_with_tracks(tank: &mut Tank, new_pos: Vector2, new_tracks: &mut Vec<TrackMark>) {
    let distance = vec2_distance(tank.pos, new_pos);
    tank.track_distance += distance;
    tank.tread_phase += distance * 0.05;
    tank.pos = new_pos;
    if tank.track_distance > TRACK_STEP_DISTANCE {
        tank.track_distance = 0.0;
        new_tracks.push(track_mark_for_tank(tank));
    }
}

fn track_mark_for_tank(tank: &Tank) -> TrackMark {
    TrackMark {
        pos: vec2_add(
            tank.pos,
            vec2_scale(vec2_from_angle(tank.body_angle + std::f32::consts::PI), TRACK_OFFSET),
        ),
        rotation: tank.body_angle,
        age: 0.0,
    }
}

fn position_clear(world: &World, team: Team, pos: Vector2, radius: f32) -> bool {
    let bounds = world.world_bounds();
    if pos.x - radius < bounds.x
        || pos.y - radius < bounds.y
        || pos.x + radius > bounds.x + bounds.width
        || pos.y + radius > bounds.y + bounds.height
    {
        return false;
    }
    if world.is_inside_enemy_zone(team, pos) {
        return false;
    }
    for obstacle in &world.obstacles {
        if vec2_distance(pos, obstacle.pos) < obstacle.radius + radius {
            return false;
        }
    }
    true
}
