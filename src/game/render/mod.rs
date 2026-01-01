mod helpers;
mod hud;
mod title;
mod world;

use raylib::prelude::RaylibDraw;

use crate::assets::Assets;

use super::{Game, ScreenState};

impl Game {
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        assets: &Assets,
        screen_width: i32,
        screen_height: i32,
    ) {
        d.clear_background(raylib::prelude::Color::new(32, 96, 160, 255));
        match self.state {
            ScreenState::Title => self.draw_title(d, assets, screen_width, screen_height),
            ScreenState::Playing | ScreenState::RoundOver => {
                self.draw_world(d, assets, screen_width, screen_height);
                self.draw_hud(d, screen_width, screen_height);
                if self.state == ScreenState::RoundOver {
                    self.draw_round_over(d, screen_width, screen_height);
                }
            }
        }
    }
}
