use crate::display::{Color, Display, BLOCK_SIZE};
use crate::serial_println;
use crate::world::{Game, GameState, Direction};
use alloc::collections::VecDeque;
use rand::prelude::*;


use crate::world::ScreenPos as Point;

#[derive(Debug, PartialEq, Eq)]
pub struct World {
    pub width: usize,
    pub height: usize,
    pub food: Option<Point>,
    pub score: usize,
    pub game_over: bool,
    pub speed: usize,
    pub direction: Direction,
    pub snake_length: usize,
    pub snake_head: Point,
    pub snake_body: VecDeque<Point>,
    pub snake_tail: Option<Point>,
    rng: rand::rngs::SmallRng,
    counter: u64,
}

impl World {
    pub fn new(width: usize, height: usize) -> World {
        let start = Point {
            x: width / 2,
            y: height / 2,
        };
        World {
            width,
            height,
            food: None,
            score: 0,
            game_over: false,
            speed: 1,
            direction: Direction::Right,
            snake_length: 1,
            snake_head: start,
            snake_body: VecDeque::new(),
            snake_tail: None,
            rng: rand::rngs::SmallRng::from_seed([0; 32]),
            counter: 0,
        }
    }

    fn place_random_food(&mut self) {
        let mut point = Point { x: 0, y: 0 };
        while self.snake_body.contains(&point)
            || self.snake_head == point
            || point.x >= self.width - 2 * BLOCK_SIZE
            || point.y >= self.height - 2 * BLOCK_SIZE
            || point.x <= 2 * BLOCK_SIZE
            || point.y <= 2 * BLOCK_SIZE
        {
            point = Point {
                x: self.rand(self.width),
                y: self.rand(self.height),
            };
            point.x = point.x - point.x % BLOCK_SIZE;
            point.y = point.y - point.y % BLOCK_SIZE;
        }
        self.food = Some(point);
    }

    // rand implements a simple pseudo random number generator
    // that returns a random number between 0 and max
    fn rand(&mut self, max: usize) -> usize {
        let result = self.rng.next_u64() as usize % max;
        result
    }
}

impl Game for World {
    fn reset(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.food = None;
        self.score = 0;
        self.game_over = false;
        self.speed = 2;
        self.direction = Direction::Right;
        self.snake_length = 1;
        self.snake_head = Point {
            x: width / 2,
            y: height / 2,
        };
        self.snake_body = VecDeque::new();
        self.snake_tail = None;
        self.counter = 0;
    }

    // step moves the snake one step forward
    fn step(&mut self) -> GameState {
        self.counter += 1;
        if self.counter % self.speed as u64 != 0 {
            return GameState::Live;
        }
        if self.game_over {
            return GameState::GameOver;
        }
        // update snake head
        let mut new_head = self.snake_head;
        if self.direction == Direction::Up {
            new_head.y -= BLOCK_SIZE;
        } else if self.direction == Direction::Right {
            new_head.x += BLOCK_SIZE;
        } else if self.direction == Direction::Down {
            new_head.y += BLOCK_SIZE;
        } else if self.direction == Direction::Left {
            new_head.x -= BLOCK_SIZE;
        }
        new_head.x = new_head.x % self.width;
        new_head.y = new_head.y % self.height;

        self.snake_body.push_back(self.snake_head);
        self.snake_head = new_head;
        if let Some(food) = self.food {
            if self.snake_head == food {
                serial_println!("found food!!!");
                self.score += 1;
                self.food = None;
                self.snake_length += 1;
            } else {
                self.snake_tail = self.snake_body.pop_front();
            }
        } else {
            self.place_random_food();
            self.snake_tail = self.snake_body.pop_front();
        }
        if self.snake_head.x >= self.width - 2 * BLOCK_SIZE
            || self.snake_head.y >= self.height - 2 * BLOCK_SIZE
            || self.snake_head.x <= BLOCK_SIZE
            || self.snake_head.y <= BLOCK_SIZE
        {
            self.game_over = true;
        }
        for body in self.snake_body.iter() {
            if self.snake_head == *body {
                self.game_over = true;
            }
        }

        if self.game_over {
            GameState::GameOver
        } else {
            GameState::Live
        }
    }

    fn on_keypress(&mut self, key: pc_keyboard::DecodedKey) {
        let w = self;
        match key {
            pc_keyboard::DecodedKey::Unicode(character) => match character {
                'a' => {
                    if w.direction != Direction::Right {
                        w.direction = Direction::Left;
                    }
                }
                'd' => {
                    if w.direction != Direction::Left {
                        w.direction = Direction::Right;
                    }
                }
                'w' => {
                    if w.direction != Direction::Down {
                        w.direction = Direction::Up;
                    }
                }
                's' => {
                    if w.direction != Direction::Up {
                        w.direction = Direction::Down;
                    }
                }
                _ => (),
            },
            pc_keyboard::DecodedKey::RawKey(key) => match key {
                pc_keyboard::KeyCode::ArrowLeft => w.direction = Direction::Left,
                pc_keyboard::KeyCode::ArrowRight => w.direction = Direction::Right,
                pc_keyboard::KeyCode::ArrowUp => w.direction = Direction::Up,
                pc_keyboard::KeyCode::ArrowDown => w.direction = Direction::Down,
                _ => {}
            },
        }
    }

    fn draw(&mut self, display: &mut Display) {
        // if game is over, print "GAME OVER"
        if self.game_over {
            return;
        }

        // draw food
        if let Some(food) = self.food {
            display.write_block(food.x, food.y, Color::LightRed);
        }
        if let Some(tail) = self.snake_tail {
            display.write_block(tail.x, tail.y, Color::Black);
            self.snake_tail = None;
        }
        // draw snake head
        display.write_block(self.snake_head.x, self.snake_head.y, Color::Green);
        // draw snake
        for part in self.snake_body.iter() {
            display.write_block(part.x, part.y, Color::LightGreen);
        }
    }
}
