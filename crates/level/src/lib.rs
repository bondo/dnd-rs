use std::fmt::Debug;

mod gen;
use bevy::log::info;
use gen::{GenCell, GenFloor, GenLevel};

mod grid;
use grid::{Grid, GridPos};

mod solver;
pub use solver::Solver;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellFloor {
    Empty,
    Treasure,
    Monster,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Wall,
    Floor(CellFloor),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    kind: CellKind,
    position: GridPos,
}

impl Cell {
    pub fn x(&self) -> usize {
        self.position.x
    }

    pub fn y(&self) -> usize {
        self.position.y
    }

    pub fn kind(&self) -> &CellKind {
        &self.kind
    }

    pub fn has_wall(&self) -> bool {
        self.kind == CellKind::Wall
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self.kind {
            CellKind::Wall => '#',
            CellKind::Floor(CellFloor::Empty) => '.',
            CellKind::Floor(CellFloor::Treasure) => 'T',
            CellKind::Floor(CellFloor::Monster) => 'M',
        };
        write!(f, "{}", c)
    }
}

impl Debug for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.grid)
    }
}

#[derive(bevy::prelude::Component, Clone, PartialEq)]
pub struct Level {
    grid: Grid<Cell>,
}

impl Level {
    pub fn random(width: usize, height: usize) -> Result<Self, &'static str> {
        let level_start = chrono::Utc::now();
        let level = GenLevel::random(width, height)?.into();
        info!(
            "Generated level in {:?}",
            chrono::Utc::now()
                .signed_duration_since(level_start)
                .to_std()
                .unwrap()
        );
        Ok(level)
    }

    pub fn builder(width: usize, height: usize) -> LevelBuilder {
        LevelBuilder::new(width, height)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.grid.iter().map(|(c, _)| c)
    }

    pub fn width(&self) -> usize {
        self.grid.width()
    }

    pub fn height(&self) -> usize {
        self.grid.height()
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        self.grid[(x, y).into()].has_wall()
    }
}

impl From<GenLevel> for Level {
    fn from(gen: GenLevel) -> Self {
        Level {
            grid: gen.map(|cell, position| Cell {
                kind: match cell {
                    GenCell::Any | GenCell::Wall => CellKind::Wall,
                    GenCell::Floor(GenFloor::Treasure) => CellKind::Floor(CellFloor::Treasure),
                    GenCell::Floor(GenFloor::Empty) => CellKind::Floor(
                        match gen.count_neighbors(position, |n| matches!(n, GenCell::Floor(_))) {
                            0 => panic!("Floor cell with no neighbors"),
                            1 => CellFloor::Monster,
                            _ => CellFloor::Empty,
                        },
                    ),
                },
                position,
            }),
        }
    }
}

pub struct LevelBuilder {
    width: usize,
    height: usize,
    check_unique_solution: bool,
    check_too_many_walls: bool,
}

impl LevelBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        LevelBuilder {
            width,
            height,
            check_unique_solution: false,
            check_too_many_walls: false,
        }
    }

    pub fn check_unique_solution(mut self) -> Self {
        self.check_unique_solution = true;
        self
    }

    pub fn check_too_many_walls(mut self) -> Self {
        self.check_too_many_walls = true;
        self
    }

    pub fn build(&self) -> Result<Level, &'static str> {
        let start = chrono::Utc::now();
        let level = loop {
            let level = Level::random(self.width, self.height)?;

            if self.check_too_many_walls {
                // Check that we don't have any 3x3 wall blocks
                let has_too_many_walls = level.iter().any(|cell| {
                    let x = cell.x();
                    let y = cell.y();

                    if x + 2 >= level.width() || y + 2 >= level.height() {
                        return false;
                    }

                    (0..3).all(|dx| (0..3).all(|dy| level.is_wall(x + dx, y + dy)))
                });

                if has_too_many_walls {
                    info!("Level has 3x3 wall blocks");
                    continue;
                } else {
                    info!("Level has no 3x3 wall blocks");
                }
            }

            if self.check_unique_solution {
                let solver_start = chrono::Utc::now();
                let mut solver = Solver::from_level(&level);
                let Some(_) = solver.next() else {
                    panic!("Generated level without solution:\n{:?}", level);
                };
                let has_unique_solution = solver.next().is_none();
                info!(
                    "Solved level in {:?}",
                    chrono::Utc::now()
                        .signed_duration_since(solver_start)
                        .to_std()
                        .unwrap()
                );

                if has_unique_solution {
                    info!("Level has unique solution");
                } else {
                    info!("Level has multiple solutions");
                    continue;
                }
            }

            break level;
        };
        info!(
            "Generated {} level in {:?}",
            match (self.check_unique_solution, self.check_too_many_walls) {
                (true, true) => "validated",
                (true, false) => "unique",
                (false, true) => "filtered",
                (false, false) => "random",
            },
            chrono::Utc::now()
                .signed_duration_since(start)
                .to_std()
                .unwrap()
        );
        Ok(level)
    }
}
