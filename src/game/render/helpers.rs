use raylib::prelude::{Color, RaylibDraw, RaylibDrawHandle, Rectangle, Texture2D, Vector2};

use crate::assets::{Assets, TankPalette};
use crate::config::TANK_RADIUS;
use crate::entities::{Explosion, Powerup, PowerupKind, SmokeColor, Tank};
use crate::math::{rad_to_deg, vec2};

use super::super::constants::SPRITE_ROT_OFFSET_DEG;

pub(super) fn explosion_frame<'a>(assets: &'a Assets, explosion: &Explosion) -> &'a Texture2D {
    let frame = (explosion.age / 0.08).floor() as usize;
    let idx = frame.min(5);
    match explosion.color {
        SmokeColor::Orange => &assets.smoke.orange[idx],
        SmokeColor::Yellow => &assets.smoke.yellow[idx],
        SmokeColor::Grey => &assets.smoke.grey[idx],
        SmokeColor::White => &assets.smoke.white[idx],
    }
}

pub(super) fn draw_tank_preview(d: &mut RaylibDrawHandle, palette: &TankPalette, pos: Vector2) {
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

pub(super) fn draw_texture_centered(
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

pub(super) fn draw_barrel(
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

pub(super) fn sprite_rotation(angle: f32) -> f32 {
    rad_to_deg(angle) + SPRITE_ROT_OFFSET_DEG
}

pub(super) fn draw_tank_health(d: &mut RaylibDrawHandle, tank: &Tank) {
    if tank.health >= tank.max_health || tank.health_flash <= 0.0 {
        return;
    }
    let pct = (tank.health / tank.max_health).clamp(0.0, 1.0);
    let bar_w = 44.0;
    let bar_h = 6.0;
    let x = tank.pos.x - bar_w * 0.5;
    let y = tank.pos.y - TANK_RADIUS - 16.0;
    d.draw_rectangle(
        x as i32,
        y as i32,
        bar_w as i32,
        bar_h as i32,
        Color::new(10, 10, 10, 190),
    );
    d.draw_rectangle(
        (x + 1.0) as i32,
        (y + 1.0) as i32,
        ((bar_w - 2.0) * pct) as i32,
        (bar_h - 2.0) as i32,
        tank.team.color(),
    );
}

pub(super) fn draw_powerup_markers(d: &mut RaylibDrawHandle, tank: &Tank) {
    let mut ring = 0.0;
    if tank.invincible_timer > 0.0 {
        ring += 1.0;
        let pulse = (tank.invincible_timer * 4.0).sin().abs();
        d.draw_circle_lines(
            tank.pos.x as i32,
            tank.pos.y as i32,
            TANK_RADIUS + 8.0 + pulse * 4.0,
            invincible_color(220),
        );
    }
    if tank.rapid_timer > 0.0 {
        ring += 1.0;
        let pulse = (tank.rapid_timer * 5.0).sin().abs();
        d.draw_circle_lines(
            tank.pos.x as i32,
            tank.pos.y as i32,
            TANK_RADIUS + 4.0 + ring * 6.0 + pulse * 3.0,
            rapid_color(220),
        );
    }
}

pub(super) fn draw_powerup(d: &mut RaylibDrawHandle, assets: &Assets, powerup: &Powerup) {
    let frames = match powerup.kind {
        PowerupKind::Invincible => &assets.smoke.white,
        PowerupKind::RapidRange => &assets.smoke.orange,
        PowerupKind::Heal => &assets.smoke.yellow,
    };
    let idx = ((powerup.age * 8.0) as usize) % frames.len();
    let pulse = (powerup.age * 6.0).sin().abs();
    let tint = powerup_color(powerup.kind, 235);
    draw_texture_centered(d, &frames[idx], powerup.pos, 0.0, tint);
    d.draw_circle_lines(
        powerup.pos.x as i32,
        powerup.pos.y as i32,
        26.0 + pulse * 6.0,
        powerup_color(powerup.kind, 220),
    );
}

fn powerup_color(kind: PowerupKind, alpha: u8) -> Color {
    match kind {
        PowerupKind::Invincible => invincible_color(alpha),
        PowerupKind::RapidRange => rapid_color(alpha),
        PowerupKind::Heal => heal_color(alpha),
    }
}

fn invincible_color(alpha: u8) -> Color {
    Color::new(120, 210, 255, alpha)
}

fn rapid_color(alpha: u8) -> Color {
    Color::new(255, 200, 90, alpha)
}

fn heal_color(alpha: u8) -> Color {
    Color::new(120, 230, 160, alpha)
}
