use rand::{rngs::SmallRng, Rng, SeedableRng};
use raylib::prelude::*;
use std::f32::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const TILE_SIZE: f32 = 128.0;
const MAP_WIDTH: i32 = 50;
const MAP_HEIGHT: i32 = 30;
const ROUND_TIME: f32 = 120.0;
const TANKS_PER_TEAM: usize = 4;
const TANK_RADIUS: f32 = 28.0;
const BULLET_RADIUS: f32 = 6.0;
const BULLET_SPEED: f32 = 520.0;
const TANK_SPEED: f32 = 130.0;
const BODY_ROT_SPEED: f32 = 2.6;
const TURRET_ROT_SPEED: f32 = 3.4;
const FIRE_COOLDOWN: f32 = 1.1;
const RESPAWN_TIME: f32 = 3.0;
const TRACK_LIFE: f32 = 8.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug_frame = args.iter().any(|arg| arg == "--render-frame");
    let seed_override = parse_seed(&args);

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tanks: Dominion")
        .resizable()
        .build();

    rl.set_target_fps(60);

    let seed = seed_override.unwrap_or_else(system_seed);
    let assets = Assets::load(&mut rl, &thread);
    let mut game = Game::new(seed, &assets);

    if debug_frame {
        game.update(1.0 / 60.0, &assets, &rl);
        {
            let mut d = rl.begin_drawing(&thread);
            game.draw(&mut d, &assets);
        }
        rl.take_screenshot(&thread, "debug_frame.png");
        return;
    }

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        game.update(dt, &assets, &rl);
        let mut d = rl.begin_drawing(&thread);
        game.draw(&mut d, &assets);
    }
}

