use rand::prelude::*;
use std::fmt::Display;

const SIZE: usize = 9;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Floor {
    Empty,
    Treasure,
    Monster,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Any,
    Wall,
    Floor(Floor),
}

pub struct Level {
    cells: [[Cell; SIZE]; SIZE],
}

impl Level {
    pub fn new() -> Self {
        let mut cells = [[Cell::Any; SIZE]; SIZE];
        let mut rng = rand::thread_rng();

        let treasure_room_x = rng.gen::<usize>() % (SIZE - 2);
        let treasure_room_y = rng.gen::<usize>() % (SIZE - 2);

        // Fill treasure room with floor
        (treasure_room_x..treasure_room_x + 3).for_each(|x| {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                cells[x][y] = Cell::Floor(Floor::Empty);
            });
        });

        let treasure_x = rng.gen::<usize>() % 3 + treasure_room_x;
        let treasure_y = rng.gen::<usize>() % 3 + treasure_room_y;
        cells[treasure_x][treasure_y] = Cell::Floor(Floor::Treasure);

        let mut potential_exits: Vec<(usize, usize)> = Vec::new();

        // Fill in walls around treasure room

        // Left side
        if treasure_room_x > 0 {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                cells[treasure_room_x - 1][y] = Cell::Wall;
                if treasure_room_x > 1 {
                    potential_exits.push((treasure_room_x - 1, y));
                }
            })
        }

        // Right side
        if treasure_room_x + 3 < SIZE {
            (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                cells[treasure_room_x + 3][y] = Cell::Wall;
                if treasure_room_x + 4 < SIZE {
                    potential_exits.push((treasure_room_x + 3, y));
                }
            })
        }

        // Top side
        if treasure_room_y > 0 {
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                cells[x][treasure_room_y - 1] = Cell::Wall;
                if treasure_room_y > 1 {
                    potential_exits.push((x, treasure_room_y - 1));
                }
            })
        }

        // Bottom side
        if treasure_room_y + 3 < SIZE {
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                cells[x][treasure_room_y + 3] = Cell::Wall;
                if treasure_room_y + 4 < SIZE {
                    potential_exits.push((x, treasure_room_y + 3));
                }
            })
        }

        // Top left corner
        if treasure_room_x > 0 && treasure_room_y > 0 {
            cells[treasure_room_x - 1][treasure_room_y - 1] = Cell::Wall;
        }

        // Top right corner
        if treasure_room_x + 3 < SIZE && treasure_room_y > 0 {
            cells[treasure_room_x + 3][treasure_room_y - 1] = Cell::Wall;
        }

        // Bottom left corner
        if treasure_room_x > 0 && treasure_room_y + 3 < SIZE {
            cells[treasure_room_x - 1][treasure_room_y + 3] = Cell::Wall;
        }

        // Bottom right corner
        if treasure_room_x + 3 < SIZE && treasure_room_y + 3 < SIZE {
            cells[treasure_room_x + 3][treasure_room_y + 3] = Cell::Wall;
        }

        let (exit_x, exit_y) = potential_exits[rng.gen_range(0..potential_exits.len())];
        cells[exit_x][exit_y] = Cell::Any;

        let mut expand_from = vec![(exit_x, exit_y)];
        loop {
            if expand_from.is_empty() {
                break;
            }

            // Pick a random cell to expand from
            let (x, y) = expand_from.swap_remove(rng.gen_range(0..expand_from.len()));

            if cells[x][y] != Cell::Any {
                continue;
            }

            if Level::filtered_neighbours(x, y, |nx, ny| matches!(cells[nx][ny], Cell::Floor(_)))
                .len()
                != 1
            {
                continue;
            }

            cells[x][y] = Cell::Floor(Floor::Empty);

            expand_from.append(&mut Level::filtered_neighbours(x, y, |nx, ny| {
                cells[nx][ny] == Cell::Any
            }));
        }

        // Replace remaining Any cells with walls and place monsters
        (0..SIZE).for_each(|x| {
            (0..SIZE).for_each(|y| match cells[x][y] {
                Cell::Any => {
                    cells[x][y] = Cell::Wall;
                }
                Cell::Floor(Floor::Empty) => {
                    let num_floor_neighbours = Level::filtered_neighbours(x, y, |nx, ny| {
                        matches!(cells[nx][ny], Cell::Floor(_))
                    })
                    .len();

                    assert_ne!(
                        num_floor_neighbours, 0,
                        "no empty floor should be without neighbours"
                    );

                    if num_floor_neighbours == 1 {
                        cells[x][y] = Cell::Floor(Floor::Monster);
                    }
                }
                _ => {}
            })
        });
        Level { cells }
    }

    fn filtered_neighbours(
        x: usize,
        y: usize,
        mut f: impl FnMut(usize, usize) -> bool,
    ) -> Vec<(usize, usize)> {
        [
            if y > 0 { Some((x, y - 1)) } else { None },
            if y + 1 < SIZE { Some((x, y + 1)) } else { None },
            if x > 0 { Some((x - 1, y)) } else { None },
            if x + 1 < SIZE { Some((x + 1, y)) } else { None },
        ]
        .into_iter()
        .flatten()
        .filter(|(x, y)| f(*x, *y))
        .collect::<Vec<_>>()
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..SIZE {
            for x in 0..SIZE {
                let cell = match self.cells[x][y] {
                    Cell::Any => '?',
                    Cell::Wall => '#',
                    Cell::Floor(Floor::Empty) => '.',
                    Cell::Floor(Floor::Treasure) => 'T',
                    Cell::Floor(Floor::Monster) => 'M',
                };
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
