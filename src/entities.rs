use raylib::prelude::{Color, Vector2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    Red,
    Blue,
}

impl Team {
    pub fn index(self) -> usize {
        match self {
            Team::Red => 0,
            Team::Blue => 1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Team::Red => "Crimson",
            Team::Blue => "Azure",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Team::Red => Color::new(224, 70, 70, 255),
            Team::Blue => Color::new(70, 140, 232, 255),
        }
    }

    pub fn enemy(self) -> Team {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SmokeColor {
    Orange,
    Yellow,
    Grey,
    White,
}

#[derive(Clone, Debug)]
pub struct Tank {
    pub team: Team,
    pub pos: Vector2,
    pub body_angle: f32,
    pub turret_angle: f32,
    pub speed: f32,
    pub fire_cooldown: f32,
    pub alive: bool,
    pub respawn_timer: f32,
    pub waypoint: Vector2,
    pub track_distance: f32,
    pub tread_phase: f32,
    pub health: f32,
    pub max_health: f32,
    pub health_flash: f32,
    pub invincible_timer: f32,
    pub rapid_timer: f32,
}

#[derive(Clone, Debug)]
pub struct Bullet {
    pub pos: Vector2,
    pub vel: Vector2,
    pub team: Team,
    pub life: f32,
}

#[derive(Clone, Debug)]
pub struct TrackMark {
    pub pos: Vector2,
    pub rotation: f32,
    pub age: f32,
}

#[derive(Clone, Debug)]
pub struct Explosion {
    pub pos: Vector2,
    pub color: SmokeColor,
    pub age: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum PowerupKind {
    Invincible,
    RapidRange,
    Heal,
}

#[derive(Clone, Debug)]
pub struct Powerup {
    pub kind: PowerupKind,
    pub pos: Vector2,
    pub age: f32,
}
