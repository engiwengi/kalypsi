use std::ops::Range;

use leptos::leptos_dom::console_log;
use nanorand::{Rng, WyRand};

use crate::{find_runs, BoolMatrix, TriBoolMatrix};

pub struct GridGenerator {
    symmetry: Symmetry,
    width: usize,
    cells: Vec<Cell>,
    rand: WyRand,
}

impl GridGenerator {
    pub fn cells(self) -> Vec<bool> {
        self.cells
            .into_iter()
            .map(|c| matches!(c, Cell::Black))
            .collect()
    }

    pub fn new<M>(matrix: M) -> Self
    where
        M: TriBoolMatrix,
    {
        let width = matrix.cols();
        let height = matrix.rows();

        let mut new = Self {
            symmetry: Symmetry::Point,
            width: matrix.cols(),
            cells: vec![Cell::default(); width * height],
            rand: WyRand::new_seed(rand::random::<u64>()),
        };

        for y in -1..=height as isize {
            for x in -1..=width as isize {
                if y < 0 || x < 0 || x == width as isize || y == width as isize {
                    new.block((x, y), 3);
                }
            }
        }

        for y in 0..height as isize {
            for x in 0..width as isize {
                let maybe_at = matrix.maybe_at((x as usize, y as usize));

                if let Some(false) = maybe_at {
                    new.reserve((x, y));
                } else if maybe_at.is_none() {
                    new.block((x, y), 3);
                }
            }
        }

        new
    }

    // TODO change to be a target average word len
    pub fn place_blacks(
        &mut self,
        target_avg_word_len: Range<f32>,
        target_word_count: Range<usize>,
    ) {
        let start = self.cells.clone();
        let mut best = None;
        let mut best_avg_len = 0.0;
        let mut best_avg_len_diff = f32::MAX;
        let mut best_word_count = 0;
        let mut best_word_count_diff = usize::MAX;
        let max_iters = 10000;
        let mut iter = 0;

        while !target_word_count.contains(&best_word_count)
            || !target_avg_word_len.contains(&best_avg_len)
        {
            if iter > max_iters {
                break;
            }

            let mut avg_len = f32::MAX;
            let mut current_word_count = 0;

            iter += 1;
            while !(target_avg_word_len.contains(&avg_len)
                && target_word_count.contains(&current_word_count))
                && self.choose_next_black().is_ok()
            {
                let runs = find_runs(&*self);
                avg_len = runs.iter().fold(0, |acc, v| acc + v.2) as f32 / runs.len() as f32;
                current_word_count = runs.len();
            }

            let diff = if target_avg_word_len.contains(&avg_len) {
                0.0
            } else {
                (target_avg_word_len.start - avg_len)
                    .abs()
                    .min((target_avg_word_len.end - avg_len).abs())
            };

            let word_count_diff = if target_word_count.contains(&current_word_count) {
                0
            } else {
                target_word_count
                    .start
                    .abs_diff(current_word_count)
                    .min(target_word_count.end.abs_diff(current_word_count))
            };

            if diff <= best_avg_len_diff && word_count_diff <= best_word_count_diff {
                best = Some(std::mem::take(&mut self.cells));
                best_avg_len = avg_len;
                best_avg_len_diff = diff;
                best_word_count = current_word_count;
                best_word_count_diff = word_count_diff;
            }

            self.cells = start.clone();
        }

        console_log(&format!("iter: {}", iter));

        self.cells = best.unwrap_or(start);
    }