fn parse_seed(args: &[String]) -> Option<u64> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--seed" {
            if let Some(value) = iter.next() {
                if let Ok(parsed) = value.parse::<u64>() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

fn system_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreenState {
    Title,
    Playing,
    RoundOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Team {
    Red,
    Blue,
}

impl Team {
    fn index(self) -> usize {
        match self {
            Team::Red => 0,
            Team::Blue => 1,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Team::Red => "Crimson",
            Team::Blue => "Azure",
        }
    }

    fn color(self) -> Color {
        match self {
            Team::Red => Color::new(224, 70, 70, 255),
            Team::Blue => Color::new(70, 140, 232, 255),
        }
    }

    fn enemy(self) -> Team {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TileKind {
    Grass,
    Dirt,
    Sand,
}

#[derive(Clone, Copy, Debug)]
struct SpawnZone {
    rect: Rectangle,
    team: Team,
}

impl SpawnZone {
    fn contains(&self, pos: Vector2) -> bool {
        pos.x >= self.rect.x
            && pos.x <= self.rect.x + self.rect.width
            && pos.y >= self.rect.y
            && pos.y <= self.rect.y + self.rect.height
    }
}

#[derive(Clone, Copy, Debug)]
enum ObstacleKind {
    TreeSmall,
    TreeLarge,
    SandbagBrown,
    SandbagBeige,
    Oil,
    BarrelRedUp,
    BarrelRedSide,
    BarrelGreyUp,
    BarrelGreySide,
    BarrelGreyRust,
    BarrelGreenUp,
    BarrelGreenSide,
    BarrelGreenSideDamaged,
}

#[derive(Clone, Debug)]
struct Obstacle {
    kind: ObstacleKind,
    pos: Vector2,
    radius: f32,
}

#[derive(Clone, Copy, Debug)]
enum SmokeColor {
    Orange,
    Yellow,
    Grey,
    White,
}

struct World {
    width: i32,
    height: i32,
    tiles: Vec<TileKind>,
    obstacles: Vec<Obstacle>,
    spawn_zones: [SpawnZone; 2],
}

impl World {
    fn new(rng: &mut SmallRng) -> Self {
        let width = MAP_WIDTH;
        let height = MAP_HEIGHT;
        let mut values = vec![0.0f32; (width * height) as usize];
        for value in &mut values {
            *value = rng.gen_range(0.0..1.0);
        }

        for _ in 0..3 {
            let mut next = values.clone();
            for y in 0..height {
                for x in 0..width {
                    let mut sum = 0.0;
                    let mut count = 0.0;
                    for ny in (y - 1).max(0)..=(y + 1).min(height - 1) {
                        for nx in (x - 1).max(0)..=(x + 1).min(width - 1) {
                            let idx = (ny * width + nx) as usize;
                            sum += values[idx];
                            count += 1.0;
                        }
                    }
                    let idx = (y * width + x) as usize;
                    next[idx] = sum / count;
                }
            }
            values = next;
        }

        let mut tiles = vec![TileKind::Grass; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let v = values[idx];
                tiles[idx] = if v < 0.35 {
                    TileKind::Sand
                } else if v < 0.62 {
                    TileKind::Dirt
                } else {
                    TileKind::Grass
                };
            }
        }

        let zone_w = 7;
        let zone_h = 8;
        let zone_y = (height - zone_h) / 2;
        let left_zone = SpawnZone {
            rect: Rectangle {
                x: 2.0 * TILE_SIZE,
                y: zone_y as f32 * TILE_SIZE,
                width: zone_w as f32 * TILE_SIZE,
                height: zone_h as f32 * TILE_SIZE,
            },
            team: Team::Red,
        };
        let right_zone = SpawnZone {
            rect: Rectangle {
                x: (width - zone_w - 2) as f32 * TILE_SIZE,
                y: zone_y as f32 * TILE_SIZE,
                width: zone_w as f32 * TILE_SIZE,
                height: zone_h as f32 * TILE_SIZE,
            },
            team: Team::Blue,
        };

        for y in zone_y..zone_y + zone_h {
            for x in 2..2 + zone_w {
                let idx = (y * width + x) as usize;
                tiles[idx] = TileKind::Grass;
            }
            for x in width - zone_w - 2..width - 2 {
                let idx = (y * width + x) as usize;
                tiles[idx] = TileKind::Sand;
            }
        }

        let mut world = Self {
            width,
            height,
            tiles,
            obstacles: Vec::new(),
            spawn_zones: [left_zone, right_zone],
        };
        world.generate_obstacles(rng);
        world
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    fn tile_kind(&self, x: i32, y: i32) -> TileKind {
        self.tiles[self.index(x, y)]
    }

    fn world_bounds(&self) -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.width as f32 * TILE_SIZE,
            height: self.height as f32 * TILE_SIZE,
        }
    }

    fn random_point_in_zone(&self, team: Team, rng: &mut SmallRng) -> Vector2 {
        let zone = self
            .spawn_zones
            .iter()
            .find(|zone| zone.team == team)
            .unwrap();
        let margin = TILE_SIZE * 0.4;
        let x = rng.gen_range(zone.rect.x + margin..zone.rect.x + zone.rect.width - margin);
        let y = rng.gen_range(zone.rect.y + margin..zone.rect.y + zone.rect.height - margin);
        vec2(x, y)
    }

    fn is_inside_enemy_zone(&self, team: Team, pos: Vector2) -> bool {
        self.spawn_zones
            .iter()
            .any(|zone| zone.team != team && zone.contains(pos))
    }

    fn generate_obstacles(&mut self, rng: &mut SmallRng) {
        let all_kinds = [
            ObstacleKind::TreeSmall,
            ObstacleKind::TreeLarge,
            ObstacleKind::SandbagBrown,
            ObstacleKind::SandbagBeige,
            ObstacleKind::Oil,
            ObstacleKind::BarrelRedUp,
            ObstacleKind::BarrelRedSide,
            ObstacleKind::BarrelGreyUp,
            ObstacleKind::BarrelGreySide,
            ObstacleKind::BarrelGreyRust,
            ObstacleKind::BarrelGreenUp,
            ObstacleKind::BarrelGreenSide,
            ObstacleKind::BarrelGreenSideDamaged,
        ];

        for kind in all_kinds {
            if let Some(pos) = self.find_open_obstacle_position(kind, rng, 120) {
                self.obstacles.push(Obstacle {
                    kind,
                    pos,
                    radius: obstacle_radius(kind),
                });
            }
        }

        let target = ((self.width * self.height) as f32 * 0.08) as usize;
        let mut attempts = 0;
        while self.obstacles.len() < target && attempts < target * 8 {
            attempts += 1;
            let kind = all_kinds[rng.gen_range(0..all_kinds.len())];
            if let Some(pos) = self.find_open_obstacle_position(kind, rng, 40) {
                self.obstacles.push(Obstacle {
                    kind,
                    pos,
                    radius: obstacle_radius(kind),
                });
            }
        }
    }

    fn find_open_obstacle_position(
        &self,
        kind: ObstacleKind,
        rng: &mut SmallRng,
        attempts: usize,
    ) -> Option<Vector2> {
        let bounds = self.world_bounds();
        let radius = obstacle_radius(kind);
        for _ in 0..attempts {
            let x = rng.gen_range(bounds.x + radius..bounds.x + bounds.width - radius);
            let y = rng.gen_range(bounds.y + radius..bounds.y + bounds.height - radius);
            let pos = vec2(x, y);
            if self.is_inside_enemy_zone(Team::Red, pos) || self.is_inside_enemy_zone(Team::Blue, pos) {
                continue;
            }
            if self
                .obstacles
                .iter()
                .any(|obs| vec2_distance(obs.pos, pos) < obs.radius + radius + 16.0)
            {
                continue;
            }
            return Some(pos);
        }
        None
    }
}

#[derive(Clone)]
struct TankPalette {
    body: Texture2D,
    barrel: Texture2D,
    outline_body: Texture2D,
    outline_barrel: Texture2D,
}

#[derive(Clone)]
struct BulletPalette {
    normal: Texture2D,
    silver: Texture2D,
    outline: Texture2D,
    silver_outline: Texture2D,
}

struct TileTextures {
    grass: Texture2D,
    dirt: Texture2D,
    sand: Texture2D,
}

struct TankTextures {
    red: TankPalette,
    blue: TankPalette,
    green: TankPalette,
    beige: TankPalette,
    black: TankPalette,
}

struct BulletTextures {
    red: BulletPalette,
    blue: BulletPalette,
    green: BulletPalette,
    beige: BulletPalette,
    yellow: BulletPalette,
    silver: BulletPalette,
}

struct ObstacleTextures {
    tree_small: Texture2D,
    tree_large: Texture2D,
    sandbag_brown: Texture2D,
    sandbag_beige: Texture2D,
    oil: Texture2D,
    barrel_red_up: Texture2D,
    barrel_red_side: Texture2D,
    barrel_grey_up: Texture2D,
    barrel_grey_side: Texture2D,
    barrel_grey_rust: Texture2D,
    barrel_green_up: Texture2D,
    barrel_green_side: Texture2D,
    barrel_green_side_damaged: Texture2D,
}

struct SmokeTextures {
    orange: Vec<Texture2D>,
    yellow: Vec<Texture2D>,
    grey: Vec<Texture2D>,
    white: Vec<Texture2D>,
}

struct Assets {
    tiles: TileTextures,
    tanks: TankTextures,
    bullets: BulletTextures,
    obstacles: ObstacleTextures,
    smoke: SmokeTextures,
    tracks_large: Texture2D,
    tracks_small: Texture2D,
}

impl Assets {
    fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let tiles = TileTextures {
            grass: rl
                .load_texture(thread, "assets/PNG/Environment/grass.png")
                .expect("grass"),
            dirt: rl
                .load_texture(thread, "assets/PNG/Environment/dirt.png")
                .expect("dirt"),
            sand: rl
                .load_texture(thread, "assets/PNG/Environment/sand.png")
                .expect("sand"),
        };

        let tanks = TankTextures {
            red: load_tank_palette(rl, thread, "Red"),
            blue: load_tank_palette(rl, thread, "Blue"),
            green: load_tank_palette(rl, thread, "Green"),
            beige: load_tank_palette(rl, thread, "Beige"),
            black: load_tank_palette(rl, thread, "Black"),
        };

        let bullets = BulletTextures {
            red: load_bullet_palette(rl, thread, "Red"),
            blue: load_bullet_palette(rl, thread, "Blue"),
            green: load_bullet_palette(rl, thread, "Green"),
            beige: load_bullet_palette(rl, thread, "Beige"),
            yellow: load_bullet_palette(rl, thread, "Yellow"),
            silver: load_bullet_palette(rl, thread, "Silver"),
        };

        let obstacles = ObstacleTextures {
            tree_small: rl
                .load_texture(thread, "assets/PNG/Environment/treeSmall.png")
                .expect("tree small"),
            tree_large: rl
                .load_texture(thread, "assets/PNG/Environment/treeLarge.png")
                .expect("tree large"),
            sandbag_brown: rl
                .load_texture(thread, "assets/PNG/Obstacles/sandbagBrown.png")
                .expect("sandbag brown"),
            sandbag_beige: rl
                .load_texture(thread, "assets/PNG/Obstacles/sandbagBeige.png")
                .expect("sandbag beige"),
            oil: rl
                .load_texture(thread, "assets/PNG/Obstacles/oil.png")
                .expect("oil"),
            barrel_red_up: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelRed_up.png")
                .expect("barrel red up"),
            barrel_red_side: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelRed_side.png")
                .expect("barrel red side"),
            barrel_grey_up: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelGrey_up.png")
                .expect("barrel grey up"),
            barrel_grey_side: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelGrey_side.png")
                .expect("barrel grey side"),
            barrel_grey_rust: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelGrey_sde_rust.png")
                .expect("barrel grey rust"),
            barrel_green_up: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelGreen_up.png")
                .expect("barrel green up"),
            barrel_green_side: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelGreen_side.png")
                .expect("barrel green side"),
            barrel_green_side_damaged: rl
                .load_texture(thread, "assets/PNG/Obstacles/barrelGreen_side_damaged.png")
                .expect("barrel green side damaged"),
        };

        let smoke = SmokeTextures {
            orange: load_smoke_frames(rl, thread, "Orange"),
            yellow: load_smoke_frames(rl, thread, "Yellow"),
            grey: load_smoke_frames(rl, thread, "Grey"),
            white: load_smoke_frames(rl, thread, "White"),
        };

        let tracks_large = rl
            .load_texture(thread, "assets/PNG/Tanks/tracksLarge.png")
            .expect("tracks large");
        let tracks_small = rl
            .load_texture(thread, "assets/PNG/Tanks/tracksSmall.png")
            .expect("tracks small");

        Self {
            tiles,
            tanks,
            bullets,
            obstacles,
            smoke,
            tracks_large,
            tracks_small,
        }
    }
}

fn load_tank_palette(rl: &mut RaylibHandle, thread: &RaylibThread, color: &str) -> TankPalette {
    TankPalette {
        body: rl
            .load_texture(thread, &format!("assets/PNG/Tanks/tank{color}.png"))
            .expect("tank body"),
        barrel: rl
            .load_texture(thread, &format!("assets/PNG/Tanks/barrel{color}.png"))
            .expect("tank barrel"),
        outline_body: rl
            .load_texture(thread, &format!(
                "assets/PNG/Tanks/tank{color}_outline.png"
            ))
            .expect("tank outline"),
        outline_barrel: rl
            .load_texture(thread, &format!(
                "assets/PNG/Tanks/barrel{color}_outline.png"
            ))
            .expect("barrel outline"),
    }
}

fn load_bullet_palette(rl: &mut RaylibHandle, thread: &RaylibThread, color: &str) -> BulletPalette {
    BulletPalette {
        normal: rl
            .load_texture(thread, &format!("assets/PNG/Bullets/bullet{color}.png"))
            .expect("bullet"),
        silver: rl
            .load_texture(thread, &format!(
                "assets/PNG/Bullets/bullet{color}Silver.png"
            ))
            .expect("bullet silver"),
        outline: rl
            .load_texture(thread, &format!(
                "assets/PNG/Bullets/bullet{color}_outline.png"
            ))
            .expect("bullet outline"),
        silver_outline: rl
            .load_texture(thread, &format!(
                "assets/PNG/Bullets/bullet{color}Silver_outline.png"
            ))
            .expect("bullet silver outline"),
    }
}

fn load_smoke_frames(rl: &mut RaylibHandle, thread: &RaylibThread, color: &str) -> Vec<Texture2D> {
    (0..6)
        .map(|idx| {
            rl.load_texture(
                thread,
                &format!("assets/PNG/Smoke/smoke{color}{idx}.png"),
            )
            .expect("smoke")
        })
        .collect()
}

#[derive(Clone, Debug)]
struct Tank {
    id: usize,
    team: Team,
    pos: Vector2,
    body_angle: f32,
    turret_angle: f32,
    speed: f32,
    fire_cooldown: f32,
    alive: bool,
    respawn_timer: f32,
    waypoint: Vector2,
    track_distance: f32,
    tread_phase: f32,
}

#[derive(Clone, Debug)]
struct Bullet {
    pos: Vector2,
    vel: Vector2,
    team: Team,
    life: f32,
}

#[derive(Clone, Debug)]
struct TrackMark {
    pos: Vector2,
    rotation: f32,
    age: f32,
}

#[derive(Clone, Debug)]
struct Explosion {
    pos: Vector2,
    color: SmokeColor,
    age: f32,
}

struct Game {
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
    seed: u64,
}

impl Game {
    fn new(seed: u64, assets: &Assets) -> Self {
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
            seed,
        };
        game.seed = seed;
        game.reset_round(assets);
        game.state = ScreenState::Title;
        game
    }

    fn reset_round(&mut self, _assets: &Assets) {
        self.world = World::new(&mut self.rng);
        self.tanks = spawn_tanks(&mut self.rng, &self.world);
        self.bullets.clear();
        self.tracks.clear();
        self.explosions.clear();
        self.round_timer = ROUND_TIME;
        self.team_kills = [0, 0];
        self.last_winner = None;
    }

    fn update(&mut self, dt: f32, assets: &Assets, rl: &RaylibHandle) {
        match self.state {
            ScreenState::Title => {
                if is_start_pressed(rl) {
                    self.reset_round(assets);
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
                    self.reset_round(assets);
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

        for tank in &mut self.tanks {
            if !tank.alive {
                tank.respawn_timer -= dt;
                if tank.respawn_timer <= 0.0 {
                    tank.alive = true;
                    tank.pos = self.world.random_point_in_zone(tank.team, &mut self.rng);
                    tank.body_angle = random_angle(&mut self.rng);
                    tank.turret_angle = tank.body_angle;
                    tank.fire_cooldown = FIRE_COOLDOWN * 0.5;
                }
                continue;
            }

            tank.fire_cooldown = (tank.fire_cooldown - dt).max(0.0);
            tank.tread_phase += dt * tank.speed * 0.02;

            let (target_pos, target_dist) = find_target_snapshot(&snapshot, tank.team, tank.pos);
            let mut desired_dir = vec2(0.0, 0.0);

            if let Some(target) = target_pos {
                if target_dist > 260.0 {
                    desired_dir = vec2_normalize(vec2_sub(target, tank.pos));
                } else if target_dist < 180.0 {
                    desired_dir = vec2_normalize(vec2_sub(tank.pos, target));
                } else {
                    let to_target = vec2_sub(target, tank.pos);
                    desired_dir = vec2_normalize(vec2(-to_target.y, to_target.x));
                }

                let target_angle = vec2_angle(vec2_sub(target, tank.pos));
                tank.turret_angle = rotate_towards(tank.turret_angle, target_angle, TURRET_ROT_SPEED * dt);

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
                let to_waypoint = vec2_sub(tank.waypoint, tank.pos);
                if vec2_length(to_waypoint) < 80.0 {
                    tank.waypoint = pick_waypoint(&self.world, tank.team, &mut self.rng);
                }
                desired_dir = vec2_normalize(to_waypoint);
                tank.turret_angle = rotate_towards(tank.turret_angle, tank.body_angle, TURRET_ROT_SPEED * dt);
            }

            let avoidance = self.avoidance_vector(tank.team, tank.pos);
            let steer = vec2_normalize(vec2_add(desired_dir, vec2_scale(avoidance, 1.4)));
            if vec2_length(steer) > 0.1 {
                let target_angle = vec2_angle(steer);
                tank.body_angle = rotate_towards(tank.body_angle, target_angle, BODY_ROT_SPEED * dt);
                let velocity = vec2_scale(vec2_from_angle(tank.body_angle), tank.speed);
                let new_pos = vec2_add(tank.pos, vec2_scale(velocity, dt));
                if self.position_clear(tank.team, new_pos, TANK_RADIUS) {
                    let distance = vec2_distance(tank.pos, new_pos);
                    tank.track_distance += distance;
                    tank.pos = new_pos;
                    if tank.track_distance > 40.0 {
                        tank.track_distance = 0.0;
                        new_tracks.push(TrackMark {
                            pos: vec2_add(
                                tank.pos,
                                vec2_scale(vec2_from_angle(tank.body_angle + PI), 18.0),
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


    fn avoidance_vector(&self, team: Team, pos: Vector2) -> Vector2 {
        let mut steer = vec2(0.0, 0.0);
        for obstacle in &self.world.obstacles {
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

        if self.world.is_inside_enemy_zone(team, pos) {
            let zone = self
                .world
                .spawn_zones
                .iter()
                .find(|zone| zone.team == team.enemy())
                .unwrap();
            let zone_center = vec2(
                zone.rect.x + zone.rect.width * 0.5,
                zone.rect.y + zone.rect.height * 0.5,
            );
            steer = vec2_add(steer, vec2_normalize(vec2_sub(pos, zone_center)));
        }

        steer
    }

    fn position_clear(&self, team: Team, pos: Vector2, radius: f32) -> bool {
        let bounds = self.world.world_bounds();
        if pos.x - radius < bounds.x
            || pos.y - radius < bounds.y
            || pos.x + radius > bounds.x + bounds.width
            || pos.y + radius > bounds.y + bounds.height
        {
            return false;
        }
        if self.world.is_inside_enemy_zone(team, pos) {
            return false;
        }
        for obstacle in &self.world.obstacles {
            if vec2_distance(pos, obstacle.pos) < obstacle.radius + radius {
                return false;
            }
        }
        true
    }

    fn draw(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
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
                        TileKind::Grass => &assets.tiles.grass,
                        TileKind::Dirt => &assets.tiles.dirt,
                        TileKind::Sand => &assets.tiles.sand,
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
        for id in 0..TANKS_PER_TEAM {
            let pos = world.random_point_in_zone(team, rng);
            let angle = random_angle(rng);
            tanks.push(Tank {
                id,
                team,
                pos,
                body_angle: angle,
                turret_angle: angle,
                speed: TANK_SPEED + rng.gen_range(-12.0..14.0),
                fire_cooldown: rng.gen_range(0.0..0.8),
                alive: true,
                respawn_timer: 0.0,
                waypoint: pick_waypoint(world, team, rng),
                track_distance: rng.gen_range(0.0..40.0),
                tread_phase: rng.gen_range(0.0..3.0),
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
            rng.gen_range(bounds.x + margin..bounds.x + bounds.width - margin),
            rng.gen_range(bounds.y + margin..bounds.y + bounds.height - margin),
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

fn obstacle_radius(kind: ObstacleKind) -> f32 {
    match kind {
        ObstacleKind::TreeSmall => 40.0,
        ObstacleKind::TreeLarge => 60.0,
        ObstacleKind::SandbagBrown | ObstacleKind::SandbagBeige => 30.0,
        ObstacleKind::Oil => 24.0,
        ObstacleKind::BarrelRedUp
        | ObstacleKind::BarrelRedSide
        | ObstacleKind::BarrelGreyUp
        | ObstacleKind::BarrelGreySide
        | ObstacleKind::BarrelGreyRust
        | ObstacleKind::BarrelGreenUp
        | ObstacleKind::BarrelGreenSide
        | ObstacleKind::BarrelGreenSideDamaged => 22.0,
    }
}

fn obstacle_texture<'a>(assets: &'a Assets, kind: ObstacleKind) -> &'a Texture2D {
    match kind {
        ObstacleKind::TreeSmall => &assets.obstacles.tree_small,
        ObstacleKind::TreeLarge => &assets.obstacles.tree_large,
        ObstacleKind::SandbagBrown => &assets.obstacles.sandbag_brown,
        ObstacleKind::SandbagBeige => &assets.obstacles.sandbag_beige,
        ObstacleKind::Oil => &assets.obstacles.oil,
        ObstacleKind::BarrelRedUp => &assets.obstacles.barrel_red_up,
        ObstacleKind::BarrelRedSide => &assets.obstacles.barrel_red_side,
        ObstacleKind::BarrelGreyUp => &assets.obstacles.barrel_grey_up,
        ObstacleKind::BarrelGreySide => &assets.obstacles.barrel_grey_side,
        ObstacleKind::BarrelGreyRust => &assets.obstacles.barrel_grey_rust,
        ObstacleKind::BarrelGreenUp => &assets.obstacles.barrel_green_up,
        ObstacleKind::BarrelGreenSide => &assets.obstacles.barrel_green_side,
        ObstacleKind::BarrelGreenSideDamaged => &assets.obstacles.barrel_green_side_damaged,
    }
}

fn tank_palette<'a>(assets: &'a Assets, team: Team) -> &'a TankPalette {
    match team {
        Team::Red => &assets.tanks.red,
        Team::Blue => &assets.tanks.blue,
    }
}

fn bullet_palette<'a>(assets: &'a Assets, team: Team) -> &'a BulletPalette {
    match team {
        Team::Red => &assets.bullets.red,
        Team::Blue => &assets.bullets.blue,
    }
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

fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2 { x, y }
}

fn vec2_add(a: Vector2, b: Vector2) -> Vector2 {
    vec2(a.x + b.x, a.y + b.y)
}

fn vec2_sub(a: Vector2, b: Vector2) -> Vector2 {
    vec2(a.x - b.x, a.y - b.y)
}

fn vec2_scale(v: Vector2, s: f32) -> Vector2 {
    vec2(v.x * s, v.y * s)
}

fn vec2_length(v: Vector2) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}

fn vec2_distance(a: Vector2, b: Vector2) -> f32 {
    vec2_length(vec2_sub(a, b))
}

fn vec2_normalize(v: Vector2) -> Vector2 {
    let len = vec2_length(v);
    if len > 0.0 {
        vec2_scale(v, 1.0 / len)
    } else {
        vec2(0.0, 0.0)
    }
}

fn vec2_from_angle(angle: f32) -> Vector2 {
    vec2(angle.cos(), angle.sin())
}

fn vec2_angle(v: Vector2) -> f32 {
    v.y.atan2(v.x)
}

fn angle_difference(a: f32, b: f32) -> f32 {
    let mut diff = b - a;
    while diff > PI {
        diff -= PI * 2.0;
    }
    while diff < -PI {
        diff += PI * 2.0;
    }
    diff.abs()
}

fn rotate_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let mut diff = target - current;
    while diff > PI {
        diff -= PI * 2.0;
    }
    while diff < -PI {
        diff += PI * 2.0;
    }
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}

fn random_angle(rng: &mut SmallRng) -> f32 {
    rng.gen_range(0.0..(PI * 2.0))
}

fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let clamped = alpha.clamp(0.0, 1.0);
    Color::new(color.r, color.g, color.b, (clamped * color.a as f32) as u8)
}

fn point_in_bounds(pos: Vector2, bounds: &Rectangle) -> bool {
    pos.x >= bounds.x
        && pos.x <= bounds.x + bounds.width
        && pos.y >= bounds.y
        && pos.y <= bounds.y + bounds.height
}

fn push_outside_rect(pos: Vector2, rect: Rectangle, margin: f32) -> Vector2 {
    if pos.x < rect.x
        || pos.x > rect.x + rect.width
        || pos.y < rect.y
        || pos.y > rect.y + rect.height
    {
        return pos;
    }

    let dist_left = (pos.x - rect.x).abs();
    let dist_right = (rect.x + rect.width - pos.x).abs();
    let dist_top = (pos.y - rect.y).abs();
    let dist_bottom = (rect.y + rect.height - pos.y).abs();

    if dist_left <= dist_right && dist_left <= dist_top && dist_left <= dist_bottom {
        vec2(rect.x - margin, pos.y)
    } else if dist_right <= dist_top && dist_right <= dist_bottom {
        vec2(rect.x + rect.width + margin, pos.y)
    } else if dist_top <= dist_bottom {
        vec2(pos.x, rect.y - margin)
    } else {
        vec2(pos.x, rect.y + rect.height + margin)
    }
}

fn is_start_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
}
