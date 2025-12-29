use rand::{rngs::SmallRng, Rng, SeedableRng};
use raylib::prelude::*;

use crate::assets::{bullet_palette, obstacle_texture, tank_palette, Assets, TankPalette};
use crate::config::{
    BODY_ROT_SPEED, BULLET_RADIUS, BULLET_SPEED, FIRE_COOLDOWN, RESPAWN_TIME, ROUND_TIME,
    TANKS_PER_TEAM, TANK_RADIUS, TANK_SPEED, TILE_SIZE, TRACK_LIFE, TURRET_ROT_SPEED,
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::entities::{Bullet, Explosion, SmokeColor, Tank, Team, TrackMark};
use crate::math::{
    angle_difference, point_in_bounds, push_outside_rect, rad_to_deg, random_angle, rotate_towards,
    vec2, vec2_add, vec2_angle, vec2_distance, vec2_from_angle, vec2_length, vec2_normalize,
    vec2_scale, vec2_sub, with_alpha,
};
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
    rng: SmallRng,
    round_timer: f32,
    team_kills: [u32; 2],
    last_winner: Option<Team>,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);
        let world = World::new(&mut rng);
        let tanks = spawn_tanks(&mut rng, &world);
        let mut game = Self {
            state: ScreenState::Title,
            world,
            tanks,
            bullets: Vec::new(),
            tracks: Vec::new(),
            explosions: Vec::new(),
            rng,
            round_timer: ROUND_TIME,
            team_kills: [0, 0],
            last_winner: None,
        };
        game.reset_round();
        game.state = ScreenState::Title;
        game
    }

    fn reset_round(&mut self) {
        self.world = World::new(&mut self.rng);
        self.tanks = spawn_tanks(&mut self.rng, &self.world);
        self.bullets.clear();
        self.tracks.clear();
        self.explosions.clear();
        self.round_timer = ROUND_TIME;
        self.team_kills = [0, 0];
        self.last_winner = None;
    }

    pub fn update(&mut self, dt: f32, rl: &RaylibHandle) {
        match self.state {
            ScreenState::Title => {
                if is_start_pressed(rl) {
                    self.reset_round();
                    self.state = ScreenState::Playing;
                }
            }
            ScreenState::Playing => {
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
                self.update_tanks(dt);
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

    fn update_tanks(&mut self, dt: f32) {
        let snapshot: Vec<(Team, Vector2, bool)> = self
            .tanks
            .iter()
            .map(|tank| (tank.team, tank.pos, tank.alive))
            .collect();
        let mut new_bullets = Vec::new();
        let mut new_tracks = Vec::new();
        let world = &self.world;

        for tank in &mut self.tanks {
            if !tank.alive {
                tank.respawn_timer -= dt;
                if tank.respawn_timer <= 0.0 {
                    tank.alive = true;
                    tank.pos = world.random_point_in_zone(tank.team, &mut self.rng);
                    tank.body_angle = random_angle(&mut self.rng);
                    tank.turret_angle = tank.body_angle;
                    tank.fire_cooldown = FIRE_COOLDOWN * 0.5;
                }
                continue;
            }

            tank.fire_cooldown = (tank.fire_cooldown - dt).max(0.0);
            tank.tread_phase += dt * tank.speed * 0.02;

            let (target_pos, target_dist) = find_target_snapshot(&snapshot, tank.team, tank.pos);
            let desired_dir = if let Some(target) = target_pos {
                if target_dist > 260.0 {
                    vec2_normalize(vec2_sub(target, tank.pos))
                } else if target_dist < 180.0 {
                    vec2_normalize(vec2_sub(tank.pos, target))
                } else {
                    let to_target = vec2_sub(target, tank.pos);
                    vec2_normalize(vec2(-to_target.y, to_target.x))
                }
            } else {
                let to_waypoint = vec2_sub(tank.waypoint, tank.pos);
                if vec2_length(to_waypoint) < 80.0 {
                    tank.waypoint = pick_waypoint(world, tank.team, &mut self.rng);
                }
                vec2_normalize(to_waypoint)
            };

            if let Some(target) = target_pos {
                let target_angle = vec2_angle(vec2_sub(target, tank.pos));
                tank.turret_angle =
                    rotate_towards(tank.turret_angle, target_angle, TURRET_ROT_SPEED * dt);

                if angle_difference(tank.turret_angle, target_angle) < 0.22
                    && tank.fire_cooldown <= 0.0
                    && target_dist < 900.0
                {
                    let barrel_len = 46.0;
                    let dir = vec2_from_angle(tank.turret_angle);
                    let pos = vec2_add(tank.pos, vec2_scale(dir, barrel_len));
                    let vel = vec2_scale(dir, BULLET_SPEED);
                    new_bullets.push(Bullet {
                        pos,
                        vel,
                        team: tank.team,
                        life: 2.2,
                    });
                    tank.fire_cooldown = FIRE_COOLDOWN;
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
                let velocity = vec2_scale(vec2_from_angle(tank.body_angle), tank.speed);
                let new_pos = vec2_add(tank.pos, vec2_scale(velocity, dt));
                if position_clear(world, tank.team, new_pos, TANK_RADIUS) {
                    let distance = vec2_distance(tank.pos, new_pos);
                    tank.track_distance += distance;
                    tank.pos = new_pos;
                    if tank.track_distance > 40.0 {
                        tank.track_distance = 0.0;
                        new_tracks.push(TrackMark {
                            pos: vec2_add(
                                tank.pos,
                                vec2_scale(vec2_from_angle(tank.body_angle + std::f32::consts::PI), 18.0),
                            ),
                            rotation: tank.body_angle,
                            age: 0.0,
                        });
                    }
                }
            }
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

    pub fn draw(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        d.clear_background(Color::new(32, 96, 160, 255));
        match self.state {
            ScreenState::Title => self.draw_title(d, assets),
            ScreenState::Playing | ScreenState::RoundOver => {
                self.draw_world(d, assets);
                self.draw_hud(d);
                if self.state == ScreenState::RoundOver {
                    self.draw_round_over(d);
                }
            }
        }
    }

    fn draw_title(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        d.clear_background(Color::new(30, 85, 140, 255));
        let title = "TANKS: DOMINION";
        let title_size = 56;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            (WINDOW_WIDTH - title_width) / 2,
            40,
            title_size,
            Color::new(245, 245, 245, 255),
        );

        let subtitle = "Two squads. Procedural frontier. Score the most eliminations.";
        let sub_size = 20;
        let sub_width = d.measure_text(subtitle, sub_size);
        d.draw_text(
            subtitle,
            (WINDOW_WIDTH - sub_width) / 2,
            110,
            sub_size,
            Color::new(220, 220, 220, 255),
        );

        let info = "Press ENTER to deploy";
        let info_size = 28;
        let info_width = d.measure_text(info, info_size);
        d.draw_text(
            info,
            (WINDOW_WIDTH - info_width) / 2,
            150,
            info_size,
            Color::new(240, 200, 110, 255),
        );

        let mut x = 120.0;
        let y = 240.0;
        let tank_line = [
            &assets.tanks.red,
            &assets.tanks.blue,
            &assets.tanks.green,
            &assets.tanks.beige,
            &assets.tanks.black,
        ];
        for palette in tank_line {
            draw_tank_preview(d, palette, vec2(x, y));
            x += 140.0;
        }

        let bullet_y = 350.0;
        let bullets = [
            &assets.bullets.red,
            &assets.bullets.blue,
            &assets.bullets.green,
            &assets.bullets.beige,
            &assets.bullets.yellow,
            &assets.bullets.silver,
        ];
        let mut bx = 120.0;
        for bullet in bullets {
            draw_texture_centered(d, &bullet.normal, vec2(bx, bullet_y), 0.0, Color::WHITE);
            draw_texture_centered(
                d,
                &bullet.silver,
                vec2(bx, bullet_y + 40.0),
                0.0,
                Color::WHITE,
            );
            draw_texture_centered(
                d,
                &bullet.outline,
                vec2(bx, bullet_y + 80.0),
                0.0,
                Color::WHITE,
            );
            draw_texture_centered(
                d,
                &bullet.silver_outline,
                vec2(bx, bullet_y + 120.0),
                0.0,
                Color::WHITE,
            );
            bx += 100.0;
        }

        let obstacles = [
            &assets.obstacles.tree_small,
            &assets.obstacles.tree_large,
            &assets.obstacles.sandbag_brown,
            &assets.obstacles.sandbag_beige,
            &assets.obstacles.oil,
            &assets.obstacles.barrel_red_up,
            &assets.obstacles.barrel_red_side,
            &assets.obstacles.barrel_grey_up,
            &assets.obstacles.barrel_grey_side,
            &assets.obstacles.barrel_grey_rust,
            &assets.obstacles.barrel_green_up,
            &assets.obstacles.barrel_green_side,
            &assets.obstacles.barrel_green_side_damaged,
        ];
        let mut ox = 80.0;
        let mut oy = 520.0;
        for texture in obstacles {
            draw_texture_centered(d, texture, vec2(ox, oy), 0.0, Color::WHITE);
            ox += 90.0;
            if ox > WINDOW_WIDTH as f32 - 90.0 {
                ox = 80.0;
                oy += 80.0;
            }
        }

        let mut smoke_x = WINDOW_WIDTH as f32 - 220.0;
        let mut smoke_y = 250.0;
        let smoke_frames = assets
            .smoke
            .orange
            .iter()
            .chain(assets.smoke.yellow.iter())
            .chain(assets.smoke.grey.iter())
            .chain(assets.smoke.white.iter());
        for frame in smoke_frames {
            draw_texture_centered(d, frame, vec2(smoke_x, smoke_y), 0.0, Color::WHITE);
            smoke_y += 50.0;
            if smoke_y > WINDOW_HEIGHT as f32 - 80.0 {
                smoke_y = 250.0;
                smoke_x += 80.0;
            }
        }
    }

    fn draw_world(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        let camera = self.camera();
        d.draw_mode2D(camera, |mut d2, _| {
            for y in 0..self.world.height {
                for x in 0..self.world.width {
                    let texture = match self.world.tile_kind(x, y) {
                        crate::world::TileKind::Grass => &assets.tiles.grass,
                        crate::world::TileKind::Dirt => &assets.tiles.dirt,
                        crate::world::TileKind::Sand => &assets.tiles.sand,
                    };
                    let pos = vec2(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                    d2.draw_texture_ex(texture, pos, 0.0, 1.0, Color::WHITE);
                }
            }

            for zone in &self.world.spawn_zones {
                let tint = with_alpha(zone.team.color(), 0.18);
                d2.draw_rectangle_rec(zone.rect, tint);
            }

            for track in &self.tracks {
                let alpha = (1.0 - track.age / TRACK_LIFE).max(0.0);
                let tint = with_alpha(Color::new(200, 200, 200, 255), alpha * 0.7);
                let texture = if (track.age * 10.0) as i32 % 2 == 0 {
                    &assets.tracks_large
                } else {
                    &assets.tracks_small
                };
                draw_texture_centered(&mut d2, texture, track.pos, rad_to_deg(track.rotation), tint);
            }

            for obstacle in &self.world.obstacles {
                let texture = obstacle_texture(assets, obstacle.kind);
                draw_texture_centered(&mut d2, texture, obstacle.pos, 0.0, Color::WHITE);
            }

            for tank in &self.tanks {
                if !tank.alive {
                    continue;
                }
                let palette = tank_palette(assets, tank.team);
                let tread_texture = if (tank.tread_phase as i32) % 2 == 0 {
                    &assets.tracks_large
                } else {
                    &assets.tracks_small
                };
                draw_texture_centered(
                    &mut d2,
                    tread_texture,
                    tank.pos,
                    rad_to_deg(tank.body_angle),
                    with_alpha(Color::new(160, 160, 160, 255), 0.35),
                );

                draw_texture_centered(
                    &mut d2,
                    &palette.outline_body,
                    vec2(tank.pos.x + 2.0, tank.pos.y + 2.0),
                    rad_to_deg(tank.body_angle),
                    Color::new(0, 0, 0, 90),
                );
                draw_texture_centered(
                    &mut d2,
                    &palette.body,
                    tank.pos,
                    rad_to_deg(tank.body_angle),
                    Color::WHITE,
                );
                draw_barrel(
                    &mut d2,
                    &palette.outline_barrel,
                    tank.pos,
                    tank.turret_angle,
                    Color::new(0, 0, 0, 90),
                );
                draw_barrel(
                    &mut d2,
                    &palette.barrel,
                    tank.pos,
                    tank.turret_angle,
                    Color::WHITE,
                );
            }

            for bullet in &self.bullets {
                let palette = bullet_palette(assets, bullet.team);
                let rotation = rad_to_deg(vec2_angle(bullet.vel));
                draw_texture_centered(&mut d2, &palette.normal, bullet.pos, rotation, Color::WHITE);
            }

            for explosion in &self.explosions {
                let frame = explosion_frame(assets, explosion);
                draw_texture_centered(&mut d2, frame, explosion.pos, 0.0, Color::WHITE);
            }
        });
    }

    fn draw_hud(&self, d: &mut RaylibDrawHandle) {
        let bar_height = 48;
        d.draw_rectangle(0, 0, WINDOW_WIDTH, bar_height, Color::new(20, 24, 28, 220));
        let red_label = format!("{}: {}", Team::Red.name(), self.team_kills[0]);
        let blue_label = format!("{}: {}", Team::Blue.name(), self.team_kills[1]);
        d.draw_text(&red_label, 20, 12, 20, Team::Red.color());
        let blue_width = d.measure_text(&blue_label, 20);
        d.draw_text(
            &blue_label,
            WINDOW_WIDTH - blue_width - 20,
            12,
            20,
            Team::Blue.color(),
        );
        let time_label = format!("Time: {:>3.0}", self.round_timer.ceil());
        let time_width = d.measure_text(&time_label, 20);
        d.draw_text(
            &time_label,
            (WINDOW_WIDTH - time_width) / 2,
            12,
            20,
            Color::new(240, 240, 240, 255),
        );
    }

    fn draw_round_over(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, Color::new(10, 10, 10, 160));
        let message = match self.last_winner {
            Some(team) => format!("{} wins the round!", team.name()),
            None => "Stalemate!".to_string(),
        };
        let size = 46;
        let width = d.measure_text(&message, size);
        d.draw_text(
            &message,
            (WINDOW_WIDTH - width) / 2,
            WINDOW_HEIGHT / 2 - 40,
            size,
            Color::new(240, 240, 240, 255),
        );
        let prompt = "Press ENTER to redeploy";
        let prompt_size = 24;
        let prompt_width = d.measure_text(prompt, prompt_size);
        d.draw_text(
            prompt,
            (WINDOW_WIDTH - prompt_width) / 2,
            WINDOW_HEIGHT / 2 + 20,
            prompt_size,
            Color::new(220, 200, 120, 255),
        );
    }

    fn camera(&self) -> Camera2D {
        let mut center = vec2(0.0, 0.0);
        let mut count = 0.0;
        for tank in &self.tanks {
            if tank.alive {
                center = vec2_add(center, tank.pos);
                count += 1.0;
            }
        }
        if count > 0.0 {
            center = vec2_scale(center, 1.0 / count);
        } else {
            center = vec2(
                self.world.width as f32 * TILE_SIZE * 0.5,
                self.world.height as f32 * TILE_SIZE * 0.5,
            );
        }

        Camera2D {
            target: center,
            offset: vec2(WINDOW_WIDTH as f32 * 0.5, WINDOW_HEIGHT as f32 * 0.55),
            rotation: 0.0,
            zoom: 0.55,
        }
    }
}

fn spawn_tanks(rng: &mut SmallRng, world: &World) -> Vec<Tank> {
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
            });
        }
    }
    tanks
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

fn explosion_frame<'a>(assets: &'a Assets, explosion: &Explosion) -> &'a Texture2D {
    let frame = (explosion.age / 0.08).floor() as usize;
    let idx = frame.min(5);
    match explosion.color {
        SmokeColor::Orange => &assets.smoke.orange[idx],
        SmokeColor::Yellow => &assets.smoke.yellow[idx],
        SmokeColor::Grey => &assets.smoke.grey[idx],
        SmokeColor::White => &assets.smoke.white[idx],
    }
}