    fn choose_next_black(&mut self) -> Result<(), ()> {
        let mut runs = find_runs(&*self);
        runs.sort_unstable_by_key(|v| v.2);

        while !runs.is_empty() {
            let next_run = self.rand.generate_range(0..runs.len());
            // let next_run = runs.remove(next_run);
            let next_run = runs.pop().unwrap();

            let mut placeables = Vec::with_capacity(next_run.2);
            for i in 0..next_run.2 as isize {
                let coord = if next_run.3 {
                    (next_run.0 as isize + i, next_run.1 as isize)
                } else {
                    (next_run.0 as isize, next_run.1 as isize + i)
                };

                let coords = self.reflect(coord);
                let mut near = false;
                for coord in coords.iter() {
                    let has = coords
                        .iter()
                        .filter(|c| !(c.0 == coord.0 && c.1 == coord.1))
                        .any(|c| c.0 == coord.0 && c.1.abs_diff(coord.1) < 3);
                    let has2 = coords
                        .iter()
                        .filter(|c| !(c.0 == coord.0 && c.1 == coord.1))
                        .any(|c| c.1 == coord.1 && c.0.abs_diff(coord.0) < 3);
                    near = near || has || has2;
                }

                if !near
                    && coords
                        .iter()
                        .filter_map(|&c| self.get(c))
                        .all(|c| c.can_place())
                {
                    placeables.push(coord);
                }
            }

            let min_len = self.rand.generate_range(0..4);

            if placeables.len() <= min_len {
                // console_log("continuing");
                continue;
            }

            let next = self.rand.generate_range(0..placeables.len());

            let coord = placeables.into_iter().nth(next).unwrap();

            for coord in self.reflect(coord) {
                // console_log(&format!("placing at {:?}", coord));
                self.block(coord, 3);
            }
            return Ok(());
        }

        Err(())

        // let mut placeables = Vec::with_capacity(self.cells.len());
        // for y in 0..self.height() as isize {
        //     for x in 0..self.width as isize {
        //         let coord = (x, y);
        //         let coords = self.reflect(coord);
        //         let mut near = false;
        //         for coord in coords.iter() {
        //             let has = coords
        //                 .iter()
        //                 .filter(|c| !(c.0 == coord.0 && c.1 == coord.1))
        //                 .any(|c| c.0 == coord.0 && c.1.abs_diff(coord.1) <
        // 3);             let has2 = coords
        //                 .iter()
        //                 .filter(|c| !(c.0 == coord.0 && c.1 == coord.1))
        //                 .any(|c| c.1 == coord.1 && c.0.abs_diff(coord.0) <
        // 3);             near = near || has || has2;
        //         }
        //
        //         if !near
        //             && coords
        //                 .iter()
        //                 .filter_map(|&c| self.get(c))
        //                 .all(|c| c.can_place())
        //         {
        //             placeables.push(coord);
        //         }
        //     }
        // }
        //
        // if placeables.is_empty() {
        //     return Err(());
        // }
        //
        // // console_log(&format!("{:?}", placeables));
        //
        // // placeables.sort_unstable_by_key(|&(x, y)| {
        // //     (self.width / 2 - x.abs_diff(self.width as isize / 2))
        // //         .max(self.height() / 2 - y.abs_diff(self.height() as isize
        // / 2)) // });
        //
        // let next = self.rand.generate_range(0..placeables.len());
        //
        // let coord = placeables.into_iter().nth(next).unwrap();
        //
        // for coord in self.reflect(coord) {
        //     // console_log(&format!("placing at {:?}", coord));
        //     self.block(coord, 3);
        // }
        // Ok(())
    }

    fn calculate_coord(&self, index: usize) -> (isize, isize) {
        ((index % self.width) as isize, (index / self.width) as isize)
    }

    fn block(&mut self, coord: (isize, isize), n: isize) {
        if let Some(cell) = self.get_mut(coord) {
            cell.place();
        }

        for dir in Direction::iter() {
            self.block_in_dir(coord, dir, n);
        }
    }

    fn reserve(&mut self, coord: (isize, isize)) {
        if let Some(Cell::White { reserved, .. }) = self.get_mut(coord) {
            *reserved = true;
        }
    }

    // todo turn into iterator
    fn reflect(&self, coord: (isize, isize)) -> Vec<(isize, isize)> {
        match self.symmetry {
            Symmetry::Quarter => vec![
                coord,
                (
                    self.width as isize - coord.0 - 1,
                    self.height() as isize - coord.1 - 1,
                ),
                (coord.0, self.height() as isize - coord.1 - 1),
                (self.width as isize - coord.0 - 1, coord.1),
            ],
            Symmetry::Point => vec![
                coord,
                (
                    self.width as isize - coord.0 - 1,
                    self.height() as isize - coord.1 - 1,
                ),
            ],
            Symmetry::None => vec![coord],
        }
    }

