use crate::world::Direction;
use alloc::vec::Vec;
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct World {}

#[derive(Debug)]
pub struct Tile {
    pub val: Option<u64>,
    pub changed: bool,
    pub row: usize,
    pub col: usize,
}

impl Tile {
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            val: None,
            changed: false,
            row,
            col,
        }
    }

    fn can_merge(&self, other: &Self) -> bool {
        if let (Some(ref v1), Some(ref v2)) = (self.val, other.val) {
            if v1 == v2 {
                return true;
            }
        }
        false
    }
}

pub(crate) struct Board {
    rows: usize,
    cols: usize,
    tiles: Vec<Tile>,
    rng: rand::rngs::SmallRng,
}

impl Board {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut tiles = Vec::with_capacity(rows * cols);

        for row in 0..rows {
            for col in 0..cols {
                tiles.push(Tile::new(row, col));
            }
        }

        let seed = unsafe { core::arch::x86_64::_rdtsc() };

        Self {
            rows,
            cols,
            tiles,
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }

    pub fn reset(&mut self) {
        for t in self.tiles.iter_mut() {
            t.val = None;
            t.changed = true;
        }
    }

    pub fn has_changed(&self) -> bool {
        self.tiles.iter().any(|v| v.changed)
    }

    pub fn tiles_need_redraw<'a>(&'a self) -> impl Iterator<Item = &'a Tile> {
        self.tiles.iter().filter(|v| v.changed)
    }

    pub fn clear_changed(&mut self) {
        self.tiles.iter_mut().for_each(|v| v.changed = false)
    }

    pub fn random_fill_empty_tile(&mut self) -> bool {
        let empty_tiles = self.tiles.iter().filter(|v| v.val.is_none()).count();
        if empty_tiles == 0 {
            return false;
        }

        let random = (self.rng.next_u64() as usize) % empty_tiles;
        let to_fill = self
            .tiles
            .iter_mut()
            .filter(|v| v.val.is_none())
            .nth(random)
            .unwrap();
        assert!(to_fill.val.is_none());

        // In the original game there is a 10% chance that this is a 4
        // reference: https://github.com/gabrielecirulli/2048/blob/fc1ef4fe5a5fcccea7590f3e4c187c75980b353f/js/game_manager.js#L71
        let fill_val = if self.rng.next_u64() % 10 == 0 { 4 } else { 2 };

        to_fill.val = Some(fill_val);
        to_fill.changed = true;
        true
    }

    pub fn is_game_over(&self) -> bool {
        if self.tiles.iter().any(|v| v.val.is_none()) {
            return false;
        }

        if self.can_merge_any() {
            return false;
        }

        true
    }

    fn can_merge_any(&self) -> bool {
        for dir in [
            Direction::Up,
            Direction::Down,
            Direction::Right,
            Direction::Left,
        ] {
            let (dim_x, dim_y) = self.get_dimension_from_direction(dir);
            for x in 0..dim_x {
                for y in 0..dim_y - 1 {
                    let cur = self.get_tile(dir, x, y);
                    let next = self.get_tile(dir, x, y + 1);
                    if cur.can_merge(next) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn pos(&self, dir: Direction, x: usize, y: usize) -> usize {
        match dir {
            Direction::Up => x + self.cols * y,
            Direction::Down => x + self.cols * (self.rows - y - 1),
            Direction::Left => x * self.cols + y,
            Direction::Right => x * self.cols + (self.cols - y - 1),
        }
    }

    fn get_tile(&self, dir: Direction, x: usize, y: usize) -> &Tile {
        let idx = self.pos(dir, x, y);
        &self.tiles[idx]
    }

    fn get_tile_mut(&mut self, dir: Direction, x: usize, y: usize) -> &mut Tile {
        let idx = self.pos(dir, x, y);
        &mut self.tiles[idx]
    }

    fn get_dimension_from_direction(&self, dir: Direction) -> (usize, usize) {
        match dir {
            Direction::Up | Direction::Down => (self.cols, self.rows),
            Direction::Left | Direction::Right => (self.rows, self.cols),
        }
    }

    pub fn max_val(&self) -> u64 {
        if let Some(max) = self.tiles.iter().map(|v| v.val.unwrap_or(0)).max() {
            max
        } else {
            0
        }
    }

    pub fn move_direction(&mut self, dir: Direction) -> bool {
        let mut changed = false;
        let (x_dim, y_dim) = self.get_dimension_from_direction(dir);
        for x in 0..x_dim {
            let mut has_merged_one = false;
            for y in 0..y_dim {
                let cur = &self.get_tile(dir, x, y);
                if cur.val.is_none() {
                    continue;
                }
                // println!("move tile: ({}, {}) => {:?}", x, y, cur);

                let mut destination = y;
                for y2 in (0..y).rev() {
                    let t = &self.tiles[self.pos(dir, x, y2)];
                    if t.val.is_none() {
                        destination = y2;
                        continue;
                    } else {
                        if !has_merged_one && t.can_merge(cur) {
                            destination = y2;
                        }
                        break;
                    }
                }

                if destination == y {
                    continue;
                }

                let val = cur.val.clone();
                // cur.changed = true;
                changed = true;

                // println!("  ({} {}) -> ({} {})", x, y, x, destination);
                let destination = &mut self.get_tile_mut(dir, x, destination);
                if destination.val.is_none() {
                    destination.val = val;
                    destination.changed = true;
                } else {
                    destination.val = Some(val.unwrap() * 2);
                    destination.changed = true;
                    has_merged_one = true
                }
                // println!("  destination => {:?}", destination);

                let cur = &mut self.get_tile_mut(dir, x, y);
                cur.val = None;
                cur.changed = true;
                // println!("  self => {:?}", cur);
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos() {
        let board = Board::new(3, 4);

        assert_eq!(board.pos(Direction::Left, 1, 1), 5);
        assert_eq!(board.pos(Direction::Left, 2, 2), 10);

        assert_eq!(board.pos(Direction::Right, 1, 1), 6);
        assert_eq!(board.pos(Direction::Right, 2, 2), 9);

        assert_eq!(board.pos(Direction::Up, 3, 1), 7);
        assert_eq!(board.pos(Direction::Up, 2, 2), 10);

        assert_eq!(board.pos(Direction::Down, 3, 2), 3);
        assert_eq!(board.pos(Direction::Down, 3, 1), 7);
        assert_eq!(board.pos(Direction::Down, 2, 2), 2);
    }

    #[test]
    fn test_move_simple() {
        let mut board = Board::new(3, 4);

        board.get_tile_mut(Direction::Left, 0, 0).val = Some(2);
        board.get_tile_mut(Direction::Left, 1, 1).val = Some(4);
        board.get_tile_mut(Direction::Left, 2, 2).val = Some(8);

        board.move_direction(Direction::Left);

        assert_eq!(board.get_tile_mut(Direction::Left, 0, 0).val, Some(2));
        assert_eq!(board.get_tile_mut(Direction::Left, 1, 0).val, Some(4));
        assert_eq!(board.get_tile_mut(Direction::Left, 2, 0).val, Some(8));

        assert_eq!(board.get_tile_mut(Direction::Left, 1, 1).val, None);
        assert_eq!(board.get_tile_mut(Direction::Left, 2, 2).val, None);
    }

    #[test]
    fn test_move_mutiple() {
        let mut board = Board::new(3, 5);

        board.get_tile_mut(Direction::Right, 0, 0).val = None;
        board.get_tile_mut(Direction::Right, 0, 1).val = Some(2);
        board.get_tile_mut(Direction::Right, 0, 2).val = None;
        board.get_tile_mut(Direction::Right, 0, 3).val = Some(4);

        board.move_direction(Direction::Right);

        assert_eq!(board.get_tile_mut(Direction::Right, 0, 0).val, Some(2));
        assert_eq!(board.get_tile_mut(Direction::Right, 0, 1).val, Some(4));
        assert_eq!(board.get_tile_mut(Direction::Right, 0, 2).val, None);
        assert_eq!(board.get_tile_mut(Direction::Right, 0, 3).val, None);
        assert_eq!(board.get_tile_mut(Direction::Right, 0, 4).val, None);
    }

    #[test]
    fn test_merge() {
        let mut board = Board::new(5, 3);

        board.get_tile_mut(Direction::Up, 0, 0).val = None;
        board.get_tile_mut(Direction::Up, 0, 1).val = Some(2);
        board.get_tile_mut(Direction::Up, 0, 2).val = None;
        board.get_tile_mut(Direction::Up, 0, 3).val = Some(2);
        board.get_tile_mut(Direction::Up, 0, 4).val = Some(4);

        board.move_direction(Direction::Up);

        assert_eq!(board.get_tile_mut(Direction::Up, 0, 0).val, Some(4));
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 1).val, Some(4));
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 2).val, None);
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 3).val, None);
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 4).val, None);
    }

    #[test]
    fn test_not_merge() {
        let mut board = Board::new(5, 3);

        board.get_tile_mut(Direction::Up, 0, 0).val = None;
        board.get_tile_mut(Direction::Up, 0, 1).val = Some(2);
        board.get_tile_mut(Direction::Up, 0, 2).val = None;
        board.get_tile_mut(Direction::Up, 0, 3).val = Some(4);
        board.get_tile_mut(Direction::Up, 0, 4).val = Some(2);

        board.move_direction(Direction::Up);

        assert_eq!(board.get_tile_mut(Direction::Up, 0, 0).val, Some(2));
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 1).val, Some(4));
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 2).val, Some(2));
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 3).val, None);
        assert_eq!(board.get_tile_mut(Direction::Up, 0, 4).val, None);
    }
}
