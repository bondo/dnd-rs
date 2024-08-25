// Based on https://github.com/MischaU8/dungeons_diagrams/blob/bf29a0454aec28476ac80286e130feeaa4081dec/src/solver.nim

use bevy::utils::HashSet;

use crate::{
    grid::{Grid, GridPos},
    Cell, CellFloor, CellKind, Level,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SolverCell {
    Hallway,
    Wall,
    Unknown,
    Monster,
    Treasure,
}

impl From<&Cell> for SolverCell {
    fn from(value: &Cell) -> Self {
        match value.kind {
            CellKind::Floor(CellFloor::Monster) => Self::Monster,
            CellKind::Floor(CellFloor::Treasure) => Self::Treasure,
            _ => Self::Unknown,
        }
    }
}

type SolverLevel = Grid<SolverCell>;

impl From<&Level> for SolverLevel {
    fn from(level: &Level) -> Self {
        level.grid.map(|cell, _position| SolverCell::from(cell))
    }
}

impl SolverLevel {
    fn count_islands(&self) -> usize {
        let mut visited = Grid::new(self.width(), self.height(), false);
        let mut islands = 0;

        self.iter().for_each(|(cell, pos)| {
            if visited[pos] {
                return;
            }

            if *cell == SolverCell::Wall {
                return;
            }

            let mut stack = vec![pos];
            visited[pos] = true;

            while let Some(pos) = stack.pop() {
                for neighbor in self.neighbors(pos) {
                    if visited[neighbor] {
                        continue;
                    }

                    if self[neighbor] == SolverCell::Wall {
                        continue;
                    }

                    visited[neighbor] = true;
                    stack.push(neighbor);
                }
            }

            islands += 1;
        });

        islands
    }

    fn find_treasures(&self) -> Vec<GridPos> {
        self.iter()
            .filter_map(|(cell, pos)| {
                if *cell == SolverCell::Treasure {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect()
    }

    fn all_treasure_room_tiles(&self, pos: GridPos) -> Vec<GridPos> {
        let mut tiles = Vec::new();
        for x in pos.x..=pos.x + 2 {
            for y in pos.y..=pos.y + 2 {
                if x >= self.width() || y >= self.height() {
                    continue;
                }
                tiles.push((x, y).into());
            }
        }
        tiles
    }

    fn is_empty_3x3(&self, pos: GridPos) -> bool {
        if pos.x + 2 >= self.width() || pos.y + 2 >= self.height() {
            return false;
        }
        for x in pos.x..=pos.x + 2 {
            for y in pos.y..=pos.y + 2 {
                if !matches!(
                    self[(x, y).into()],
                    SolverCell::Hallway | SolverCell::Treasure
                ) {
                    return false;
                }
            }
        }
        true
    }

    fn find_treasure_room_entrances(&self, pos: GridPos) -> Vec<GridPos> {
        let mut entrances = Vec::new();

        if pos.x > 0 {
            for y in pos.y..self.height().min(pos.y + 3) {
                let p = (pos.x - 1, y).into();
                if self[p] != SolverCell::Wall {
                    entrances.push(p);
                }
            }
        }

        if pos.y > 0 {
            for x in pos.x..self.width().min(pos.x + 3) {
                let p = (x, pos.y - 1).into();
                if self[p] != SolverCell::Wall {
                    entrances.push(p);
                }
            }
        }

        if pos.x + 3 < self.width() {
            for y in pos.y..self.height().min(pos.y + 3) {
                let p = (pos.x + 3, y).into();
                if self[p] != SolverCell::Wall {
                    entrances.push(p);
                }
            }
        }

        if pos.y + 3 < self.height() {
            for x in pos.x..self.width().min(pos.x + 3) {
                let p = (x, pos.y + 3).into();
                if self[p] != SolverCell::Wall {
                    entrances.push(p);
                }
            }
        }

        entrances
    }

    fn find_treasure_room(&self, treasure: &GridPos) -> Vec<GridPos> {
        let tx = treasure.x as isize;
        let ty = treasure.y as isize;
        let mut possible_rooms = Vec::new();
        for x in 0.max(tx - 2)..=tx {
            for y in 0.max(ty - 2)..=ty {
                let pos = (x as usize, y as usize).into();
                if self.is_empty_3x3(pos) {
                    possible_rooms.push(pos);
                }
            }
        }
        possible_rooms
    }

    fn is_wide_hallway(&self, pos: GridPos) -> bool {
        if pos.x + 1 >= self.width() || pos.y + 1 >= self.height() {
            return false;
        }
        for x in pos.x..=pos.x + 1 {
            for y in pos.y..=pos.y + 1 {
                if matches!(self[(x, y).into()], SolverCell::Wall | SolverCell::Treasure) {
                    return false;
                }
            }
        }
        true
    }
}

pub struct Solver {
    level: SolverLevel,
    row_numbers: Vec<usize>,
    col_numbers: Vec<usize>,
    treasures: Vec<GridPos>,
    solved: bool,
}

impl Solver {
    pub fn new(level: &Level) -> Self {
        let mut col_numbers = vec![0; level.width()];
        let mut row_numbers = vec![0; level.height()];
        level.iter().for_each(|c| {
            if c.has_wall() {
                col_numbers[c.x()] += 1;
                row_numbers[c.y()] += 1;
            }
        });

        let level = SolverLevel::from(level);

        let treasures = level.find_treasures();

        Self {
            level,
            row_numbers,
            col_numbers,
            treasures,
            solved: false,
        }
    }

    fn check_full_validity(&self) -> bool {
        let mut col_numbers = vec![0; self.level.width()];
        let mut row_numbers = vec![0; self.level.height()];
        self.level.iter().for_each(|(c, p)| {
            if *c == SolverCell::Wall {
                col_numbers[p.x] += 1;
                row_numbers[p.y] += 1;
            }
        });

        if row_numbers != self.row_numbers || col_numbers != self.col_numbers {
            return false;
        }

        // all dead ends are monsters, all monsters are on dead ends
        if !self.level.iter().all(|(cell, pos)| {
            let is_monster = *cell == SolverCell::Monster;
            let is_dead_end = self.level.count_neighbors(pos, |n| *n == SolverCell::Wall) == 3;
            is_monster == is_dead_end
        }) {
            return false;
        }

        // treasure room always 3x3 with single entrance
        let mut treasure_tiles: HashSet<GridPos> = HashSet::new();
        for treasure in &self.treasures {
            let rooms = self.level.find_treasure_room(treasure);
            if rooms.len() != 1 {
                return false;
            }
            let room = rooms[0];
            let entrances = self.level.find_treasure_room_entrances(room);
            if entrances.len() != 1 {
                return false;
            }
            for tile in self.level.all_treasure_room_tiles(room) {
                assert!(treasure_tiles.insert(tile));
            }
        }

        // hallways always one square wide; no 2x2 blocks outside treasure rooms
        if !self.level.iter().all(|(cell, pos)| {
            if treasure_tiles.contains(&pos) {
                return true;
            }
            if *cell == SolverCell::Hallway {
                return !self.level.is_wide_hallway(pos);
            }
            true
        }) {
            return false;
        }

        // all unshaded squares connected into single continuous shape
        if self.level.count_islands() != 1 {
            return false;
        }

        true
    }

    fn check_quick_validity(&self, pos: GridPos, cell: SolverCell) -> bool {
        let row_walls = self.level.count_row(pos.y, |c| *c == SolverCell::Wall);
        let row_unknowns = self.level.count_row(pos.y, |c| *c == SolverCell::Unknown);
        let min_row_walls = row_walls + if cell == SolverCell::Wall { 1 } else { 0 };
        let max_row_walls = row_walls + row_unknowns - 1;
        if min_row_walls > self.row_numbers[pos.y] || max_row_walls < self.row_numbers[pos.y] {
            return false;
        }

        let col_walls = self.level.count_col(pos.x, |c| *c == SolverCell::Wall);
        let col_unknowns = self.level.count_col(pos.x, |c| *c == SolverCell::Unknown);
        let min_col_walls = col_walls + if cell == SolverCell::Wall { 1 } else { 0 };
        let max_col_walls = col_walls + col_unknowns - 1;
        if min_col_walls > self.col_numbers[pos.x] || max_col_walls < self.col_numbers[pos.x] {
            return false;
        }

        // all dead ends are monsters, all monsters are on dead ends
        for neighbor in self.level.neighbors(pos) {
            if self.level[neighbor] == SolverCell::Unknown {
                continue;
            }
            let num_walls = self
                .level
                .count_neighbors(neighbor, |n| *n == SolverCell::Wall);
            let num_unknowns = self
                .level
                .count_neighbors(neighbor, |n| *n == SolverCell::Unknown);
            let min_walls = num_walls + if cell == SolverCell::Wall { 1 } else { 0 };
            let max_walls = num_walls + num_unknowns - 1;

            let has_monster = self.level[neighbor] == SolverCell::Monster;
            let is_dead_end = min_walls <= 3 && 3 <= max_walls;
            if has_monster != is_dead_end {
                return false;
            }
        }

        true
    }

    fn place_cell(&mut self, pos: Option<GridPos>) {
        if self.solved {
            return;
        }
        let Some(pos) = pos else {
            if self.check_full_validity() {
                self.solved = true;
            }
            return;
        };
        if self.level[pos] != SolverCell::Unknown {
            self.place_cell(self.level.next_pos(&pos));
            return;
        }
        for cell in vec![SolverCell::Hallway, SolverCell::Wall] {
            if !self.check_quick_validity(pos, cell) {
                continue;
            }
            self.level[pos] = cell;
            self.place_cell(self.level.next_pos(&pos));
            self.level[pos] = SolverCell::Unknown;
        }
    }

    pub fn solution(&mut self) -> Option<Level> {
        self.place_cell(Some((0, 0).into()));

        if self.solved {
            Some(Level {
                grid: self.level.map(|cell, pos| Cell {
                    kind: match cell {
                        SolverCell::Wall => CellKind::Wall,
                        SolverCell::Hallway => CellKind::Floor(CellFloor::Empty),
                        SolverCell::Monster => CellKind::Floor(CellFloor::Monster),
                        SolverCell::Treasure => CellKind::Floor(CellFloor::Treasure),
                        SolverCell::Unknown => panic!("Unknown cell"),
                    },
                    position: pos,
                }),
            })
        } else {
            None
        }
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
    let mut solver = Solver::new(&level);
    let solution = solver.solution();
    assert_eq!(solution, Some(level));
}
