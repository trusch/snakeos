use crate::display::{Color, Display};
use crate::game2048::board;
use crate::serial_println;
use crate::world::{CharsBuf, Direction, Game, GameState, ScreenPos};
use core::fmt::Write;
use pc_keyboard::{DecodedKey, KeyCode};

const BOARD_SIZE: usize = 4;
const MARGIN_PIXELS: usize = 4; // pixels
const BOARDER_PIXELS: usize = 4;

pub struct World {
    board: board::Board,
    game_over: bool,
    width: usize,
    height: usize,
    tile_size: usize,
    boarder_drawn: bool,
    result_drawn: bool,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let tile_size = (core::cmp::min(width, height) * 8 / 10 - MARGIN_PIXELS * (BOARD_SIZE + 1))
            / BOARD_SIZE;

        // make sure tile size is even
        let tile_size = tile_size - tile_size % 2;

        Self {
            board: board::Board::new(BOARD_SIZE, BOARD_SIZE),
            game_over: false,
            width,
            height,
            tile_size,
            boarder_drawn: false,
            result_drawn: false,
        }
    }

    fn draw_tile(&self, tile: &board::Tile, display: &mut Display) {
        let center = ScreenPos::new(self.width, self.height).center();
        let off = (BOARD_SIZE * self.tile_size + (BOARD_SIZE + 1) * MARGIN_PIXELS) / 2;

        let left_top = center.left(off).up(off);

        let pos = left_top
            .right(MARGIN_PIXELS)
            .right_blocks(tile.col, self.tile_size + MARGIN_PIXELS)
            .down(MARGIN_PIXELS)
            .down_blocks(tile.row, self.tile_size + MARGIN_PIXELS);

        let color = tile_color2(tile);
        display.draw_rect(pos.x, pos.y, self.tile_size, self.tile_size, color);

        if let Some(val) = &tile.val {
            let mut num = CharsBuf::<4>::new();
            write!(num, "{}", val);

            let x = pos.x + (self.tile_size - 8 * num.len()) / 2;
            let y = pos.y + (self.tile_size - 8) / 2;
            display.set_xy(x, y);
            for c in num.chars() {
                display.write_char_colored(*c, Color::White, color);
            }
        }
    }

    fn draw_boarder(&self, display: &mut Display) {
        let color = Color::RGB32(0xeee4da);

        let half = BOARD_SIZE / 2;
        let boarder_len =
            BOARD_SIZE * self.tile_size + (BOARD_SIZE + 1) * MARGIN_PIXELS + BOARDER_PIXELS * 2;
        let off = boarder_len / 2;

        let center = ScreenPos::new(self.width, self.height).center();
        let left_top = center.left(off).up(off);
        display.draw_rect(left_top.x, left_top.y, boarder_len, MARGIN_PIXELS, color);
        display.draw_rect(left_top.x, left_top.y, MARGIN_PIXELS, boarder_len, color);

        let right_top = center.right(off).up(off).left(MARGIN_PIXELS);
        display.draw_rect(right_top.x, right_top.y, MARGIN_PIXELS, boarder_len, color);

        let left_bot = center.left(off).down(off).up(MARGIN_PIXELS);
        display.draw_rect(left_bot.x, left_bot.y, boarder_len, MARGIN_PIXELS, color);

        serial_println!("BOARDER: {}", boarder_len);
        serial_println!("BOARDER: left_top: {:?}", left_top);
        serial_println!("BOARDER: right_top: {:?}", right_top);
        serial_println!("BOARDER: left_bot: {:?}", left_bot);
    }
}

// color scheme from https://play2048.co/
fn tile_color(tile: &board::Tile) -> Color {
    match tile.val {
        None => Color::White,
        Some(2) => Color::RGB32(0xeee4da),
        Some(4) => Color::RGB32(0xeee1c9),
        Some(8) => Color::RGB32(0xf3b27a),
        Some(16) => Color::RGB32(0xf69664),
        Some(32) => Color::RGB32(0xf77c5f),
        Some(64) => Color::RGB32(0xff75f3b),
        Some(128) => Color::RGB32(0xedd073),
        Some(256) => Color::RGB32(0xedcc62),
        Some(512) => Color::RGB32(0xedc950),
        Some(1024) => Color::RGB32(0xedc53f),
        Some(2048) => Color::RGB32(0xedc22e),
        _ => Color::Red,
    }
}

// color scheme from https://github.com/dev-family/wasm-204://github.com/dev-family/wasm-2048
fn tile_color2(tile: &board::Tile) -> Color {
    match tile.val {
        None => Color::RGB32(0x323846),
        Some(2) => Color::RGB32(0xe91e63),
        Some(4) => Color::RGB32(0xe91e1f),
        Some(8) => Color::RGB32(0xe9601e),
        Some(16) => Color::RGB32(0x2196f3),
        Some(32) => Color::RGB32(0x2150f3),
        Some(64) => Color::RGB32(0x3821f3),
        Some(128) => Color::RGB32(0x4caf50),
        Some(256) => Color::RGB32(0x4caf71),
        Some(512) => Color::RGB32(0x4caf92),
        Some(1024) => Color::RGB32(0xff9800),
        Some(2048) => Color::RGB32(0xffad00),
        _ => Color::Red,
    }
}

impl Game for World {
    fn reset(&mut self, width: usize, height: usize) {
        self.board.reset();

        self.board.random_fill_empty_tile();
        self.board.random_fill_empty_tile();
        self.game_over = false;
        self.result_drawn = false;
        self.boarder_drawn = false;
    }

    // step moves the snake one step forward
    fn step(&mut self) -> GameState {
        if self.game_over {
            return GameState::Live;
        }

        if self.board.is_game_over() || self.board.max_val() == 2048 {
            self.game_over = true;
            return GameState::Live;
        }

        GameState::Live
    }

    fn on_keypress(&mut self, key: DecodedKey) {
        if self.game_over || self.board.has_changed() {
            return;
        }

        let direction = match key {
            DecodedKey::RawKey(KeyCode::ArrowLeft) => Some(Direction::Left),
            DecodedKey::RawKey(KeyCode::ArrowRight) => Some(Direction::Right),
            DecodedKey::RawKey(KeyCode::ArrowUp) => Some(Direction::Up),
            DecodedKey::RawKey(KeyCode::ArrowDown) => Some(Direction::Down),
            _ => None,
        };

        if direction.is_some() {
            if self.board.move_direction(direction.unwrap()) {
                self.board.random_fill_empty_tile();
            }

            self.game_over = self.board.is_game_over();
        }
    }

    fn draw(&mut self, display: &mut Display) {
        /*
        if !self.boarder_drawn {
            self.draw_boarder(display);
            self.boarder_drawn = true;
        }
        */

        if self.game_over {
            if !self.result_drawn {
                let mut result = CharsBuf::<128>::new();
                let score = self.board.max_val();
                if score >= 2048 {
                    write!(result, "Congratulation! Press 'r' to restart.").unwrap();
                } else {
                    write!(result, "Game over! Press 'r' to restart.").unwrap();
                }

                let x = (self.width - 8 * result.len()) / 2;
                let y = self.height - self.height / 10;

                display.set_xy(x, y);
                for c in result.chars() {
                    display.write_char_colored(*c, Color::White, Color::Black);
                }
                self.result_drawn = true;
            }
        }

        for tile in self.board.tiles_need_redraw() {
            self.draw_tile(tile, display);
        }

        self.board.clear_changed();
    }
}
