extern crate alloc;
use crate::display::Display;
use core::fmt::Write;

use alloc::boxed::Box;
use alloc::vec::Vec;
use pc_keyboard::{DecodedKey, KeyCode};

#[derive(PartialEq, Debug)]
pub enum GameState {
    Live,
    GameOver,
}

pub trait Game {
    fn on_keypress(&mut self, key: DecodedKey);
    fn reset(&mut self, width: usize, height: usize);

    // meaning of return value:
    //    true  => game over
    //    false => continue
    fn step(&mut self) -> GameState;
    fn draw(&mut self, display: &mut Display);
}

#[derive(Clone, Debug)]
enum State {
    Welcome((bool, usize)), // (dirty, selected_game)
    Running(usize),
    GameOver(bool), // dirty
}

unsafe impl Send for World {}

pub struct World {
    games: Vec<Box<dyn Game>>,
    game_names: Vec<&'static str>,
    state: State,
    width: usize,
    height: usize,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            games: Vec::new(),
            game_names: Vec::new(),
            state: State::Welcome((true, 0)),
            width,
            height,
        }
    }

    pub fn add_game(&mut self, mut game: Box<dyn Game>, name: &'static str) {
        game.reset(self.width, self.height);
        self.games.push(game);
        self.game_names.push(name);
    }

    pub fn on_keypress(&mut self, key: DecodedKey, display: &mut Display) {
        match self.state {
            State::Running(i) if key == DecodedKey::Unicode('r') => {
                self.state = State::Welcome((true, 0));
            }
            State::Running(i) => {
                let w = &mut self.games[i];
                w.on_keypress(key);
            }
            State::Welcome((_, selected_game)) => {
                match key {
                    DecodedKey::Unicode('q') => {
                        // TODO: how to quit in no_std?
                    }
                    DecodedKey::Unicode('\n') | DecodedKey::RawKey(KeyCode::Enter) => {
                        display.clear();
                        self.state = State::Running(selected_game);
                        let w = &mut self.games[selected_game];
                        w.reset(self.width, self.height);
                    }
                    DecodedKey::RawKey(KeyCode::ArrowDown) => {
                        self.state = State::Welcome((true, (selected_game + 1) % self.games.len()));
                    }
                    DecodedKey::RawKey(KeyCode::ArrowUp) => {
                        let game = if selected_game == 0 {
                            self.games.len() - 1
                        } else {
                            selected_game - 1
                        };

                        self.state = State::Welcome((true, game));
                    }
                    _ => {}
                };
            }
            State::GameOver(_) => {
                match key {
                    DecodedKey::Unicode('r') => {
                        self.state = State::Welcome((true, 0));
                    }
                    _ => {}
                };
            }
        }
    }

    pub fn on_tick(&mut self, display: &mut Display) {
        match self.state {
            State::Running(i) => {
                let game = &mut self.games[i];
                if game.step() == GameState::GameOver {
                    self.state = State::GameOver(true);
                    return;
                }
                game.draw(display);
            }
            State::Welcome((dirty, selected_game)) => {
                if dirty {
                    self.draw_welcome(display);
                    self.state = State::Welcome((false, selected_game));
                }
            }
            State::GameOver(dirty) => {
                if dirty {
                    self.draw_game_over(display);
                    self.state = State::GameOver(false);
                }
            }
        }
    }

    fn draw_game_over(&mut self, display: &mut Display) {
        display.clear();
        let msg = "GAME OVER";
        display.set_xy(
            display.info.unwrap().horizontal_resolution / 2 - msg.len() * 8 / 2,
            display.info.unwrap().vertical_resolution / 2,
        );
        display.write_str(msg).unwrap();
        let msg = "(press 'r' to restart)";
        display.set_xy(
            display.info.unwrap().horizontal_resolution / 2 - msg.len() * 8 / 2,
            display.info.unwrap().vertical_resolution / 2 + 10,
        );
        display.write_str(msg);
        // serial_println!("GAME OVER");
    }

    pub fn draw_welcome(&mut self, display: &mut Display) {
        let (w, h) = (
            display.info.unwrap().horizontal_resolution,
            display.info.unwrap().vertical_resolution,
        );
        display.clear();
        display.draw_borders();

        let mut y_pos = h / 2 - 30;

        let msg = "<=== Welcome to SnakeOS ===>";
        display.set_xy(w / 2 - ((msg.len() / 2) * 8), y_pos);
        write!(display, "{}", msg);
        y_pos += 40;

        let selected_game = match self.state {
            State::Welcome((_, v)) => v,
            _ => unreachable!(),
        };

        for (i, name) in self.game_names.iter().enumerate() {
            display.set_xy(w / 2 - 40, y_pos);
            if selected_game == i {
                write!(display, "*  {}", name);
            } else {
                write!(display, "   {}", name);
            }
            y_pos += 30;
        }

        let msg = "Up/Down to select game, then press enter to start";
        display.set_xy(w / 2 - ((msg.len() / 2) * 8), self.height - 80);
        write!(display, "{}", msg);
        y_pos += 20;

        let footer = "by trusch";
        display.set_xy(
            w - footer.len() * 8 - 3 * crate::display::BLOCK_SIZE,
            h - 4 * crate::display::BLOCK_SIZE,
        );
        write!(display, "{}", footer);

        for i in 1..10000 {
            display.write_block(0, 0, crate::Color::Black);
        }
    }
}

// Position on screen in pixels
// (0,0) -> left top corner
// (width,height) -> right bottom corner
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenPos {
    pub x: usize,
    pub y: usize,
}

impl ScreenPos {
    pub fn new(x: usize, y: usize) -> Self {
        ScreenPos { x, y }
    }

    pub fn up(&self, y_offset: usize) -> Self {
        Self {
            x: self.x,
            y: self.y - y_offset,
        }
    }

    pub fn down(&self, y_offset: usize) -> Self {
        Self {
            x: self.x,
            y: self.y + y_offset,
        }
    }

    pub fn left(&self, x_offset: usize) -> Self {
        Self {
            x: self.x - x_offset,
            y: self.y,
        }
    }

    pub fn right(&self, x_offset: usize) -> Self {
        Self {
            x: self.x + x_offset,
            y: self.y,
        }
    }

    pub fn center(&self) -> Self {
        Self {
            x: self.x / 2,
            y: self.y / 2,
        }
    }

    pub fn up_blocks(&self, n: usize, block_size: usize) -> Self {
        self.up(n * block_size)
    }

    pub fn down_blocks(&self, n: usize, block_size: usize) -> Self {
        self.down(n * block_size)
    }

    pub fn left_blocks(&self, n: usize, block_size: usize) -> Self {
        self.left(n * block_size)
    }

    pub fn right_blocks(&self, n: usize, block_size: usize) -> Self {
        self.right(n * block_size)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

pub struct CharsBuf<const T: usize> {
    data: [char; T],
    written_bytes: usize,
}

impl<const T: usize> CharsBuf<T> {
    pub fn new() -> Self {
        Self {
            data: [0 as char; T],
            written_bytes: 0,
        }
    }

    pub fn chars(&self) -> &[char] {
        &self.data[..self.written_bytes]
    }

    pub fn len(&self) -> usize {
        self.written_bytes
    }
}

impl<const T: usize> core::fmt::Write for CharsBuf<T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars().take(self.data.len() - self.written_bytes) {
            self.data[self.written_bytes] = c;
            self.written_bytes += 1
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chars_buf_truncate() {
        let mut n2 = CharsBuf::<2>::new();
        write!(n2, "{}", 1);
    }
}
