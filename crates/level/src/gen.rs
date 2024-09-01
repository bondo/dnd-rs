use std::fmt::Debug;

use fastrand::Rng;

use super::grid::{Grid, GridPos};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenFloor {
    Empty,
    Treasure,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenCell {
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

pub(crate) type GenLevel = Grid<GenCell>;

impl GenLevel {
    pub fn random(width: usize, height: usize) -> Result<Self, &'static str> {
        if width < 6 || height < 6 {
            return Err("width and height must be at least 6");
        }

        let mut grid = Grid::new(width, height, GenCell::Any);
        let mut rng = Rng::new();

        let treasure_room_x = rng.usize(0..width - 2);
        let treasure_room_y = rng.usize(0..height - 2);

        // Fill treasure room with floor
        (treasure_room_x..treasure_room_x + 3).for_each(|x| {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                grid[(x, y).into()] = GenCell::Floor(GenFloor::Empty);
            });
        });

        let treasure_x = rng.usize(0..3) + treasure_room_x;
        let treasure_y = rng.usize(0..3) + treasure_room_y;
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

        let exit = potential_exits[rng.usize(0..potential_exits.len())];
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

        Ok(grid)
    }
}

struct WorkQueue {
    rng: Rng,
    vec: Vec<GridPos>,
}

impl WorkQueue {
    fn new(rng: Rng) -> Self {
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

        let idx = self.rng.usize(0..self.vec.len());
        let p = self.vec.swap_remove(idx);
        Some(p)
    }
}
