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

impl std::fmt::Debug for SolverCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            SolverCell::Wall => '#',
            SolverCell::Hallway => '.',
            SolverCell::Monster => 'M',
            SolverCell::Treasure => 'T',
            SolverCell::Unknown => '?',
        };
        write!(f, "{}", c)
    }
}

type SolverLevel = Grid<SolverCell>;

impl From<&Level> for SolverLevel {
    fn from(level: &Level) -> Self {
        level.grid.map(|cell, _position| SolverCell::from(cell))
    }
}

impl From<&SolverLevel> for Level {
    fn from(level: &SolverLevel) -> Self {
        Self {
            grid: level.map(|cell, pos| Cell {
                kind: match cell {
                    SolverCell::Wall => CellKind::Wall,
                    SolverCell::Hallway => CellKind::Floor(CellFloor::Empty),
                    SolverCell::Monster => CellKind::Floor(CellFloor::Monster),
                    SolverCell::Treasure => CellKind::Floor(CellFloor::Treasure),
                    SolverCell::Unknown => panic!("Unknown cell:\n{level:?}"),
                },
                position: pos,
            }),
        }
    }
}

impl TryFrom<&str> for SolverLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines = value.trim().lines();
        let height = lines.clone().count();
        let width = lines.clone().next().map(str::len).unwrap_or(0);
        let mut grid = Grid::new(width, height, SolverCell::Unknown);

        for (y, line) in lines.enumerate() {
            let line = line.trim();
            if line.len() != width {
                return Err(format!("Invalid line length at line {}", y));
            }
            for (x, c) in line.chars().enumerate() {
                let pos = (x, y).into();
                grid[pos] = match c {
                    '#' => SolverCell::Wall,
                    '.' => SolverCell::Hallway,
                    'M' => SolverCell::Monster,
                    'T' => SolverCell::Treasure,
                    '?' => SolverCell::Unknown,
                    _ => {
                        return Err(format!("Invalid character at ({}, {}): {}", x, y, c));
                    }
                };
            }
        }

        Ok(grid)
    }
}

impl TryFrom<&str> for IterativeSolver {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut lines = value.trim().lines();
        let Some(col_header) = lines.next() else {
            return Err("Column header not found".to_string());
        };

        let mut col_numbers: Vec<usize> = Vec::new();
        for v in col_header.split_whitespace() {
            let Ok(v) = v.parse::<usize>() else {
                return Err(format!(
                    "Could not read column header value as integer: {v}"
                ));
            };
            col_numbers.push(v);
        }

        let width = col_numbers.len();
        let height = lines.clone().count();
        let mut grid = Grid::new(width, height, SolverCell::Unknown);

        let mut row_numbers: Vec<usize> = Vec::new();
        for (y, line) in lines.enumerate() {
            let mut line = line.split_whitespace();
            if line.clone().count() != width + 1 {
                return Err(format!(
                    "Invalid line length {} at line {}. Expected {}.",
                    line.clone().count(),
                    y,
                    width + 1
                ));
            }
            let Some(row_header) = line.next() else {
                return Err(format!("Row header not found on line {}", y));
            };
            let Ok(row_header) = row_header.parse::<usize>() else {
                return Err(format!(
                    "Could not row header on line {y} as integer: {row_header}"
                ));
            };
            row_numbers.push(row_header);
            for (x, v) in line.enumerate() {
                let pos = (x, y).into();
                grid[pos] = match v {
                    "#" => SolverCell::Wall,
                    "." => SolverCell::Hallway,
                    "M" => SolverCell::Monster,
                    "T" => SolverCell::Treasure,
                    "?" => SolverCell::Unknown,
                    _ => {
                        return Err(format!("Invalid value at {pos:?}: {v}"));
                    }
                };
            }
        }

        Ok(IterativeSolver::from_parts(grid, row_numbers, col_numbers))
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

pub struct IterativeSolver {
    level: SolverLevel,
    row_numbers: Vec<usize>,
    col_numbers: Vec<usize>,
    treasures: Vec<GridPos>,
    max_solutions: usize,
    solutions: Vec<Level>,
}

impl std::fmt::Debug for IterativeSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Solver (solutions = {})", self.solutions.len())?;

        let level = format!("{:?}", self.level);
        let level_rows = level.trim().split('\n').collect::<Vec<_>>();

