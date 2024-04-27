use crate::snake::{Direction, Snake};
use crate::DisplayController;
use alloc::sync::Arc;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
};
use esp_hal::gpio::{GpioPin, Input, PullDown};
use esp_hal::prelude::_embedded_hal_digital_v2_InputPin;
use esp_println::println;
use fastrand::Rng;
use ssd1306::mode::DisplayConfig;
const SNAKE_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyleBuilder::new()
    .fill_color(BinaryColor::On)
    .stroke_width(1)
    .stroke_color(BinaryColor::On)
    .build();

const BOARD_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyleBuilder::new()
    .stroke_width(1)
    .stroke_color(BinaryColor::On)
    .build();

const TEXT_STYLE: MonoTextStyle<'static, BinaryColor> = MonoTextStyleBuilder::new()
    .font(&FONT_6X10)
    .text_color(BinaryColor::On)
    .build();

pub struct Board {
    pub width: u32,
    pub height: u32,
}

pub struct Food {
    pub x: i32,
    pub y: i32,
    pub size: u32,
}
pub struct Game {
    pub board: Board,
    pub snake: Snake,
    pub display: DisplayController,
    pub game_over: bool,
    pub food_exists: bool,
    pub food: Food,
}

impl Game {
    pub fn new(width: u32, height: u32, display: DisplayController) -> Self {
        Self {
            food: Food {
                x: 24,
                y: 8,
                size: 4,
            },
            food_exists: true,
            game_over: false,
            board: Board { width, height },
            snake: Snake::new(8, 8),
            display,
        }
    }

    pub async fn draw(&mut self) {
        let mut display = self.display.lock().await;

        display.init().unwrap();

        Rectangle::new(
            Point::new(0, 0),
            Size::new(self.board.width, self.board.height),
        )
        .into_styled(BOARD_STYLE)
        .draw(&mut *display)
        .unwrap();

        if self.game_over {
            Text::with_baseline("Game Over", Point::new(32, 32), TEXT_STYLE, Baseline::Top)
                .draw(&mut *display)
                .unwrap();
            display.flush().unwrap();
            return;
        }

        for block in &self.snake.body {
            Rectangle::new(
                Point::new(block.x as i32, block.y as i32),
                Size::new(block.size, block.size),
            )
            .into_styled(SNAKE_STYLE)
            .draw(&mut *display)
            .unwrap();
        }

        if self.food_exists {
            Rectangle::new(
                Point::new(self.food.x as i32, self.food.y as i32),
                Size::new(3, 3),
            )
            .into_styled(SNAKE_STYLE)
            .draw(&mut *display)
            .unwrap();
        }

        display.flush().unwrap();
    }

    pub fn update(&mut self, rng: Rng) {
        if self.game_over {
            return; // Restart game
        }

        if !self.food_exists {
            self.add_food(rng);
        }

        self.update_snake();
    }

    fn update_snake(&mut self) {
        if self.check_if_snake_alive() {
            self.snake.move_forward();
            self.check_eating();
        } else {
            self.game_over = true;
        }
    }

    fn add_food(&mut self, mut rng: Rng) {
        let mut new_x = rng.usize(1..self.board.width as usize - 1) / 4 * 4;
        let mut new_y = rng.usize(1..self.board.height as usize - 1) / 4 * 4;

        while self.snake.overlap_tail(new_x as i32, new_y as i32) {
            new_x = rng.usize(1..self.board.width as usize - 1) / 4 * 4;
            new_y = rng.usize(1..self.board.height as usize - 1) / 4 * 4;
        }

        self.food.x = new_x as i32;
        self.food.y = new_y as i32;
        self.food_exists = true;
    }

    fn check_eating(&mut self) {
        let (head_x, head_y) = self.snake.head_position();
        println!("{} {}", head_x, head_y);
        println!("{} {}", self.food.x, self.food.y);
        if head_x == self.food.x && head_y == self.food.y {
            self.food_exists = false;
            self.snake.restore_tail();
        }
    }

    fn check_if_snake_alive(&self) -> bool {
        let (next_x, next_y) = self.snake.next_head();
        if self.snake.overlap_tail(next_x, next_y) {
            return false;
        }

        let result = next_x > 0
            && next_y > 0
            && next_x < self.board.width as i32 - 1
            && next_y < self.board.height as i32 - 1;
        return result;
    }
    pub fn handle_input(
        &mut self,
        up_button: Arc<GpioPin<Input<PullDown>, 14>>,
        down_button: Arc<GpioPin<Input<PullDown>, 12>>,
        left_button: Arc<GpioPin<Input<PullDown>, 33>>,
        right_button: Arc<GpioPin<Input<PullDown>, 32>>,
    ) {
        if self.game_over {
            return;
        }

        let direction = if up_button.is_high().unwrap() {
            Direction::Up
        } else if down_button.is_high().unwrap() {
            Direction::Down
        } else if left_button.is_high().unwrap() {
            Direction::Left
        } else if right_button.is_high().unwrap() {
            Direction::Right
        } else {
            return;
        };

        if direction == self.snake.head_direction().opposite() {
            return;
        }
        self.snake.change_direction(Some(direction));
    }

    pub fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.food_exists = true;
        self.game_over = false;
        self.food = Food {
            x: 24,
            y: 8,
            size: 4,
        }
    }
}