fn draw_tank_preview(d: &mut RaylibDrawHandle, palette: &TankPalette, pos: Vector2) {
    draw_texture_centered(
        d,
        &palette.outline_body,
        vec2(pos.x + 2.0, pos.y + 2.0),
        0.0,
        Color::new(0, 0, 0, 90),
    );
    draw_texture_centered(d, &palette.body, pos, 0.0, Color::WHITE);
    draw_barrel(
        d,
        &palette.outline_barrel,
        pos,
        0.3,
        Color::new(0, 0, 0, 90),
    );
    draw_barrel(d, &palette.barrel, pos, 0.3, Color::WHITE);
}

fn draw_texture_centered(
    d: &mut RaylibDrawHandle,
    texture: &Texture2D,
    pos: Vector2,
    rotation: f32,
    tint: Color,
) {
    let w = texture.width as f32;
    let h = texture.height as f32;
    let dest = Rectangle {
        x: pos.x - w / 2.0,
        y: pos.y - h / 2.0,
        width: w,
        height: h,
    };
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: w,
        height: h,
    };
    d.draw_texture_pro(
        texture,
        src,
        dest,
        Vector2 { x: w / 2.0, y: h / 2.0 },
        rotation,
        tint,
    );
}

fn draw_barrel(
    d: &mut RaylibDrawHandle,
    texture: &Texture2D,
    pos: Vector2,
    angle: f32,
    tint: Color,
) {
    let w = texture.width as f32;
    let h = texture.height as f32;
    let origin = Vector2 { x: w / 2.0, y: h * 0.75 };
    let dest = Rectangle {
        x: pos.x - origin.x,
        y: pos.y - origin.y,
        width: w,
        height: h,
    };
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: w,
        height: h,
    };
    d.draw_texture_pro(
        texture,
        src,
        dest,
        origin,
        rad_to_deg(angle),
        tint,
    );
}

fn is_start_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
}
