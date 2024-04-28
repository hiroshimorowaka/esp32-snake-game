use crate::snake::{Direction, Snake, SNAKE_AND_FOOD_SIZE};
use crate::DisplayController;
use alloc::format;
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

const SNAKE_DEFAULT_X: i32 = 24;
const SNAKE_DEFAULT_Y: i32 = 20;

struct Board {
    width: u32,
    height: u32,
}

struct Food {
    x: i32,
    y: i32,
    size: u32,
}

impl Default for Food {
    fn default() -> Self {
        Self {
            x: 32,
            y: 16,
            size: 4,
        }
    }
}

pub struct Game {
    board: Board,
    snake: Snake,
    display: DisplayController,
    game_over: bool,
    food_exists: bool,
    food: Food,
    score: u32,
}

impl Game {
    pub fn new(width: u32, height: u32, display: DisplayController) -> Self {
        Self {
            score: 0,
            food: Food::default(),
            food_exists: true,
            game_over: false,
            board: Board { width, height },
            snake: Snake::new(SNAKE_DEFAULT_X, SNAKE_DEFAULT_Y),
            display,
        }
    }

    pub async fn draw(&mut self) {
        let mut display = self.display.lock().await;

        display.init().unwrap();

        if self.game_over {
            Text::with_baseline("Game Over", Point::new(36, 32), TEXT_STYLE, Baseline::Top)
                .draw(&mut *display)
                .unwrap();
            Text::with_baseline(
                format!("Score: {}", self.score).as_str(),
                Point::new(36, 42),
                TEXT_STYLE,
                Baseline::Top,
            )
            .draw(&mut *display)
            .unwrap();
            display.flush().unwrap();
            return;
        }

        // Draw board walls
        Rectangle::new(
            Point::new(2, 2),
            Size::new(self.board.width - 4, self.board.height - 4),
        )
        .into_styled(BOARD_STYLE)
        .draw(&mut *display)
        .unwrap();

        //Draw snake
        for block in &self.snake.body {
            Rectangle::new(
                Point::new(block.x as i32, block.y as i32),
                Size::new(block.size, block.size),
            )
            .into_styled(SNAKE_STYLE)
            .draw(&mut *display)
            .unwrap();
        }

        //Draw food
        if self.food_exists {
            Rectangle::new(
                Point::new(self.food.x as i32, self.food.y as i32),
                Size::new(self.food.size, self.food.size),
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
        let mut new_x = (rng.usize(1..self.board.width as usize))
            .next_multiple_of(SNAKE_AND_FOOD_SIZE as usize)
            .clamp(SNAKE_AND_FOOD_SIZE as usize, 120);
        let mut new_y = (rng.usize(1..self.board.height as usize))
            .next_multiple_of(SNAKE_AND_FOOD_SIZE as usize)
            .clamp(SNAKE_AND_FOOD_SIZE as usize, 56);

        while self.snake.overlap_tail(new_x as i32, new_y as i32) {
            new_x = (rng.usize(1..self.board.width as usize))
                .next_multiple_of(SNAKE_AND_FOOD_SIZE as usize)
                .clamp(SNAKE_AND_FOOD_SIZE as usize, 120);
            new_y = (rng.usize(1..self.board.height as usize))
                .next_multiple_of(SNAKE_AND_FOOD_SIZE as usize)
                .clamp(SNAKE_AND_FOOD_SIZE as usize, 56);
        }
        self.food.x = new_x as i32;
        self.food.y = new_y as i32;
        self.food_exists = true;
    }

    fn check_eating(&mut self) {
        let (head_x, head_y) = self.snake.head_position();
        if head_x == self.food.x && head_y == self.food.y {
            self.food_exists = false;
            self.snake.restore_tail();
            self.score += 4;
        }
    }

    fn check_if_snake_alive(&self) -> bool {
        let (next_x, next_y) = self.snake.next_head();
        if self.snake.overlap_tail(next_x, next_y) {
            return false;
        }

        let result = next_x > 0
            && next_y > 0
            && next_x < self.board.width as i32 - SNAKE_AND_FOOD_SIZE as i32
            && next_y < self.board.height as i32 - SNAKE_AND_FOOD_SIZE as i32;
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
            if up_button.is_high().unwrap() || right_button.is_high().unwrap() {
                self.restart();
            }
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

    fn restart(&mut self) {
        self.snake = Snake::new(SNAKE_DEFAULT_X, SNAKE_DEFAULT_Y);
        self.food_exists = true;
        self.game_over = false;
        self.food = Food::default();
        self.score = 0;
    }
}
