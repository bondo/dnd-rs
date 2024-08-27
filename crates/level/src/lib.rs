use std::{fmt::Debug, time::Instant};

mod gen;
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

#[derive(bevy::prelude::Resource, Clone, PartialEq)]
pub struct Level {
    grid: Grid<Cell>,
}

impl Level {
    pub fn random(width: usize, height: usize) -> Self {
        GenLevel::random(width, height).into()
    }

    pub fn random_unique_solution(width: usize, height: usize) -> Self {
        let start = Instant::now();
        let level = loop {
            let level_start = Instant::now();
            let level = Level::random(width, height);
            println!("Generated level in {:?}", level_start.elapsed());

            let solver_start = Instant::now();
            let mut solver = Solver::from_level(&level);
            let Some(_) = solver.next() else {
                panic!("Generated level without solution:\n{:?}", level);
            };
            let is_unique = solver.next().is_none();
            println!("Solved level in {:?}", solver_start.elapsed());

            if is_unique {
                println!("Level has unique solution");
                break level;
            }

            println!("Level has multiple solutions");
        };
        println!("Generated unique level in {:?}", start.elapsed());
        level
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
