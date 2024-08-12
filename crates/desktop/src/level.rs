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

        let (exit_x, exit_y) = potential_exits[rng.gen::<usize>() % potential_exits.len()];
        cells[exit_x][exit_y] = Cell::Floor(Floor::Empty);

        let mut expand_from = vec![(exit_x, exit_y)];
        while let Some((x, y)) = expand_from.pop() {
            let mut free_neighbours: Vec<(usize, usize)> = Vec::new();

            let top_left_good = x == 0 || y == 0 || !matches!(cells[x - 1][y - 1], Cell::Floor(_));

            let top_right_good =
                x + 1 == SIZE || y == 0 || !matches!(cells[x + 1][y - 1], Cell::Floor(_));

            let bottom_left_good =
                x == 0 || y + 1 == SIZE || !matches!(cells[x - 1][y + 1], Cell::Floor(_));

            let bottom_right_good =
                x + 1 == SIZE || y + 1 == SIZE || !matches!(cells[x + 1][y + 1], Cell::Floor(_));

            // Move up
            if y > 0
                    && top_left_good
                    && top_right_good
                    && cells[x][y - 1] == Cell::Any
                    // Don't connect to existing corridor
                    && (y == 1 || !matches!(cells[x][y - 2], Cell::Floor(_)))
            {
                free_neighbours.push((x, y - 1));
            }

            // Move down
            if y + 1 < SIZE
                    && bottom_left_good
                    && bottom_right_good
                    && cells[x][y + 1] == Cell::Any
                    // Don't connect to existing corridor
                    && (y + 2 == SIZE || !matches!(cells[x][y + 2], Cell::Floor(_)))
            {
                free_neighbours.push((x, y + 1));
            }

            // Move left
            if x > 0
                    && top_left_good
                    && bottom_left_good
                    && cells[x - 1][y] == Cell::Any
                    // Don't connect to existing corridor
                    && (x == 1 || !matches!(cells[x - 2][y], Cell::Floor(_)))
            {
                free_neighbours.push((x - 1, y));
            }

            // Move right
            if x + 1 < SIZE
                    && top_right_good
                    && bottom_right_good
                    && cells[x + 1][y] == Cell::Any
                    // Don't connect to existing corridor
                    && (x + 2 == SIZE || !matches!(cells[x + 2][y], Cell::Floor(_)))
            {
                free_neighbours.push((x + 1, y));
            }

            // Ignore a random number of neighbours
            if free_neighbours.len() > 1 {
                free_neighbours.shuffle(&mut rng);
                let num_drop = rng.gen_range(0..free_neighbours.len());
                (0..num_drop).for_each(|_| {
                    free_neighbours.pop();
                });
            }

            // Put floor on remaining neighbours and queue expansion
            free_neighbours.into_iter().for_each(|(x, y)| {
                cells[x][y] = Cell::Floor(Floor::Empty);
                expand_from.push((x, y));
            });

            // Shuffle queue
            expand_from.shuffle(&mut rng);
        }

        // Replace remaining Any cells with walls and place monsters
        (0..SIZE).for_each(|x| {
            (0..SIZE).for_each(|y| {
                match cells[x][y] {
                    Cell::Any => {
                        cells[x][y] = Cell::Wall;
                    }
                    Cell::Floor(Floor::Empty) => {
                        let mut num_neighbours = 0;

                        // Up
                        if y > 0 && matches!(cells[x][y - 1], Cell::Floor(_)) {
                            num_neighbours += 1;
                        }

                        // Down
                        if y + 1 < SIZE && matches!(cells[x][y + 1], Cell::Floor(_)) {
                            num_neighbours += 1;
                        }

                        // Left
                        if x > 0 && matches!(cells[x - 1][y], Cell::Floor(_)) {
                            num_neighbours += 1;
                        }

                        // Right
                        if x + 1 < SIZE && matches!(cells[x + 1][y], Cell::Floor(_)) {
                            num_neighbours += 1;
                        }

                        assert_ne!(
                            num_neighbours, 0,
                            "no empty floor should be without neighbours"
                        );

                        if num_neighbours == 1 {
                            cells[x][y] = Cell::Floor(Floor::Monster);
                        }
                    }
                    _ => {}
                }
            })
        });
        Level { cells }
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