        // Print column header
        write!(f, "  ")?;
        for i in 0..self.level.width() {
            write!(f, "{}", self.col_numbers[i])?;
        }
        writeln!(f)?;

        // Print rows, prefixed with row header
        assert_eq!(self.row_numbers.len(), level_rows.len());
        for (i, row) in level_rows.iter().enumerate() {
            writeln!(f, "{} {row}", self.row_numbers[i])?;
        }

        Ok(())
    }
}

impl IterativeSolver {
    #[allow(dead_code)]
    pub fn from_level(level: &Level) -> Self {
        let mut col_numbers = vec![0; level.width()];
        let mut row_numbers = vec![0; level.height()];
        level.iter().for_each(|c| {
            if c.has_wall() {
                col_numbers[c.x()] += 1;
                row_numbers[c.y()] += 1;
            }
        });

        let level = SolverLevel::from(level);

        Self::from_parts(level, row_numbers, col_numbers)
    }

    fn from_parts(level: SolverLevel, row_numbers: Vec<usize>, col_numbers: Vec<usize>) -> Self {
        let treasures = level.find_treasures();

        Self {
            level,
            row_numbers,
            col_numbers,
            treasures,
            max_solutions: 0,
            solutions: Vec::new(),
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
            let num_neighbors = self.level.count_neighbors(pos, |_| true);
            let num_walls = self.level.count_neighbors(pos, |n| *n == SolverCell::Wall);
            let is_monster = *cell == SolverCell::Monster;
            let is_dead_end = *cell != SolverCell::Wall && num_walls == (num_neighbors - 1);
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
        let max_row_walls = row_walls + row_unknowns - if cell == SolverCell::Wall { 0 } else { 1 };
        if min_row_walls > self.row_numbers[pos.y] || max_row_walls < self.row_numbers[pos.y] {
            return false;
        }

        let col_walls = self.level.count_col(pos.x, |c| *c == SolverCell::Wall);
        let col_unknowns = self.level.count_col(pos.x, |c| *c == SolverCell::Unknown);
        let min_col_walls = col_walls + if cell == SolverCell::Wall { 1 } else { 0 };
        let max_col_walls = col_walls + col_unknowns - if cell == SolverCell::Wall { 0 } else { 1 };
        if min_col_walls > self.col_numbers[pos.x] || max_col_walls < self.col_numbers[pos.x] {
            return false;
        }

        // all dead ends are monsters, all monsters are on dead ends
        for neighbor in self.level.neighbors(pos) {
            if self.level[neighbor] == SolverCell::Unknown {
                continue;
            }
            let num_neighbors = self.level.count_neighbors(neighbor, |_| true);

            let num_walls = self
                .level
                .count_neighbors(neighbor, |n| *n == SolverCell::Wall);
            let num_unknowns = self
                .level
                .count_neighbors(neighbor, |n| *n == SolverCell::Unknown);
            let min_walls = num_walls + if cell == SolverCell::Wall { 1 } else { 0 };
            let max_walls = num_walls + num_unknowns - if cell == SolverCell::Wall { 0 } else { 1 };

            let has_monster = self.level[neighbor] == SolverCell::Monster;
            let is_dead_end = self.level[neighbor] != SolverCell::Wall
                && min_walls == (num_neighbors - 1)
                && num_unknowns < 2;
            if has_monster
                && !(min_walls <= (num_neighbors - 1) && (num_neighbors - 1) <= max_walls)
            {
                return false;
            }
            if !has_monster && is_dead_end {
                return false;
            }
        }

        true
    }

    fn solve(&mut self) {
        let mut stack: Vec<(Option<GridPos>, Option<SolverCell>)> =
            Vec::with_capacity(self.level.width() * self.level.height() * 2);

        stack.push((Some((0, 0).into()), None));

        while let Some((pos, cell)) = stack.pop() {
            let Some(pos) = pos else {
                if self.check_full_validity() {
                    self.solutions.push((&self.level).into());

                    if self.solutions.len() >= self.max_solutions {
                        return;
                    }
                }
                continue;
            };
            match cell {
                // Backtrack
                Some(SolverCell::Unknown) => {
                    self.level[pos] = SolverCell::Unknown;
                }
                // Try cell
                Some(cell) => {
                    if !self.check_quick_validity(pos, cell) {
                        continue;
                    }
                    self.level[pos] = cell;
                    stack.push((self.level.next_pos(&pos), None));
                }
                // If cell at pos is unknown, try all possibilities
                None => {
                    if self.level[pos] == SolverCell::Unknown {
                        stack.push((Some(pos), Some(SolverCell::Unknown)));
                        stack.push((Some(pos), Some(SolverCell::Wall)));
                        stack.push((Some(pos), Some(SolverCell::Hallway)));
                    } else {
                        stack.push((self.level.next_pos(&pos), None));
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn first_solution(mut self) -> Option<Level> {
        self.max_solutions = 1;
        self.solve();

        self.solutions.pop()
    }

    #[allow(dead_code)]
    pub fn all_solutions(mut self) -> Vec<Level> {
        self.max_solutions = usize::MAX;
        self.solve();

        self.solutions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_level_random() {
        let level = Level::random(8, 8);
        let solver = IterativeSolver::from_level(&level);
        let solutions = solver.all_solutions();
        assert!(
            solutions.contains(&level),
            "Level {level:?} not found in solutions: {solutions:?}"
        );
    }

    #[test]
    fn test_solve_level_weird() {
        let solver = IterativeSolver::try_from(
            r#"
  4 3 3 5 2 3 2 3
4 ? ? M ? ? ? ? ?
2 ? ? ? ? ? ? ? M
3 M ? ? M ? ? ? ?
5 ? ? ? ? ? ? ? M
2 ? T ? ? ? ? ? ?
2 ? ? ? ? ? ? ? ?
1 ? ? ? ? ? ? ? ?
6 ? ? ? ? ? M ? M
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SolverLevel::try_from(
                r#"
##M#...#
#....#.M
M.#M#..#
####..#M
.T.#.#..
.....#.#
...#....
#####M#M
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }

    // Source of levels: https://www.reddit.com/r/puzzles/comments/d72zg1/advanced_dungeons_and_diagrams_map_making_logic/

    #[test]
    fn test_solve_sample() {
        let solver = IterativeSolver::try_from(
            r#"
  4 2 4 1 2 1
3 ? ? ? ? ? T
1 ? ? ? ? ? ?
2 ? ? ? ? ? ?
5 ? ? ? ? ? ?
1 ? ? ? ? ? M
2 M ? ? ? ? ?
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SolverLevel::try_from(
                r#"
###..T
#.....
#.#...
#.####
....#M
M##...
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }

    #[test]
    fn test_solve_tenaxxuss_gullet() {
        let solver = IterativeSolver::try_from(
            r#"
  4 4 2 6 2 3 4 7
7 ? ? ? ? ? M ? ?
3 ? ? ? ? ? ? ? ?
4 ? T ? ? ? ? ? ?
1 ? ? ? ? ? ? ? ?
7 ? ? ? ? ? ? ? ?
1 M ? ? ? ? ? ? ?
6 ? ? ? ? ? ? ? ?
3 ? ? M ? ? ? ? M
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SolverLevel::try_from(
                r#"
#####M##
...#..##
.T.#.###
.......#
######.#
M......#
##.#.###
##M#...M
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }

    #[test]
    fn test_solve_the_twin_cities_of_the_dead() {
        let solver = IterativeSolver::try_from(
            r#"
  1 3 1 5 3 4 3 5
5 ? ? ? ? ? ? ? ?
2 ? ? T ? T ? ? ?
2 ? ? ? ? ? ? ? ?
3 ? ? ? ? ? ? ? ?
6 M ? ? ? ? ? ? ?
0 ? ? ? ? ? ? ? ?
6 ? ? ? ? ? ? ? ?
1 ? ? ? ? M ? M ?
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SolverLevel::try_from(
                r#"
...#####
..T#T..#
...#...#
##.....#
M#.#####
........
.######.
....M#M.
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }

    #[test]
    fn test_solve_the_hive_of_great_sorrow() {
        let solver = IterativeSolver::try_from(
            r#"
  3 6 0 5 4 0 6 3
6 ? ? M ? ? M ? ?
2 M ? ? ? ? ? ? M
4 ? ? ? ? ? ? ? ?
3 ? ? ? ? M ? ? ?
2 ? ? ? ? ? ? ? ?
4 ? ? ? ? ? ? ? ?
2 M ? ? ? ? ? ? M
4 ? ? ? ? ? ? ? ?
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SolverLevel::try_from(
                r#"
##M##M##
M#....#M
.#.##.#.
.#.#M.#.
...##...
##....##
M..##..M
##....##
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }
}
