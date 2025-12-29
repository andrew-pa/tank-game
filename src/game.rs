use rand::{rngs::SmallRng, Rng, SeedableRng};
use raylib::prelude::*;

use crate::assets::{bullet_palette, obstacle_texture, tank_palette, Assets, TankPalette};
use crate::config::{
    BODY_ROT_SPEED, BULLET_DAMAGE, BULLET_LIFE, BULLET_RADIUS, BULLET_SPEED, FIRE_COOLDOWN,
    HEALTH_FLASH_TIME, MAX_HEALTH, PLAYER_INTRO_TIME, POWERUP_BASE_SPAWN, POWERUP_DURATION,
    POWERUP_MAX_COUNT, POWERUP_MAX_SPAWN, POWERUP_MIN_SPAWN, RESPAWN_TIME, ROUND_COUNTDOWN,
    ROUND_TIME, TANKS_PER_TEAM, TANK_RADIUS, TANK_SPEED, TILE_SIZE, TRACK_LIFE, TURRET_ROT_SPEED,
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::entities::{Bullet, Explosion, Powerup, PowerupKind, SmokeColor, Tank, Team, TrackMark};
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

const SPRITE_ROT_OFFSET_DEG: f32 = 90.0;
const BARREL_LENGTH: f32 = 46.0;
const TRACK_STEP_DISTANCE: f32 = 40.0;
const TRACK_OFFSET: f32 = 18.0;
const AI_TARGET_FAR: f32 = 260.0;
const AI_TARGET_NEAR: f32 = 180.0;
const AI_FIRE_RANGE: f32 = 900.0;

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
        let tanks = spawn_tanks(&mut rng, &world);
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
        self.tanks = spawn_tanks(&mut self.rng, &self.world);
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

    fn update_tanks(&mut self, dt: f32, rl: &RaylibHandle) {
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

    fn update_powerups(&mut self, dt: f32) {
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
                self.powerups.push(Powerup { kind, pos, age: 0.0 });
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
                if tank.alive
                    && vec2_distance(powerup.pos, tank.pos) < TANK_RADIUS + 22.0
                {
                    match powerup.kind {
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
            if self.world.spawn_zones.iter().any(|zone| zone.contains(pos)) {
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
                draw_texture_centered(
                    &mut d2,
                    texture,
                    track.pos,
                    sprite_rotation(track.rotation),
                    tint,
                );
            }

            for obstacle in &self.world.obstacles {
                let texture = obstacle_texture(assets, obstacle.kind);
                draw_texture_centered(&mut d2, texture, obstacle.pos, 0.0, Color::WHITE);
            }

            for powerup in &self.powerups {
                draw_powerup(&mut d2, assets, powerup);
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
                    sprite_rotation(tank.body_angle),
                    with_alpha(Color::new(160, 160, 160, 255), 0.35),
                );

                draw_texture_centered(
                    &mut d2,
                    &palette.outline_body,
                    vec2(tank.pos.x + 2.0, tank.pos.y + 2.0),
                    sprite_rotation(tank.body_angle),
                    Color::new(0, 0, 0, 90),
                );
                draw_texture_centered(
                    &mut d2,
                    &palette.body,
                    tank.pos,
                    sprite_rotation(tank.body_angle),
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

                draw_tank_health(&mut d2, tank);
                draw_powerup_markers(&mut d2, tank);
            }

            for bullet in &self.bullets {
                let palette = bullet_palette(assets, bullet.team);
                let rotation = sprite_rotation(vec2_angle(bullet.vel));
                draw_texture_centered(&mut d2, &palette.normal, bullet.pos, rotation, Color::WHITE);
            }

            for explosion in &self.explosions {
                let frame = explosion_frame(assets, explosion);
                draw_texture_centered(&mut d2, frame, explosion.pos, 0.0, Color::WHITE);
            }

            if self.intro_timer > 0.0 {
                if let Some(player) = self
                    .tanks
                    .get(self.player_index)
                    .filter(|tank| tank.alive)
                {
                    let pulse = (self.intro_timer * 6.0).sin().abs();
                    let radius = TANK_RADIUS + 10.0 + pulse * 6.0;
                    d2.draw_circle_lines(
                        player.pos.x as i32,
                        player.pos.y as i32,
                        radius,
                        Color::new(255, 230, 120, 220),
                    );
                    let label = "YOU";
                    let size = 18;
                    let width = d2.measure_text(label, size);
                    d2.draw_text(
                        label,
                        (player.pos.x - width as f32 * 0.5) as i32,
                        (player.pos.y - radius - 24.0) as i32,
                        size,
                        Color::new(255, 230, 120, 235),
                    );
                }
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

        if self.countdown_timer > 0.0 {
            self.draw_countdown(d);
        }

        if let Some(player) = self.tanks.get(self.player_index) {
            self.draw_player_health(d, player);
            if !player.alive {
                self.draw_respawn_notice(d, player.respawn_timer);
            }
        }
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

    fn draw_respawn_notice(&self, d: &mut RaylibDrawHandle, timer: f32) {
        let remaining = timer.ceil().max(1.0) as i32;
        d.draw_rectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, Color::new(8, 8, 12, 150));
        let text = format!("Eliminated! Respawning in {remaining}");
        let size = 38;
        let width = d.measure_text(&text, size);
        d.draw_text(
            &text,
            (WINDOW_WIDTH - width) / 2,
            WINDOW_HEIGHT / 2 - 20,
            size,
            Color::new(240, 210, 120, 240),
        );
    }

    fn draw_player_health(&self, d: &mut RaylibDrawHandle, player: &Tank) {
        let bar_width = 260;
        let bar_height = 14;
        let x = 20;
        let y = 58;
        let pct = (player.health / player.max_health).clamp(0.0, 1.0);
        d.draw_rectangle(x, y, bar_width, bar_height, Color::new(10, 10, 10, 200));
        d.draw_rectangle(
            x + 2,
            y + 2,
            ((bar_width - 4) as f32 * pct) as i32,
            bar_height - 4,
            if player.invincible_timer > 0.0 {
                Color::new(120, 210, 255, 230)
            } else {
                player.team.color()
            },
        );
        d.draw_text("Hull", x, y - 18, 16, Color::new(230, 230, 230, 220));
    }

    fn draw_countdown(&self, d: &mut RaylibDrawHandle) {
        let count = self.countdown_timer.ceil().max(1.0) as i32;
        let text = format!("Deploying in {count}");
        let size = 44;
        let width = d.measure_text(&text, size);
        d.draw_text(
            &text,
            (WINDOW_WIDTH - width) / 2,
            WINDOW_HEIGHT / 2 - 80,
            size,
            Color::new(255, 230, 120, 240),
        );
        let hint = "WASD to move • Mouse to aim • LMB to fire";
        let hint_size = 18;
        let hint_width = d.measure_text(hint, hint_size);
        d.draw_text(
            hint,
            (WINDOW_WIDTH - hint_width) / 2,
            WINDOW_HEIGHT / 2 - 32,
            hint_size,
            Color::new(230, 230, 230, 220),
        );
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
        tank.turret_angle = rotate_towards(tank.turret_angle, tank.body_angle, TURRET_ROT_SPEED * dt);
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
        sprite_rotation(0.0),
        Color::new(0, 0, 0, 90),
    );
    draw_texture_centered(d, &palette.body, pos, sprite_rotation(0.0), Color::WHITE);
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
        x: pos.x,
        y: pos.y,
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
    let origin = Vector2 { x: w / 2.0, y: h };
    let dest = Rectangle {
        x: pos.x,
        y: pos.y,
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
        sprite_rotation(angle),
        tint,
    );
}

fn is_start_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
}

fn sprite_rotation(angle: f32) -> f32 {
    rad_to_deg(angle) + SPRITE_ROT_OFFSET_DEG
}

fn draw_tank_health(d: &mut RaylibDrawHandle, tank: &Tank) {
    if tank.health >= tank.max_health || tank.health_flash <= 0.0 {
        return;
    }
    let pct = (tank.health / tank.max_health).clamp(0.0, 1.0);
    let bar_w = 44.0;
    let bar_h = 6.0;
    let x = tank.pos.x - bar_w * 0.5;
    let y = tank.pos.y - TANK_RADIUS - 16.0;
    d.draw_rectangle(x as i32, y as i32, bar_w as i32, bar_h as i32, Color::new(10, 10, 10, 190));
    d.draw_rectangle(
        (x + 1.0) as i32,
        (y + 1.0) as i32,
        ((bar_w - 2.0) * pct) as i32,
        (bar_h - 2.0) as i32,
        tank.team.color(),
    );
}

fn draw_powerup_markers(d: &mut RaylibDrawHandle, tank: &Tank) {
    let mut ring = 0.0;
    if tank.invincible_timer > 0.0 {
        ring += 1.0;
        let pulse = (tank.invincible_timer * 4.0).sin().abs();
        d.draw_circle_lines(
            tank.pos.x as i32,
            tank.pos.y as i32,
            TANK_RADIUS + 8.0 + pulse * 4.0,
            Color::new(120, 210, 255, 220),
        );
    }
    if tank.rapid_timer > 0.0 {
        ring += 1.0;
        let pulse = (tank.rapid_timer * 5.0).sin().abs();
        d.draw_circle_lines(
            tank.pos.x as i32,
            tank.pos.y as i32,
            TANK_RADIUS + 4.0 + ring * 6.0 + pulse * 3.0,
            Color::new(255, 200, 90, 220),
        );
    }
}

fn draw_powerup(d: &mut RaylibDrawHandle, assets: &Assets, powerup: &Powerup) {
    let frames = match powerup.kind {
        PowerupKind::Invincible => &assets.smoke.white,
        PowerupKind::RapidRange => &assets.smoke.orange,
        PowerupKind::Heal => &assets.smoke.yellow,
    };
    let idx = ((powerup.age * 8.0) as usize) % frames.len();
    let pulse = (powerup.age * 6.0).sin().abs();
    let tint = match powerup.kind {
        PowerupKind::Invincible => Color::new(120, 210, 255, 235),
        PowerupKind::RapidRange => Color::new(255, 200, 90, 235),
        PowerupKind::Heal => Color::new(120, 230, 160, 235),
    };
    draw_texture_centered(d, &frames[idx], powerup.pos, 0.0, tint);
    d.draw_circle_lines(
        powerup.pos.x as i32,
        powerup.pos.y as i32,
        26.0 + pulse * 6.0,
        match powerup.kind {
            PowerupKind::Invincible => Color::new(120, 210, 255, 220),
            PowerupKind::RapidRange => Color::new(255, 200, 90, 220),
            PowerupKind::Heal => Color::new(120, 230, 160, 220),
        },
    );
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
