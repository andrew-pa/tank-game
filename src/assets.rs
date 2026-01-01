use raylib::prelude::*;

use crate::entities::Team;
use crate::world::ObstacleKind;

pub struct TankPalette {
    pub body: Texture2D,
    pub barrel: Texture2D,
    pub outline_body: Texture2D,
    pub outline_barrel: Texture2D,
}

pub struct BulletPalette {
    pub normal: Texture2D,
    pub silver: Texture2D,
    pub outline: Texture2D,
    pub silver_outline: Texture2D,
}

pub struct TileTextures {
    pub grass: Texture2D,
    pub dirt: Texture2D,
    pub sand: Texture2D,
}

pub struct TankTextures {
    pub red: TankPalette,
    pub blue: TankPalette,
    pub green: TankPalette,
    pub beige: TankPalette,
    pub black: TankPalette,
}

pub struct BulletTextures {
    pub red: BulletPalette,
    pub blue: BulletPalette,
    pub green: BulletPalette,
    pub beige: BulletPalette,
    pub yellow: BulletPalette,
    pub silver: BulletPalette,
}

pub struct ObstacleTextures {
    pub tree_small: Texture2D,
    pub tree_large: Texture2D,
    pub sandbag_brown: Texture2D,
    pub sandbag_beige: Texture2D,
    pub oil: Texture2D,
    pub barrel_red_up: Texture2D,
    pub barrel_red_side: Texture2D,
    pub barrel_grey_up: Texture2D,
    pub barrel_grey_side: Texture2D,
    pub barrel_grey_rust: Texture2D,
    pub barrel_green_up: Texture2D,
    pub barrel_green_side: Texture2D,
    pub barrel_green_side_damaged: Texture2D,
}

pub struct SmokeTextures {
    pub orange: Vec<Texture2D>,
    pub yellow: Vec<Texture2D>,
    pub grey: Vec<Texture2D>,
    pub white: Vec<Texture2D>,
}

pub struct Assets {
    pub tiles: TileTextures,
    pub tanks: TankTextures,
    pub bullets: BulletTextures,
    pub obstacles: ObstacleTextures,
    pub smoke: SmokeTextures,
    pub tracks_large: Texture2D,
    pub tracks_small: Texture2D,
}

impl Assets {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
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

pub fn load_tank_palette(rl: &mut RaylibHandle, thread: &RaylibThread, color: &str) -> TankPalette {
    TankPalette {
        body: rl
            .load_texture(thread, &format!("assets/PNG/Tanks/tank{color}.png"))
            .expect("tank body"),
        barrel: rl
            .load_texture(thread, &format!("assets/PNG/Tanks/barrel{color}.png"))
            .expect("tank barrel"),
        outline_body: rl
            .load_texture(thread, &format!("assets/PNG/Tanks/tank{color}_outline.png"))
            .expect("tank outline"),
        outline_barrel: rl
            .load_texture(
                thread,
                &format!("assets/PNG/Tanks/barrel{color}_outline.png"),
            )
            .expect("barrel outline"),
    }
}

pub fn load_bullet_palette(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    color: &str,
) -> BulletPalette {
    BulletPalette {
        normal: rl
            .load_texture(thread, &format!("assets/PNG/Bullets/bullet{color}.png"))
            .expect("bullet"),
        silver: rl
            .load_texture(
                thread,
                &format!("assets/PNG/Bullets/bullet{color}Silver.png"),
            )
            .expect("bullet silver"),
        outline: rl
            .load_texture(
                thread,
                &format!("assets/PNG/Bullets/bullet{color}_outline.png"),
            )
            .expect("bullet outline"),
        silver_outline: rl
            .load_texture(
                thread,
                &format!("assets/PNG/Bullets/bullet{color}Silver_outline.png"),
            )
            .expect("bullet silver outline"),
    }
}

pub fn load_smoke_frames(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    color: &str,
) -> Vec<Texture2D> {
    (0..6)
        .map(|idx| {
            rl.load_texture(thread, &format!("assets/PNG/Smoke/smoke{color}{idx}.png"))
                .expect("smoke")
        })
        .collect()
}

pub fn obstacle_texture<'a>(assets: &'a Assets, kind: ObstacleKind) -> &'a Texture2D {
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

pub fn tank_palette<'a>(assets: &'a Assets, team: Team) -> &'a TankPalette {
    match team {
        Team::Red => &assets.tanks.red,
        Team::Blue => &assets.tanks.blue,
    }
}

pub fn bullet_palette<'a>(assets: &'a Assets, team: Team) -> &'a BulletPalette {
    match team {
        Team::Red => &assets.bullets.red,
        Team::Blue => &assets.bullets.blue,
    }
}
