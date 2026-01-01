use rand::{Rng, rngs::SmallRng};
use raylib::prelude::{Color, Rectangle, Vector2};
use std::f32::consts::PI;

pub fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2 { x, y }
}

pub fn vec2_add(a: Vector2, b: Vector2) -> Vector2 {
    vec2(a.x + b.x, a.y + b.y)
}

pub fn vec2_sub(a: Vector2, b: Vector2) -> Vector2 {
    vec2(a.x - b.x, a.y - b.y)
}

pub fn vec2_scale(v: Vector2, s: f32) -> Vector2 {
    vec2(v.x * s, v.y * s)
}

pub fn vec2_length(v: Vector2) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}

pub fn vec2_distance(a: Vector2, b: Vector2) -> f32 {
    vec2_length(vec2_sub(a, b))
}

pub fn vec2_normalize(v: Vector2) -> Vector2 {
    let len = vec2_length(v);
    if len > 0.0 {
        vec2_scale(v, 1.0 / len)
    } else {
        vec2(0.0, 0.0)
    }
}

pub fn vec2_from_angle(angle: f32) -> Vector2 {
    vec2(angle.cos(), angle.sin())
}

pub fn vec2_angle(v: Vector2) -> f32 {
    v.y.atan2(v.x)
}

pub fn angle_difference(a: f32, b: f32) -> f32 {
    let mut diff = b - a;
    while diff > PI {
        diff -= PI * 2.0;
    }
    while diff < -PI {
        diff += PI * 2.0;
    }
    diff.abs()
}

pub fn rotate_towards(current: f32, target: f32, max_delta: f32) -> f32 {
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

pub fn random_angle(rng: &mut SmallRng) -> f32 {
    rng.random_range(0.0..(PI * 2.0))
}

pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

pub fn with_alpha(color: Color, alpha: f32) -> Color {
    let clamped = alpha.clamp(0.0, 1.0);
    Color::new(color.r, color.g, color.b, (clamped * color.a as f32) as u8)
}

pub fn point_in_bounds(pos: Vector2, bounds: &Rectangle) -> bool {
    pos.x >= bounds.x
        && pos.x <= bounds.x + bounds.width
        && pos.y >= bounds.y
        && pos.y <= bounds.y + bounds.height
}

pub fn push_outside_rect(pos: Vector2, rect: Rectangle, margin: f32) -> Vector2 {
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
