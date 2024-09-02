use bevy::utils::HashSet;

use crate::{
    grid::{Grid, GridPos},
    Cell, CellFloor, CellKind, Level,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SmartSolverCell {
    Hallway,
    Wall,
    Unknown,
    Monster,
    Treasure,
}

impl From<&Cell> for SmartSolverCell {
    fn from(value: &Cell) -> Self {
        match value.kind {
            CellKind::Floor(CellFloor::Monster) => Self::Monster,
            CellKind::Floor(CellFloor::Treasure) => Self::Treasure,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Debug for SmartSolverCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            SmartSolverCell::Wall => '#',
            SmartSolverCell::Hallway => '.',
            SmartSolverCell::Monster => 'M',
            SmartSolverCell::Treasure => 'T',
            SmartSolverCell::Unknown => '?',
        };
        write!(f, "{}", c)
    }
}

type SmartSolverLevel = Grid<SmartSolverCell>;

impl From<&Level> for SmartSolverLevel {
    fn from(level: &Level) -> Self {
        level
            .grid
            .map(|cell, _position| SmartSolverCell::from(cell))
    }
}

impl From<&SmartSolverLevel> for Level {
    fn from(level: &SmartSolverLevel) -> Self {
        Self {
            grid: level.map(|cell, pos| Cell {
                kind: match cell {
                    SmartSolverCell::Wall => CellKind::Wall,
                    SmartSolverCell::Hallway => CellKind::Floor(CellFloor::Empty),
                    SmartSolverCell::Monster => CellKind::Floor(CellFloor::Monster),
                    SmartSolverCell::Treasure => CellKind::Floor(CellFloor::Treasure),
                    SmartSolverCell::Unknown => panic!("Unknown cell:\n{level:?}"),
                },
                position: pos,
            }),
        }
    }
}

impl TryFrom<&str> for SmartSolverLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines = value.trim().lines();
        let height = lines.clone().count();
        let width = lines.clone().next().map(str::len).unwrap_or(0);
        let mut grid = Grid::new(width, height, SmartSolverCell::Unknown);

        for (y, line) in lines.enumerate() {
            let line = line.trim();
            if line.len() != width {
                return Err(format!("Invalid line length at line {}", y));
            }
            for (x, c) in line.chars().enumerate() {
                let pos = (x, y).into();
                grid[pos] = match c {
                    '#' => SmartSolverCell::Wall,
                    '.' => SmartSolverCell::Hallway,
                    'M' => SmartSolverCell::Monster,
                    'T' => SmartSolverCell::Treasure,
                    '?' => SmartSolverCell::Unknown,
                    _ => {
                        return Err(format!("Invalid character at ({}, {}): {}", x, y, c));
                    }
                };
            }
        }

        Ok(grid)
    }
}

impl TryFrom<&str> for SmartSolver {
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
        let mut grid = Grid::new(width, height, SmartSolverCell::Unknown);

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
                    "#" => SmartSolverCell::Wall,
                    "." => SmartSolverCell::Hallway,
                    "M" => SmartSolverCell::Monster,
                    "T" => SmartSolverCell::Treasure,
                    "?" => SmartSolverCell::Unknown,
                    _ => {
                        return Err(format!("Invalid value at {pos:?}: {v}"));
                    }
                };
            }
        }

        Ok(SmartSolver::from_parts(grid, row_numbers, col_numbers))
    }
}

impl SmartSolverLevel {
    fn find_treasures(&self) -> Vec<GridPos> {
        self.iter()
            .filter_map(|(cell, pos)| match cell {
                SmartSolverCell::Treasure => Some(pos),
                _ => None,
            })
            .collect()
    }

    fn find_monsters(&self) -> Vec<GridPos> {
        self.iter()
            .filter_map(|(cell, pos)| match cell {
                SmartSolverCell::Monster => Some(pos),
                _ => None,
            })
            .collect()
    }

