use glam::{UVec2, Vec2};
use glam_rect::{Rect, URect};
use speedy2d::color::Color;
use speedy2d::window::{VirtualKeyCode, WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};

static SIZE: (u8, u8) = (128, 128);
static SCALE: f32 = 5.0;

mod packer;
use packer::TexturePacker;

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

    pub fn apply_bunch(&mut self) {
        let steps = 50;
        let mut allocations = Vec::with_capacity(50);
        for _ in 0..steps {
            allocations.push(UVec2::new(fastrand::u32(2..25), fastrand::u32(2..25)));
        }
        self.rects.extend(
            self.packer
                .pack(allocations)
                .into_iter()
                .filter_map(|result| result.ok()),
        );
        self.step += steps;
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
        for (i, rect) in self.packer.clone().areas.into_iter().enumerate() {
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
                VirtualKeyCode::Key2 => self.apply_bunch(),
                VirtualKeyCode::Escape => helper.terminate_loop(),
                VirtualKeyCode::Key1 => self.reset(),
                _ => (),
            }
        }
    }
}
