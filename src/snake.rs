use alloc::collections::LinkedList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match *self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub size: u32,
}

pub const SNAKE_AND_FOOD_SIZE: u32 = 4;

pub struct Snake {
    pub direction: Direction,
    pub body: LinkedList<Block>, // The tutorial use LinkedList
    pub tail: Option<Block>,
}

impl Snake {
    pub fn new(x: i32, y: i32) -> Self {
        let mut body: LinkedList<Block> = LinkedList::new();

        body.push_back(Block {
            x: x + SNAKE_AND_FOOD_SIZE as i32 * 2,
            y,
            size: SNAKE_AND_FOOD_SIZE,
        });
        body.push_back(Block {
            x: x + SNAKE_AND_FOOD_SIZE as i32,
            y,
            size: SNAKE_AND_FOOD_SIZE,
        });

        Snake {
            direction: Direction::Right,
            body,
            tail: None,
        }
    }

    pub fn head_position(&self) -> (i32, i32) {
        let head_block = self.body.front().unwrap();
        (head_block.x, head_block.y)
    }

    pub fn change_direction(&mut self, dir: Option<Direction>) {
        if dir == Some(self.head_direction().opposite()) {
            return;
        }

        match dir {
            Some(d) => self.direction = d,
            None => (),
        }
    }

    pub fn move_forward(&mut self) {
        let (last_x, last_y) = self.head_position();

        let new_block = match self.direction {
            Direction::Up => Block {
                x: last_x,
                y: last_y - SNAKE_AND_FOOD_SIZE as i32,
                size: SNAKE_AND_FOOD_SIZE,
            },
            Direction::Down => Block {
                x: last_x,
                y: last_y + SNAKE_AND_FOOD_SIZE as i32,

                size: SNAKE_AND_FOOD_SIZE,
            },
            Direction::Left => Block {
                x: last_x - SNAKE_AND_FOOD_SIZE as i32,
                y: last_y,

                size: SNAKE_AND_FOOD_SIZE,
            },
            Direction::Right => Block {
                x: last_x + SNAKE_AND_FOOD_SIZE as i32,
                y: last_y,

                size: SNAKE_AND_FOOD_SIZE,
            },
        };

        self.body.push_front(new_block);
        let remove_block = self.body.pop_back().unwrap();

        self.tail = Some(remove_block);
    }

    pub fn head_direction(&self) -> Direction {
        self.direction
    }

    pub fn next_head(&self) -> (i32, i32) {
        let (head_x, head_y) = self.head_position();
        match self.direction {
            Direction::Up => (head_x, head_y - SNAKE_AND_FOOD_SIZE as i32),
            Direction::Down => (head_x, head_y + SNAKE_AND_FOOD_SIZE as i32),
            Direction::Left => (head_x - SNAKE_AND_FOOD_SIZE as i32, head_y),
            Direction::Right => (head_x + SNAKE_AND_FOOD_SIZE as i32, head_y),
        }
    }

    pub fn restore_tail(&mut self) {
        let blk = self.tail.clone().unwrap();
        self.body.push_back(blk);
    }

    pub fn overlap_tail(&self, x: i32, y: i32) -> bool {
        let mut ch = 0;
        for block in &self.body {
            if x == block.x && y == block.y {
                return true;
            }

            ch += 1;
            if ch == self.body.len() - 1 {
                break;
            }
        }
        return false;
    }
}
