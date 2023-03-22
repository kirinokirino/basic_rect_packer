use glam_rect::{Rect, URect};
use speedy2d::color::Color;
use speedy2d::window::{VirtualKeyCode, WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};

static SIZE: (u8, u8) = (128, 128);
static SCALE: f32 = 5.0;
fn main() {
    let window = Window::new_centered(
        "FLOATING",
        (
            (SIZE.0 as f32 * SCALE) as u32,
            (SIZE.1 as f32 * SCALE) as u32,
        ),
    )
    .unwrap();
    window.run_loop(MyWindowHandler::new())
}

struct MyWindowHandler {
    rects: Vec<URect>,
    packer: TexturePacker,
    step: usize,
}

impl MyWindowHandler {
    fn new() -> Self {
        let packer = TexturePacker::new(SIZE.0 as u32, SIZE.1 as u32);
        Self {
            rects: Vec::new(),
            packer,
            step: 0,
        }
    }

    pub fn reset(&mut self) {
        self.packer = TexturePacker::new(SIZE.0 as u32, SIZE.1 as u32);
        self.rects.clear();
    }

    pub fn apply_step(&mut self) {
        let allocation = UVec2::new(fastrand::u32(2..25), fastrand::u32(2..25));
        if let Ok(rect) = self.packer.try_allocate(allocation) {
            self.rects.push(rect);
        }
        self.step += 1;
    }
}

impl WindowHandler for MyWindowHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        graphics.clear_screen(Color::WHITE);
        let scale = Vec2::splat(SCALE);
        let mut colors = [
            Color::BLUE,
            Color::CYAN,
            Color::DARK_GRAY,
            Color::GREEN,
            Color::LIGHT_GRAY,
            Color::MAGENTA,
        ]
        .into_iter()
        .cycle();

        graphics.draw_rectangle(
            Rect::new(
                Vec2::new(0.0, 0.0) * scale,
                Vec2::new(SIZE.0 as f32, SIZE.1 as f32) * scale,
            ),
            Color::from_gray(0.8),
        );
        for (i, free_area) in self.packer.clone().areas.into_iter().enumerate() {
            let FreeRegion { rect } = free_area;
            graphics.draw_rectangle(
                Rect::new(
                    rect.top_left.as_vec2() * scale,
                    rect.bottom_right.as_vec2() * scale,
                ),
                Color::from_gray(0.5 - (0.5 * i as f32 * 0.05)),
            )
        }
        for rect in &self.rects {
            graphics.draw_rectangle(
                Rect::new(
                    rect.top_left.as_vec2() * scale,
                    rect.bottom_right.as_vec2() * scale,
                ),
                colors.next().unwrap(),
            )
        }
        helper.request_redraw();
    }

    fn on_key_up(
        &mut self,
        helper: &mut WindowHelper<()>,
        virtual_key_code: Option<speedy2d::window::VirtualKeyCode>,
        scancode: speedy2d::window::KeyScancode,
    ) {
        if let Some(key_code) = virtual_key_code {
            match key_code {
                VirtualKeyCode::Space => self.apply_step(),
                VirtualKeyCode::Escape => helper.terminate_loop(),
                VirtualKeyCode::Key1 => self.reset(),
                _ => (),
            }
        }
    }
}

/*
 *  Copyright 2021 QuantumBadger
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use glam::{UVec2, Vec2};

#[derive(Debug, Clone)]
struct FreeRegion {
    rect: URect,
}

impl FreeRegion {
    #[inline]
    fn from_rectangle(rect: URect) -> Self {
        FreeRegion { rect }
    }

    #[inline]
    fn new(width: u32, height: u32) -> Self {
        FreeRegion::from_rectangle(URect::new(UVec2::ZERO, UVec2::new(width, height)))
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum TexturePackerError {
    NotEnoughSpace,
}

#[derive(Debug, Clone)]
pub(crate) struct TexturePacker {
    areas: Vec<FreeRegion>,
}

impl TexturePacker {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        TexturePacker {
            areas: vec![FreeRegion::new(width, height)],
        }
    }

    pub(crate) fn try_allocate(&mut self, size: UVec2) -> Result<URect, TexturePackerError> {
        if size.x == 0 || size.y == 0 {
            return Ok(URect::new(UVec2::ZERO, size));
        }

        let size = size + UVec2::new(2, 2);

        // Add a one-pixel border around each texture
        let width = size.x;
        let height = size.y;

        let mut best_area: Option<&mut FreeRegion> = None;

        for area in &mut self.areas {
            let area_width = area.rect.width();
            let area_height = area.rect.height();

            if width > area.rect.width() || height > area.rect.height() {
                continue;
            }

            let update_best = if let Some(current_best) = &best_area {
                current_best.rect.width() >= area_width && current_best.rect.height() >= area_height
            } else {
                true
            };

            if update_best {
                best_area = Some(area);
            }
        }

        let best_area = best_area.ok_or(TexturePackerError::NotEnoughSpace)?;
        let URect {
            top_left,
            bottom_right,
        } = best_area.rect;

        let space_underneath =
            URect::new(UVec2::new(top_left.x, (top_left + size).y), bottom_right);

        let space_right = URect::new(
            UVec2::new((top_left + size).x, top_left.y),
            space_underneath.top_right(),
        );

        if space_right.is_zero_area() {
            best_area.rect = space_underneath
        } else {
            best_area.rect = space_right;

            if !space_underneath.is_zero_area() {
                self.areas
                    .push(FreeRegion::from_rectangle(space_underneath));
            }
        }

        Ok(URect::new(
            top_left + UVec2::new(1, 1),
            (top_left + size) - UVec2::new(1, 1),
        ))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn pack_test_fill_four_squares() {
        let mut packer = TexturePacker::new(64, 64);

        assert_eq!(
            Ok(URect::from_tuples((1, 1), (31, 31))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((33, 1), (63, 31))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((1, 33), (31, 63))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((33, 33), (63, 63))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Err(TexturePackerError::NotEnoughSpace),
            packer.try_allocate(UVec2::new(30, 30))
        );
    }

    #[test]
    fn pack_test_nonfill_four_squares() {
        let mut packer = TexturePacker::new(64, 64);

        assert_eq!(
            Ok(URect::from_tuples((1, 1), (29, 29))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Ok(URect::from_tuples((31, 1), (59, 29))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Ok(URect::from_tuples((1, 31), (29, 59))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Ok(URect::from_tuples((31, 31), (59, 59))),
            packer.try_allocate(UVec2::new(28, 28))
        );

        assert_eq!(
            Err(TexturePackerError::NotEnoughSpace),
            packer.try_allocate(UVec2::new(30, 30))
        );
    }

    #[test]
    fn pack_test_uneven_squares() {
        let mut packer = TexturePacker::new(64, 64);

        assert_eq!(
            Ok(URect::from_tuples((1, 1), (15, 15))),
            packer.try_allocate(UVec2::new(14, 14))
        );

        assert_eq!(
            Ok(URect::from_tuples((1, 17), (15, 47))),
            packer.try_allocate(UVec2::new(14, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((17, 17), (47, 47))),
            packer.try_allocate(UVec2::new(30, 30))
        );

        assert_eq!(
            Ok(URect::from_tuples((17, 1), (31, 15))),
            packer.try_allocate(UVec2::new(14, 14))
        );
    }
}
