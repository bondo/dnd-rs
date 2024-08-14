use rand::prelude::*;
use std::fmt::Debug;

use super::grid::Grid;

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
                grid[(x, y)] = GenCell::Floor(GenFloor::Empty);
            });
        });

        let treasure_x = rng.gen::<usize>() % 3 + treasure_room_x;
        let treasure_y = rng.gen::<usize>() % 3 + treasure_room_y;
        grid[(treasure_x, treasure_y)] = GenCell::Floor(GenFloor::Treasure);

        let mut potential_exits: Vec<(usize, usize)> = Vec::new();

        // Fill in walls around treasure room

        // Left side
        if treasure_room_x > 0 {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                grid[(treasure_room_x - 1, y)] = GenCell::Wall;
                if treasure_room_x > 1 {
                    potential_exits.push((treasure_room_x - 1, y));
                }
            })
        }

        // Right side
        if treasure_room_x + 3 < width {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                grid[(treasure_room_x + 3, y)] = GenCell::Wall;
                if treasure_room_x + 4 < width {
                    potential_exits.push((treasure_room_x + 3, y));
                }
            })
        }

        // Top side
        if treasure_room_y > 0 {
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                grid[(x, treasure_room_y - 1)] = GenCell::Wall;
                if treasure_room_y > 1 {
                    potential_exits.push((x, treasure_room_y - 1));
                }
            })
        }

        // Bottom side
        if treasure_room_y + 3 < height {
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                grid[(x, treasure_room_y + 3)] = GenCell::Wall;
                if treasure_room_y + 4 < height {
                    potential_exits.push((x, treasure_room_y + 3));
                }
            })
        }

        // Top left corner
        if treasure_room_x > 0 && treasure_room_y > 0 {
            grid[(treasure_room_x - 1, treasure_room_y - 1)] = GenCell::Wall;
        }

        // Top right corner
        if treasure_room_x + 3 < width && treasure_room_y > 0 {
            grid[(treasure_room_x + 3, treasure_room_y - 1)] = GenCell::Wall;
        }

        // Bottom left corner
        if treasure_room_x > 0 && treasure_room_y + 3 < height {
            grid[(treasure_room_x - 1, treasure_room_y + 3)] = GenCell::Wall;
        }

        // Bottom right corner
        if treasure_room_x + 3 < width && treasure_room_y + 3 < height {
            grid[(treasure_room_x + 3, treasure_room_y + 3)] = GenCell::Wall;
        }

        let (exit_x, exit_y) = potential_exits[rng.gen_range(0..potential_exits.len())];
        grid[(exit_x, exit_y)] = GenCell::Any;

        let mut expand_from = vec![(exit_x, exit_y)];
        loop {
            if expand_from.is_empty() {
                break;
            }

            // Pick a random cell to expand from
            let p = expand_from.swap_remove(rng.gen_range(0..expand_from.len()));

            if grid[p] != GenCell::Any {
                continue;
            }

            if grid.count_neighbors(p, |n| matches!(n, GenCell::Floor(_))) != 1 {
                continue;
            }

            grid[p] = GenCell::Floor(GenFloor::Empty);

            expand_from.append(&mut grid.filtered_neighbors(p, |n| n == &GenCell::Any));
        }

        grid
    }
}
