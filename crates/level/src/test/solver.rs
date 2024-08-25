use crate::{
    grid::{Grid, GridPos},
    Cell, CellFloor, CellKind, Level,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SolverCell {
    Unknown,
    Monster,
    TreasureRoom,

    Floor,
    Wall,
}

impl From<&Cell> for SolverCell {
    fn from(value: &Cell) -> Self {
        match value.kind {
            CellKind::Floor(CellFloor::Monster) => Self::Monster,
            _ => Self::Unknown,
        }
    }
}

type SolverLevel = Grid<SolverCell>;

impl From<&Level> for SolverLevel {
    fn from(gen: &Level) -> Self {
        gen.grid.map(|cell, _position| SolverCell::from(cell))
    }
}

pub struct Solver {
    level: SolverLevel,
    updated_positions: Vec<GridPos>,
    unknown_positions: Vec<GridPos>,
    row_numbers: Vec<usize>,
    col_numbers: Vec<usize>,
    current_treasure_room: GridPos,
    current_treasure_room_exit: GridPos,
    possible_treasure_rooms: Vec<GridPos>,
    possible_treasure_room_exits: Vec<GridPos>,
}

impl Solver {
    pub fn new(level: Level) -> Self {
        let mut col_numbers = vec![0; level.width()];
        let mut row_numbers = vec![0; level.height()];
        let mut treasure_pos: Option<GridPos> = None;
        level.iter().for_each(|c| {
            if c.has_wall() {
                col_numbers[c.x()] += 1;
                row_numbers[c.y()] += 1;
            }
            if c.kind == CellKind::Floor(CellFloor::Treasure) {
                if treasure_pos.is_some() {
                    panic!("Multiple treasures found in level");
                }
                treasure_pos = Some(c.position);
            }
        });

        let Some(treasure_pos) = treasure_pos else {
            panic!("No treasure found in level");
        };

        let mut possible_treasure_rooms: Vec<GridPos> = Vec::new();

        let tx = treasure_pos.x as isize;
        let ty = treasure_pos.y as isize;
        for x in tx - 2..=tx + 2 {
            for y in ty - 2..=ty + 2 {
                if x >= 0 && y >= 0 {
                    possible_treasure_rooms.push((x as usize, y as usize).into());
                }
            }
        }

        let current_treasure_room = possible_treasure_rooms.pop().unwrap();

        let mut possible_treasure_room_exits = Vec::new();
        let tx = current_treasure_room.x;
        let ty = current_treasure_room.y;
        if ty > 1 {
            for x in tx - 1..=tx + 3 {
                possible_treasure_room_exits.push((x, ty - 1).into());
            }
        }
        if ty + 4 < level.height() {
            for x in tx - 1..=tx + 3 {
                possible_treasure_room_exits.push((x, ty + 3).into());
            }
        }
        if tx > 1 {
            for y in ty - 1..=ty + 3 {
                possible_treasure_room_exits.push((tx - 1, y).into());
            }
        }
        if tx + 4 < level.width() {
            for y in ty - 1..=ty + 3 {
                possible_treasure_room_exits.push((tx + 3, y).into());
            }
        }

        let current_treasure_room_exit = possible_treasure_room_exits.pop().unwrap();

        let mut level = SolverLevel::from(&level);
        Solver::mark_treasure_room(
            &mut level,
            current_treasure_room,
            current_treasure_room_exit,
        );

        let unknown_positions: Vec<GridPos> = level
            .iter()
            .filter_map(|(cell, pos)| {
                if *cell == SolverCell::Unknown {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect();

        Self {
            level,
            updated_positions: Vec::with_capacity(unknown_positions.len()),
            unknown_positions,
            row_numbers,
            col_numbers,
            current_treasure_room,
            current_treasure_room_exit,
            possible_treasure_rooms,
            possible_treasure_room_exits,
        }
    }

    fn mark_treasure_room(
        level: &mut SolverLevel,
        current_treasure_room: GridPos,
        current_treasure_room_exit: GridPos,
    ) {
        let tx = current_treasure_room.x;
        let ty = current_treasure_room.y;

        for x in tx - 1..=tx + 3 {
            for y in ty - 1..=ty + 3 {
                level[(x, y).into()] = if x < tx || y < ty || x > tx + 2 || y > ty + 2 {
                    SolverCell::Wall
                } else {
                    SolverCell::TreasureRoom
                };
            }
        }

        level[current_treasure_room_exit] = SolverCell::Unknown;
    }

    fn clear_treasure_room(level: &mut SolverLevel, current_treasure_room: GridPos) {
        let tx = current_treasure_room.x;
        let ty = current_treasure_room.y;
        for x in tx - 1..=tx + 3 {
            for y in ty - 1..=ty + 3 {
                level[(x, y).into()] = SolverCell::Unknown;
            }
        }
    }

    fn is_solved(&self) -> bool {
        let mut row_numbers = vec![0; self.level.height()];
        let mut col_numbers = vec![0; self.level.width()];

        self.level.iter().for_each(|(cell, pos)| {
            if *cell == SolverCell::Wall {
                row_numbers[pos.y] += 1;
                col_numbers[pos.x] += 1;
            }
        });

        if row_numbers != self.row_numbers || col_numbers != self.col_numbers {
            return false;
        }

        self.level.iter().all(|(cell, pos)| match cell {
            SolverCell::Unknown => false,
            SolverCell::Wall => true,
            SolverCell::TreasureRoom => true,
            SolverCell::Monster => {
                self.level.count_neighbors(pos, |neighbor| {
                    matches!(neighbor, SolverCell::Floor | SolverCell::TreasureRoom)
                }) == 1
            }
            SolverCell::Floor => {
                self.level.count_neighbors(pos, |neighbor| {
                    matches!(neighbor, SolverCell::Floor | SolverCell::TreasureRoom)
                }) == 2
            }
        })

        // TODO: Check connectivity
    }

    fn can_have_floor(&self, pos: GridPos) -> bool {
        self.level.count_neighbors(pos, |neighbor| {
            matches!(neighbor, SolverCell::Floor | SolverCell::TreasureRoom)
        }) <= 2
    }

    fn can_have_wall(&self, pos: GridPos) -> bool {
        true // TODO: Check row/column numbers
    }
}

impl Iterator for Solver {
    type Item = Level;

    fn next(&mut self) -> Option<Self::Item> {
        let mut work_queue = vec![self.current_treasure_room_exit];
        let mut tried_wall = Grid::new(self.level.width(), self.level.height(), false);
        tried_wall[self.current_treasure_room_exit] = true;

        loop {
            // TODO: Backtrack. Update `tried_wall` along the way.
            //       If no more options, try next treasure room exit.
            //       If no more options, try next treasure room.
            //       If no more treasure rooms, return None.

            // while let Some(position) = self.updated_positions.pop() {
            //     self.unknown_positions.push(position);
            //     if self.level[position] == SolverCell::Wall {
            //         self.level[position] = SolverCell::Unknown;
            //     } else {
            //         self.level[position] = SolverCell::Unknown;
            //         break;
            //     }
            // }

            while let Some(pos) = work_queue.pop() {
                let cell = self.level[pos];
                if cell != SolverCell::Unknown {
                    continue;
                }
                self.level[pos] = if tried_wall[pos] && self.can_have_floor(pos) {
                    SolverCell::Floor
                } else if !tried_wall[pos] && self.can_have_wall(pos) {
                    tried_wall[pos] = true;
                    SolverCell::Wall
                } else {
                    // TODO: Backtrack
                    continue;
                };
                self.updated_positions.push(pos);

                for neighbor in self.level.neighbors(pos) {
                    if self.level[neighbor] == SolverCell::Unknown {
                        work_queue.push(neighbor);
                    }
                }
            }

            // TODO: Backtrack if level is not solvable

            // Place all unknown cells
            // loop {
            //     let Some(pos) = self.unknown_positions.pop() else {
            //         if self.is_solved() {
            //             return Some(Level {
            //                 grid: self.level.map(|cell, position| Cell {
            //                     kind: match cell {
            //                         SolverCell::Wall => CellKind::Wall,
            //                         SolverCell::Monster => CellKind::Floor(CellFloor::Monster),
            //                         SolverCell::TreasureRoom => {
            //                             CellKind::Floor(CellFloor::Treasure)
            //                         }
            //                         SolverCell::Floor => CellKind::Floor(CellFloor::Empty),
            //                         SolverCell::Unknown => panic!("Unknown cell in solved level"),
            //                     },
            //                     position,
            //                 }),
            //             });
            //         } else {
            //             // Backtrack
            //             break;
            //         }
            //     };
            // }
        }

        None
    }
}

impl PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid
    }
}

#[test]
fn test_solve_level_random() {
    let level = Level::random(8, 8);
    let solver = Solver::new(level.clone());
    let solutions: Vec<Level> = solver.collect();
    assert_eq!(solutions, vec![level]);
}
