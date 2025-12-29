use rand::{rngs::SmallRng, Rng};
use raylib::prelude::{KeyboardKey, MouseButton, RaylibHandle, Vector2};

use crate::config::{
    BODY_ROT_SPEED, BULLET_LIFE, BULLET_SPEED, FIRE_COOLDOWN, MAX_HEALTH, TANKS_PER_TEAM,
    TANK_RADIUS, TANK_SPEED, TURRET_ROT_SPEED, TILE_SIZE,
};
use crate::entities::{Bullet, Tank, Team, TrackMark};
use crate::math::{
    angle_difference, push_outside_rect, random_angle, rotate_towards, vec2, vec2_add, vec2_angle,
    vec2_distance, vec2_from_angle, vec2_length, vec2_normalize, vec2_scale, vec2_sub,
};
use crate::world::World;

use super::constants::{AI_FIRE_RANGE, AI_TARGET_FAR, AI_TARGET_NEAR, BARREL_LENGTH, TRACK_OFFSET, TRACK_STEP_DISTANCE};
use super::Game;

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

impl Game {
    pub(super) fn update_tanks(&mut self, dt: f32, rl: &RaylibHandle) {
        let snapshot: Vec<(Team, Vector2, bool)> = self
            .tanks
            .iter()
            .map(|tank| (tank.team, tank.pos, tank.alive))
            .collect();
        let mut new_bullets = Vec::new();
        let mut new_tracks = Vec::new();
        let world = &self.world;
        let camera = self.camera();
        let mouse_world = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);

        for (index, tank) in self.tanks.iter_mut().enumerate() {
            update_tank_timers(tank, dt);

            if !tank.alive {
                tank.respawn_timer -= dt;
                if tank.respawn_timer <= 0.0 {
                    respawn_tank(tank, world, &mut self.rng);
                }
                continue;
            }

            tank.fire_cooldown = (tank.fire_cooldown - dt).max(0.0);

            if index == self.player_index {
                update_player_tank(
                    tank,
                    dt,
                    rl,
                    world,
                    mouse_world,
                    &mut new_tracks,
                    &mut new_bullets,
                );
                continue;
            }
            update_ai_tank(
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
        for i in 0..self.tanks.len() {
            for j in (i + 1)..self.tanks.len() {
                if !self.tanks[i].alive || !self.tanks[j].alive {
                    continue;
                }
                let delta = vec2_sub(self.tanks[j].pos, self.tanks[i].pos);
                let dist = vec2_length(delta);
                let min_dist = TANK_RADIUS * 2.0 - 2.0;
                if dist > 0.0 && dist < min_dist {
                    let push = vec2_scale(vec2_normalize(delta), (min_dist - dist) * 0.5);
                    self.tanks[i].pos = vec2_sub(self.tanks[i].pos, push);
                    self.tanks[j].pos = vec2_add(self.tanks[j].pos, push);
                }
            }
        }

        for tank in &mut self.tanks {
            if !tank.alive {
                continue;
            }
            if let Some(zone) = self
                .world
                .spawn_zones
                .iter()
                .find(|zone| zone.team == tank.team.enemy())
            {
                tank.pos = push_outside_rect(tank.pos, zone.rect, TANK_RADIUS + 2.0);
            }
        }
    }
}

fn pick_waypoint(world: &World, team: Team, rng: &mut SmallRng) -> Vector2 {
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

fn update_player_tank(
    tank: &mut Tank,
    dt: f32,
    rl: &RaylibHandle,
    world: &World,
    mouse_world: Vector2,
    new_tracks: &mut Vec<TrackMark>,
    new_bullets: &mut Vec<Bullet>,
) {
    let mut turn: f32 = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
        turn -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        turn += 1.0;
    }
    if turn.abs() > 0.0 {
        tank.body_angle = wrap_angle(tank.body_angle + turn * BODY_ROT_SPEED * dt);
    }

    let mut movement: f32 = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        movement += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        movement -= 1.0;
    }

    if movement.abs() > 0.01 {
        let velocity = vec2_scale(
            vec2_from_angle(tank.body_angle),
            tank.speed * movement * tank_speed_multiplier(tank),
        );
        let new_pos = vec2_add(tank.pos, vec2_scale(velocity, dt));
        try_move_tank(tank, world, new_pos, new_tracks);
    }

    let turret_target = vec2_sub(mouse_world, tank.pos);
    if vec2_length(turret_target) > 0.01 {
        tank.turret_angle = vec2_angle(turret_target);
    }

    let wants_fire = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
        || rl.is_key_down(KeyboardKey::KEY_SPACE);
    if wants_fire && tank.fire_cooldown <= 0.0 {
        fire_bullet(tank, new_bullets);
    }
}

fn update_ai_tank(
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
            && target_dist < AI_FIRE_RANGE * tank_range_multiplier(tank)
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
        let velocity = vec2_scale(
            vec2_from_angle(tank.body_angle),
            tank.speed * tank_speed_multiplier(tank),
        );
        let new_pos = vec2_add(tank.pos, vec2_scale(velocity, dt));
        try_move_tank(tank, world, new_pos, new_tracks);
    }
}

fn update_tank_timers(tank: &mut Tank, dt: f32) {
    tank.health_flash = (tank.health_flash - dt).max(0.0);
    tank.invincible_timer = (tank.invincible_timer - dt).max(0.0);
    tank.rapid_timer = (tank.rapid_timer - dt).max(0.0);
}

fn respawn_tank(tank: &mut Tank, world: &World, rng: &mut SmallRng) {
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

fn try_move_tank(
    tank: &mut Tank,
    world: &World,
    new_pos: Vector2,
    new_tracks: &mut Vec<TrackMark>,
) {
    if position_clear(world, tank.team, new_pos, TANK_RADIUS) {
        move_tank_with_tracks(tank, new_pos, new_tracks);
    }
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

fn fire_bullet(tank: &mut Tank, new_bullets: &mut Vec<Bullet>) {
    new_bullets.push(bullet_from_tank(tank));
    tank.fire_cooldown = FIRE_COOLDOWN / tank_fire_rate_multiplier(tank);
}

fn bullet_from_tank(tank: &Tank) -> Bullet {
    let dir = vec2_from_angle(tank.turret_angle);
    Bullet {
        pos: vec2_add(tank.pos, vec2_scale(dir, BARREL_LENGTH)),
        vel: vec2_scale(dir, BULLET_SPEED),
        team: tank.team,
        life: BULLET_LIFE * tank_range_multiplier(tank),
    }
}

fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(std::f32::consts::PI * 2.0)
}

fn tank_speed_multiplier(tank: &Tank) -> f32 {
    if tank.invincible_timer > 0.0 {
        1.15
    } else {
        1.0
    }
}

fn tank_fire_rate_multiplier(tank: &Tank) -> f32 {
    if tank.rapid_timer > 0.0 {
        1.2
    } else {
        1.0
    }
}

fn tank_range_multiplier(tank: &Tank) -> f32 {
    if tank.rapid_timer > 0.0 {
        2.0
    } else {
        1.0
    }
}