    fn count_islands(&self) -> usize {
        let mut visited = Grid::new(self.width(), self.height(), false);
        let mut islands = 0;

        self.iter().for_each(|(&cell, pos)| {
            if visited[pos] {
                return;
            }

            if cell == SmartSolverCell::Wall {
                return;
            }

            let mut stack = vec![pos];
            visited[pos] = true;

            while let Some(pos) = stack.pop() {
                for neighbor in self.iter_neighbors(pos) {
                    if visited[neighbor] {
                        continue;
                    }

                    if self[neighbor] == SmartSolverCell::Wall {
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

    fn is_wide_hallway(&self, pos: GridPos) -> bool {
        if pos.x + 1 >= self.width() || pos.y + 1 >= self.height() {
            return false;
        }
        for x in pos.x..=pos.x + 1 {
            for y in pos.y..=pos.y + 1 {
                if matches!(
                    self[(x, y).into()],
                    SmartSolverCell::Wall | SmartSolverCell::Treasure
                ) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct SmartSolver {
    level: SmartSolverLevel,
    row_total_walls: Vec<usize>,
    col_total_walls: Vec<usize>,
    row_missing_walls: Vec<usize>,
    col_missing_walls: Vec<usize>,
    row_unknown_count: Vec<usize>,
    col_unknown_count: Vec<usize>,
    unsatisfied_monsters: Vec<GridPos>,
    unhandled_treasures: Vec<GridPos>,
    placed_treasure_rooms: Vec<GridPos>,
    next_pos: Option<GridPos>,
}

impl std::fmt::Debug for SmartSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SmartSolver")?;

        let level = format!("{:?}", self.level);
        let level_rows = level.trim().split('\n').collect::<Vec<_>>();

        // Print column header
        write!(f, "   ")?;
        for i in 0..self.level.width() {
            write!(f, " {i}")?;
        }
        writeln!(f)?;

        write!(f, "   ")?;
        for i in 0..self.level.width() {
            write!(f, " {}", self.col_total_walls[i])?;
        }
        writeln!(f)?;

        // Print rows, prefixed with row header
        assert_eq!(
            self.row_total_walls.len(),
            level_rows.len(),
            "Unexpected level rows: {level_rows:?}"
        );
        for (i, row) in level_rows.iter().enumerate() {
            write!(f, "{i} {}", self.row_total_walls[i])?;
            for c in row.chars() {
                write!(f, " {c}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl SmartSolver {
    pub fn from_level(level: &Level) -> Self {
        let mut col_numbers = vec![0; level.width()];
        let mut row_numbers = vec![0; level.height()];
        level.iter().for_each(|c| {
            if c.has_wall() {
                col_numbers[c.x()] += 1;
                row_numbers[c.y()] += 1;
            }
        });

        let level = SmartSolverLevel::from(level);

        Self::from_parts(level, row_numbers, col_numbers)
    }

    fn from_parts(
        level: SmartSolverLevel,
        row_numbers: Vec<usize>,
        col_numbers: Vec<usize>,
    ) -> Self {
        let treasures = level.find_treasures();
        let monsters = level.find_monsters();

        let mut row_unknown_count = vec![0; level.height()];
        let mut col_unknown_count = vec![0; level.width()];
        for y in 0..level.height() {
            row_unknown_count[y] = level.count_row(y, |c| c == &SmartSolverCell::Unknown);
        }
        for x in 0..level.width() {
            col_unknown_count[x] = level.count_col(x, |c| c == &SmartSolverCell::Unknown);
        }

        Self {
            level,
            row_missing_walls: row_numbers.clone(),
            col_missing_walls: col_numbers.clone(),
            row_unknown_count,
            col_unknown_count,
            row_total_walls: row_numbers,
            col_total_walls: col_numbers,
            unhandled_treasures: treasures,
            placed_treasure_rooms: Vec::new(),
            unsatisfied_monsters: monsters,
            next_pos: Some((0, 0).into()),
        }
    }

    fn put_wall(&mut self, pos: GridPos) -> Result<(), ()> {
        debug_assert_eq!(self.level[pos], SmartSolverCell::Unknown);

        self.level[pos] = SmartSolverCell::Wall;

        if self.row_missing_walls[pos.y] == 0 || self.col_missing_walls[pos.x] == 0 {
            return Err(());
        }

        self.row_missing_walls[pos.y] -= 1;
        self.col_missing_walls[pos.x] -= 1;

        self.row_unknown_count[pos.y] -= 1;
        self.col_unknown_count[pos.x] -= 1;

        Ok(())
    }

    fn put_hallway(&mut self, pos: GridPos) {
        debug_assert_eq!(self.level[pos], SmartSolverCell::Unknown);

        self.level[pos] = SmartSolverCell::Hallway;

        self.row_unknown_count[pos.y] -= 1;
        self.col_unknown_count[pos.x] -= 1;
    }

    fn fill_out_logical_values(&mut self) -> Result<(), ()> {
        let mut changed = true;
        while changed {
            changed = false;
            for y in 0..self.level.height() {
                let unknown_count = self.row_unknown_count[y];
                let missing_walls = self.row_missing_walls[y];
                if missing_walls > unknown_count {
                    return Err(());
                }
                if unknown_count == 0 {
                    continue;
                }
                if missing_walls == 0 {
                    for x in 0..self.level.width() {
                        let pos = (x, y).into();
                        if self.level[pos] == SmartSolverCell::Unknown {
                            self.put_hallway(pos);
                            changed = true;
                        }
                    }
                }
                if missing_walls == unknown_count {
                    for x in 0..self.level.width() {
                        let pos = (x, y).into();
                        if self.level[pos] == SmartSolverCell::Unknown {
                            self.put_wall(pos)?;
                            changed = true;
                        }
                    }
                }
            }

            for x in 0..self.level.width() {
                let unknown_count = self.col_unknown_count[x];
                let missing_walls = self.col_missing_walls[x];
                if missing_walls > unknown_count {
                    return Err(());
                }
                if unknown_count == 0 {
                    continue;
                }
                if missing_walls == 0 {
                    for y in 0..self.level.height() {
                        let pos = (x, y).into();
                        if self.level[pos] == SmartSolverCell::Unknown {
                            self.put_hallway(pos);
                            changed = true;
                        }
                    }
                }
                if missing_walls == unknown_count {
                    for y in 0..self.level.height() {
                        let pos = (x, y).into();
                        if self.level[pos] == SmartSolverCell::Unknown {
                            self.put_wall(pos)?;
                            changed = true;
                        }
                    }
                }
            }

            let mut remove_unsatisfied_monsters = Vec::new();
            for idx in 0..self.unsatisfied_monsters.len() {
                let pos = self.unsatisfied_monsters[idx];

                let non_wall_neighbors = self
                    .level
                    .iter_neighbors(pos)
                    .filter(|&n| self.level[n] != SmartSolverCell::Wall)
                    .collect::<Vec<_>>();

                if non_wall_neighbors.len() == 0 {
                    return Err(());
                }

                let hallway_neigbors = non_wall_neighbors
                    .iter()
                    .filter(|&&n| self.level[n] == SmartSolverCell::Hallway)
                    .collect::<Vec<_>>();

                match hallway_neigbors.len() {
                    0 => {
                        let mut sure_ways_out = if non_wall_neighbors.len() == 1 {
                            non_wall_neighbors
                        } else {
                            non_wall_neighbors
                                .iter()
                                .filter(|&&n| {
                                    self.row_missing_walls[n.y] == 0
                                        && self.col_missing_walls[n.x] == 0
                                })
                                .copied()
                                .collect::<Vec<_>>()
                        };

                        if sure_ways_out.len() > 1 {
                            return Err(());
                        }

                        if let Some(sure_way_out) = sure_ways_out.pop() {
                            debug_assert_eq!(self.level[sure_way_out], SmartSolverCell::Unknown);
                            self.put_hallway(sure_way_out);
                            changed = true;

                            for n in self
                                .level
                                .iter_neighbors(pos)
                                .filter(|&n| self.level[n] == SmartSolverCell::Unknown)
                                .collect::<Vec<_>>()
                            {
                                self.put_wall(n)?;
                            }

                            remove_unsatisfied_monsters.push(idx);
                        }
                    }
                    1 => {
                        // Put walls on remaining neighbors
                        for n in non_wall_neighbors {
                            if self.level[n] == SmartSolverCell::Unknown {
                                self.put_wall(n)?;
                                changed = true;
                            }
                        }
                        remove_unsatisfied_monsters.push(idx);
                    }
                    _ => {
                        return Err(());
                    }
                }
            }
            for idx in remove_unsatisfied_monsters.into_iter().rev() {
                self.unsatisfied_monsters.swap_remove(idx);
            }
        }

        Ok(())
    }

    fn check_full_validity(&self) -> bool {
        if self.row_missing_walls.iter().any(|&n| n > 0) {
            return false;
        }
        if self.col_missing_walls.iter().any(|&n| n > 0) {
            return false;
        }

        let mut col_numbers = vec![0; self.level.width()];
        let mut row_numbers = vec![0; self.level.height()];
        self.level.iter().for_each(|(&c, p)| {
            if c == SmartSolverCell::Wall {
                col_numbers[p.x] += 1;
                row_numbers[p.y] += 1;
            }
        });

        if row_numbers != self.row_total_walls || col_numbers != self.col_total_walls {
            return false;
        }

        // all dead ends are monsters, all monsters are on dead ends
        if !self.level.iter().all(|(&cell, pos)| {
            let num_neighbors = self.level.count_neighbors(pos, |_| true);
            let num_walls = self
                .level
                .count_neighbors(pos, |&n| n == SmartSolverCell::Wall);
            let is_monster = cell == SmartSolverCell::Monster;
            let is_dead_end = cell != SmartSolverCell::Wall && num_walls == (num_neighbors - 1);
            is_monster == is_dead_end
        }) {
            return false;
        }

        // treasure room always 3x3 with single entrance
        let mut treasure_tiles: HashSet<GridPos> = HashSet::new();
        for treasure_room in &self.placed_treasure_rooms {
            for dx in 0..3 {
                for dy in 0..3 {
                    treasure_tiles.insert((treasure_room.x + dx, treasure_room.y + dy).into());
                }
            }
        }

        // hallways always one square wide; no 2x2 blocks outside treasure rooms
        if !self.level.iter().all(|(&cell, pos)| {
            if treasure_tiles.contains(&pos) {
                return true;
            }
            if cell == SmartSolverCell::Hallway {
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

    fn possible_treasure_rooms(&self, treasure: GridPos) -> Vec<(GridPos, GridPos)> {
        let mut possible_rooms = Vec::new();
        for treasure_room_x in ((treasure.x as isize - 2).max(0) as usize)..=treasure.x {
            if treasure_room_x + 2 >= self.level.width() {
                break;
            }
            for treasure_room_y in ((treasure.y as isize - 2).max(0) as usize)..=treasure.y {
                if treasure_room_y + 2 >= self.level.height() {
                    break;
                }
                let room = (treasure_room_x, treasure_room_y).into();

                let mut valid_inside = true;
                for dx in 0..3 {
                    for dy in 0..3 {
                        let pos = (treasure_room_x + dx, treasure_room_y + dy).into();
                        match self.level[pos] {
                            SmartSolverCell::Wall | SmartSolverCell::Monster => {
                                valid_inside = false;
                                break;
                            }
                            SmartSolverCell::Treasure
                            | SmartSolverCell::Hallway
                            | SmartSolverCell::Unknown => {}
                        }
                    }
                    if !valid_inside {
                        break;
                    }
                }
                if !valid_inside {
                    continue;
                }

                let perimiter = self.get_room_peremiter(room);
                if perimiter.iter().any(|&pos| {
                    !matches!(
                        self.level[pos],
                        SmartSolverCell::Wall | SmartSolverCell::Hallway | SmartSolverCell::Unknown
                    )
                }) {
                    continue;
                }

                let perimiter_hallways = perimiter
                    .iter()
                    .filter(|&&p| self.level[p] == SmartSolverCell::Hallway)
                    .collect::<Vec<_>>();

                if perimiter_hallways.len() > 1 {
                    continue;
                }

                if perimiter_hallways.len() == 1 {
                    let exit = *perimiter_hallways[0];

                    possible_rooms.push((room, exit));
                    continue;
                }

                let potential_exits = perimiter
                    .into_iter()
                    .filter(|&p| {
                        self.level[p] == SmartSolverCell::Unknown
                            && (p.x >= room.x || p.x > 0)
                            && (p.y >= room.y || p.y > 0)
                            && (room.x + 2 >= p.x || p.x + 1 < self.level.width())
                            && (room.y + 2 >= p.y || p.y + 1 < self.level.height())
                    })
                    .collect::<Vec<_>>();

                for exit in potential_exits {
                    if self.level[exit] == SmartSolverCell::Unknown {
                        possible_rooms.push((room, exit));
                    }
                }
            }
        }
        possible_rooms
    }

    fn get_room_peremiter(&self, room: GridPos) -> Vec<GridPos> {
        let mut perimiter = Vec::new();

        // Left side
        if room.x > 0 {
            (room.y..room.y + 3).for_each(|y| {
                perimiter.push((room.x - 1, y).into());
            });
        }

        // Right side
        if room.x + 3 < self.level.width() {
            (room.y..room.y + 3).for_each(|y| {
                perimiter.push((room.x + 3, y).into());
            });
        }

        // Top side
        if room.y > 0 {
            (room.x..room.x + 3).for_each(|x| {
                perimiter.push((x, room.y - 1).into());
            });
        }

        // Bottom side
        if room.y + 3 < self.level.height() {
            (room.x..room.x + 3).for_each(|x| {
                perimiter.push((x, room.y + 3).into());
            });
        }

        perimiter
    }

    fn place_treasure_room(&mut self, room: GridPos, exit: GridPos) -> Result<(), ()> {
        if self.level[exit] == SmartSolverCell::Unknown {
            self.put_hallway(exit);
        }

        for dx in 0..3 {
            for dy in 0..3 {
                let pos = (room.x + dx, room.y + dy).into();
                if self.level[pos] == SmartSolverCell::Unknown {
                    self.put_hallway(pos);
                }
            }
        }

        for pos in self.get_room_peremiter(room) {
            if self.level[pos] == SmartSolverCell::Unknown {
                self.put_wall(pos)?;
            }
        }

        self.placed_treasure_rooms.push(room);

        Ok(())
    }

    pub fn all_solutions(&mut self) -> Vec<Level> {
        if self.fill_out_logical_values().is_err() {
            return Vec::new();
        }

        if self.check_full_validity() {
            return vec![Level::from(&self.level)];
        }

        let mut solutions = Vec::new();

        if let Some(unhandled_treasure) = self.unhandled_treasures.pop() {
            for (room, exit) in self.possible_treasure_rooms(unhandled_treasure) {
                let mut solver = self.clone();
                if solver.place_treasure_room(room, exit).is_ok() {
                    solutions.extend(solver.all_solutions());
                }
            }
        } else {
            while let Some(next_pos) = self.next_pos {
                if self.level[next_pos] != SmartSolverCell::Unknown {
                    self.next_pos = self.level.next_pos(&next_pos);
                } else {
                    break;
                }
            }

            if let Some(next_pos) = self.next_pos {
                self.next_pos = self.level.next_pos(&next_pos);

                let mut solver = self.clone();
                if solver.put_wall(next_pos).is_ok() {
                    solutions.extend(solver.all_solutions());
                }

                let mut solver = self.clone();

                if next_pos.x == 0
                    || next_pos.y == 0
                    || !matches!(
                        (
                            self.level[(next_pos.x - 1, next_pos.y).into()],
                            self.level[(next_pos.x, next_pos.y - 1).into()],
                            self.level[(next_pos.x - 1, next_pos.y - 1).into()],
                        ),
                        (
                            SmartSolverCell::Hallway,
                            SmartSolverCell::Hallway,
                            SmartSolverCell::Hallway
                        )
                    )
                {
                    solver.put_hallway(next_pos);
                    solutions.extend(solver.all_solutions());
                }
            }
        }

        solutions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_level_random() {
        for _ in 0..1000 {
            let level = Level::random(8, 8).unwrap();
            let mut solver = SmartSolver::from_level(&level);
            let solutions = solver.all_solutions();
            assert!(
                solutions.contains(&level),
                "Level {level:?} not found in solutions: {solutions:?}"
            );
        }
    }

    #[test]
    fn test_solve_regression() {
        let level = Level::from(
            &SmartSolverLevel::try_from(
                r#"
M#M#####
...M####
#.##..T#
#.M#...#
M.##...#
#..M#.#M
M#.##...
......#M
"#,
            )
            .unwrap(),
        );

        let mut solver = SmartSolver::from_level(&level);
        let solutions = solver.all_solutions();
        assert!(
            solutions.contains(&level),
            "Level {level:?} not found in solutions: {solutions:?}"
        );
    }

    #[test]
    fn test_solve_level_weird() {
        let mut solver = SmartSolver::try_from(
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
            &SmartSolverLevel::try_from(
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

    #[test]
    fn test_solve_level_weird2() {
        let mut solver = SmartSolver::try_from(
            r#"
  2 2 2 4 2 3 3 2
1 ? ? ? ? ? ? ? ?
2 ? ? ? ? ? ? ? ?
2 ? ? T ? ? ? ? ?
6 ? ? ? ? ? ? ? M
1 ? ? ? ? ? ? ? ?
3 M ? M ? M ? ? M
5 ? M ? M ? ? ? ?
0 M ? ? ? ? ? ? M
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SmartSolverLevel::try_from(
                r#"
...#....
.....##.
..T#..#.
#####.#M
.......#
M#M#M#.M
#M#M##.#
M......M
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }

    #[test]
    fn test_solve_level_nimble() {
        // Source of example: https://github.com/MischaU8/dungeons_diagrams/tree/bf29a0454aec28476ac80286e130feeaa4081dec?tab=readme-ov-file#usage

        let mut solver = SmartSolver::try_from(
            r#"
  1 4 2 7 0 4 4 4
3 ? ? ? ? ? ? ? ?
2 ? ? ? ? ? ? ? M
5 ? ? M ? ? ? ? ?
3 ? ? ? ? ? ? ? M
4 ? ? ? ? ? ? ? ?
1 ? T ? ? ? ? ? M
4 ? ? ? ? ? ? ? ?
4 ? ? ? ? ? ? ? M
"#,
        )
        .unwrap();
        let solutions = solver.all_solutions();

        let expected = Level::from(
            &SmartSolverLevel::try_from(
                r#"
.....###
.#.#...M
.#M#.###
.###...M
...#.###
.T.#...M
...#.###
####...M
"#,
            )
            .unwrap(),
        );

        assert_eq!(solutions, vec![expected]);
    }

    // Source of levels: https://www.reddit.com/r/puzzles/comments/d72zg1/advanced_dungeons_and_diagrams_map_making_logic/

    #[test]
    fn test_solve_sample() {
        let mut solver = SmartSolver::try_from(
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
            &SmartSolverLevel::try_from(
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
        let mut solver = SmartSolver::try_from(
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
            &SmartSolverLevel::try_from(
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
        let mut solver = SmartSolver::try_from(
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
            &SmartSolverLevel::try_from(
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
        let mut solver = SmartSolver::try_from(
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
            &SmartSolverLevel::try_from(
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
