use crate::entities::Tank;

pub(super) fn speed_multiplier(tank: &Tank) -> f32 {
    if tank.invincible_timer > 0.0 {
        1.15
    } else {
        1.0
    }
}

pub(super) fn fire_rate_multiplier(tank: &Tank) -> f32 {
    if tank.rapid_timer > 0.0 {
        1.2
    } else {
        1.0
    }
}

pub(super) fn range_multiplier(tank: &Tank) -> f32 {
    if tank.rapid_timer > 0.0 {
        2.0
    } else {
        1.0
    }
}
