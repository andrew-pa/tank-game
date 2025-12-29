mod helpers;
mod hud;
mod title;
mod world;

use raylib::prelude::{RaylibDraw, RaylibDrawHandle};

use crate::assets::Assets;

use super::{Game, ScreenState};

impl Game {
    pub fn draw(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        d.clear_background(raylib::prelude::Color::new(32, 96, 160, 255));
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
}
