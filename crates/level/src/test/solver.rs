use crate::{
    grid::{Grid, GridPos},
    Cell, CellFloor, CellKind, Level,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SolverCell {
    Unknown,
    Treasure,
    Monster,

    Empty,
    Wall,
    TreasureRoom,
}

impl From<&Cell> for SolverCell {
    fn from(value: &Cell) -> Self {
        match value.kind {
            CellKind::Wall => Self::Unknown,
            CellKind::Floor(CellFloor::Empty) => Self::Unknown,
            CellKind::Floor(CellFloor::Treasure) => Self::Treasure,
            CellKind::Floor(CellFloor::Monster) => Self::Monster,
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
    treasure_pos: GridPos,
}

impl Solver {
    pub fn new(level: Level) -> Self {
        let unknown_positions: Vec<GridPos> = level
            .iter()
            .filter_map(|cell| {
                if SolverCell::from(&cell) == SolverCell::Unknown {
                    Some(cell.position)
                } else {
                    None
                }
            })
            .collect();

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

        Self {
            level: (&level).into(),
            updated_positions: Vec::with_capacity(unknown_positions.len()),
            unknown_positions,
            row_numbers,
            col_numbers,
            treasure_pos,
        }
    }

    fn is_solved(&self) -> bool {
        self.level
            .map(|cell, pos| (*cell, pos))
            .iter()
            .all(|(cell, pos)| match cell {
                SolverCell::Unknown => false,
                SolverCell::Treasure => true,
                SolverCell::Monster => {
                    self.level
                        .count_neighbors(pos, |neighbor| *neighbor == SolverCell::Empty)
                        == 1
                }
                SolverCell::Empty => {
                    self.level.count_neighbors(pos, |neighbor| {
                        matches!(neighbor, SolverCell::Empty | SolverCell::TreasureRoom)
                    }) == 2
                }
                SolverCell::Wall => true,
                SolverCell::TreasureRoom => true,
            })
        // TODO: Check row and column numbers
    }
}

impl Iterator for Solver {
    type Item = Level;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(position) = self.updated_positions.pop() {
            self.unknown_positions.push(position);
            if self.level[position] == SolverCell::Wall {
                self.level[position] = SolverCell::Unknown;
            } else {
                self.level[position] = SolverCell::Unknown;
                break;
            }
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
