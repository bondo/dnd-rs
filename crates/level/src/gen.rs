use rand::prelude::*;
use std::fmt::Debug;

use super::grid::{Grid, GridPos};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GenFloor {
    Empty,
    Treasure,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GenCell {
    Any,
    Floor(GenFloor),
    Wall,
}

impl Debug for GenCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            GenCell::Any => '?',
            GenCell::Wall => '#',
            GenCell::Floor(GenFloor::Empty) => '.',
            GenCell::Floor(GenFloor::Treasure) => 'T',
        };
        write!(f, "{}", c)
    }
}

pub(super) type GenLevel = Grid<GenCell>;

impl GenLevel {
    pub fn random(width: usize, height: usize) -> Self {
        assert!(width >= 6);
        assert!(height >= 6);

        let mut grid = Grid::new(width, height, GenCell::Any);
        let mut rng = rand::thread_rng();

        let treasure_room_x = rng.gen::<usize>() % (width - 2);
        let treasure_room_y = rng.gen::<usize>() % (height - 2);

        // Fill treasure room with floor
        (treasure_room_x..treasure_room_x + 3).for_each(|x| {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                grid[(x, y).into()] = GenCell::Floor(GenFloor::Empty);
            });
        });

        let treasure_x = rng.gen::<usize>() % 3 + treasure_room_x;
        let treasure_y = rng.gen::<usize>() % 3 + treasure_room_y;
        grid[(treasure_x, treasure_y).into()] = GenCell::Floor(GenFloor::Treasure);

        let mut potential_exits: Vec<GridPos> = Vec::new();

        // Fill in walls around treasure room

        // Left side
        if treasure_room_x > 0 {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                let pos = (treasure_room_x - 1, y).into();
                grid[pos] = GenCell::Wall;
                if treasure_room_x > 1 {
                    potential_exits.push(pos);
                }
            })
        }

        // Right side
        if treasure_room_x + 3 < width {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                let pos = (treasure_room_x + 3, y).into();
                grid[pos] = GenCell::Wall;
                if treasure_room_x + 4 < width {
                    potential_exits.push(pos);
                }
            })
        }

        // Top side
        if treasure_room_y > 0 {
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                let pos = (x, treasure_room_y - 1).into();
                grid[pos] = GenCell::Wall;
                if treasure_room_y > 1 {
                    potential_exits.push(pos);
                }
            })
        }

        // Bottom side
        if treasure_room_y + 3 < height {
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                let pos = (x, treasure_room_y + 3).into();
                grid[pos] = GenCell::Wall;
                if treasure_room_y + 4 < height {
                    potential_exits.push(pos);
                }
            })
        }

        // Top left corner
        if treasure_room_x > 0 && treasure_room_y > 0 {
            grid[(treasure_room_x - 1, treasure_room_y - 1).into()] = GenCell::Wall;
        }

        // Top right corner
        if treasure_room_x + 3 < width && treasure_room_y > 0 {
            grid[(treasure_room_x + 3, treasure_room_y - 1).into()] = GenCell::Wall;
        }

        // Bottom left corner
        if treasure_room_x > 0 && treasure_room_y + 3 < height {
            grid[(treasure_room_x - 1, treasure_room_y + 3).into()] = GenCell::Wall;
        }

        // Bottom right corner
        if treasure_room_x + 3 < width && treasure_room_y + 3 < height {
            grid[(treasure_room_x + 3, treasure_room_y + 3).into()] = GenCell::Wall;
        }

        let exit = potential_exits[rng.gen_range(0..potential_exits.len())];
        grid[exit] = GenCell::Any;

        let mut work_queue = WorkQueue::new(rng);
        work_queue.extend_one(exit);

        while let Some(p) = work_queue.next() {
            if grid[p] != GenCell::Any {
                continue;
            }

            if grid.count_neighbors(p, |n| matches!(n, GenCell::Floor(_))) != 1 {
                continue;
            }

            grid[p] = GenCell::Floor(GenFloor::Empty);

            work_queue.extend(grid.filtered_neighbors(p, |n| n == &GenCell::Any));
        }

        grid
    }
}

struct WorkQueue {
    rng: ThreadRng,
    vec: Vec<GridPos>,
}

impl WorkQueue {
    fn new(rng: ThreadRng) -> Self {
        Self {
            rng,
            vec: Vec::new(),
        }
    }

    fn extend(&mut self, mut ps: Vec<GridPos>) {
        self.vec.append(&mut ps);
    }

    fn extend_one(&mut self, p: GridPos) {
        self.vec.push(p);
    }

    fn next(&mut self) -> Option<GridPos> {
        if self.vec.is_empty() {
            return None;
        }

        let idx = self.rng.gen_range(0..self.vec.len());
        let p = self.vec.swap_remove(idx);
        Some(p)
    }
}