    fn height(&self) -> usize {
        self.cells.len() / self.width
    }

    fn block_in_dir(&mut self, coord: (isize, isize), dir: Direction, n: isize) {
        for i in 1..=n {
            let coord = match dir {
                Direction::Left => (coord.0 - i, coord.1),
                Direction::Right => (coord.0 + i, coord.1),
                Direction::Down => (coord.0, coord.1 + i),
                Direction::Up => (coord.0, coord.1 - i),
            };

            if let Some(cell) = self.get_mut(coord) {
                cell.set(dir, i == 1);
            }
        }
    }

    fn get(&self, coord: (isize, isize)) -> Option<&Cell> {
        if coord.0 < 0 || coord.1 < 0 {
            return None;
        }

        self.cells
            .chunks_exact(self.width)
            .nth(coord.1 as usize)
            .and_then(|column| column.get(coord.0 as usize))
    }

    fn get_mut(&mut self, coord: (isize, isize)) -> Option<&mut Cell> {
        if coord.0 < 0 || coord.1 < 0 {
            return None;
        }

        self.cells
            .chunks_exact_mut(self.width)
            .nth(coord.1 as usize)
            .and_then(|column| column.get_mut(coord.0 as usize))
    }
}

impl<'a> BoolMatrix for &'a GridGenerator {
    fn rows(self) -> usize {
        self.height()
    }

    fn cols(self) -> usize {
        self.width
    }

    fn at(self, (x, y): (usize, usize)) -> bool {
        self.get((x as isize, y as isize))
            .map_or(false, |cell| matches!(cell, Cell::White { .. }))
    }
}

#[derive(Clone, Copy)]
enum Cell {
    White {
        reserved: bool,
        left: bool,
        right: bool,
        up: bool,
        down: bool,
    },
    Black,
}

impl Default for Cell {
    fn default() -> Self {
        Self::White {
            left: true,
            right: true,
            up: true,
            down: true,
            reserved: false,
        }
    }
}

impl Cell {
    fn can_place(&self) -> bool {
        match self {
            Cell::White {
                left,
                right,
                up,
                down,
                reserved,
            } => !*reserved && *left && *right && *up && *down,
            Cell::Black => false,
        }
    }

    fn set(&mut self, dir: Direction, val: bool) {
        match (self, dir) {
            (Cell::White { left, .. }, Direction::Left) => *left = val,
            (Cell::White { right, .. }, Direction::Right) => *right = val,
            (Cell::White { down, .. }, Direction::Down) => *down = val,
            (Cell::White { up, .. }, Direction::Up) => *up = val,
            (Cell::Black, _) => (),
        }
    }

    fn place(&mut self) {
        *self = Self::Black;
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    fn iter() -> DirectionIter {
        DirectionIter::default()
    }
}

#[derive(Default, Clone, Copy)]
struct DirectionIter(Option<Direction>);

impl Iterator for DirectionIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 = match self.0 {
            None => Some(Direction::Left),
            Some(Direction::Left) => Some(Direction::Right),
            Some(Direction::Right) => Some(Direction::Down),
            Some(Direction::Down) => Some(Direction::Up),
            Some(Direction::Up) => None,
        };

        self.0
    }
}

#[derive(Clone, Copy)]
enum Symmetry {
    Point,
    Quarter,
    None,
}

impl Symmetry {
    fn rotate(&self, coord: (isize, isize), width: usize, height: isize) -> Option<(isize, isize)> {
        match self {
            Symmetry::Point => Some((width as isize - coord.0 - 1, height - coord.1 - 1)),
            Symmetry::Quarter => Some((width as isize - coord.0 - 1, height - coord.1 - 1)),
            Symmetry::None => None,
        }
    }
}
// pub struct SymmetryIter {
//     symmetry: Symmetry,
//     next: Option<(isize, isize)>,
//     iteration: usize,
//     width: usize,
//     height: usize,
// }
//
// impl Iterator for SymmetryIter {
//     type Item = (isize, isize);
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(next) = self.next {}
//
//         self.iteration += 1;
//
//         next
//     }
// }
