use std::fmt::Debug;

mod gen;
use gen::{GenCell, GenFloor, GenLevel};

mod grid;
use grid::Grid;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Floor {
    Empty,
    Treasure,
    Monster,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Wall,
    Floor(Floor),
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Cell::Wall => '#',
            Cell::Floor(Floor::Empty) => '.',
            Cell::Floor(Floor::Treasure) => 'T',
            Cell::Floor(Floor::Monster) => 'M',
        };
        write!(f, "{}", c)
    }
}

impl Debug for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub struct Level(Grid<Cell>);

impl Level {
    pub fn random(width: usize, height: usize) -> Self {
        GenLevel::random(width, height).into()
    }
}

impl From<GenLevel> for Level {
    fn from(gen_level: GenLevel) -> Self {
        Level(gen_level.map(|cell, p| match cell {
            GenCell::Any | GenCell::Wall => Cell::Wall,
            GenCell::Floor(GenFloor::Treasure) => Cell::Floor(Floor::Treasure),
            GenCell::Floor(GenFloor::Empty) => Cell::Floor(
                match gen_level.count_neighbors(p, |n| matches!(n, GenCell::Floor(_))) {
                    0 => panic!("Floor cell with no neighbors"),
                    1 => Floor::Monster,
                    _ => Floor::Empty,
                },
            ),
        }))
    }
}
