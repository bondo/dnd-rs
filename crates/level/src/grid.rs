use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GridPos {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl From<(usize, usize)> for GridPos {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

impl Debug for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Grid<C> {
    cells: Vec<C>,
    width: usize,
    height: usize,
}

impl<C> Index<GridPos> for Grid<C> {
    type Output = C;

    fn index(&self, GridPos { x, y }: GridPos) -> &Self::Output {
        &self.cells[y * self.width + x]
    }
}

impl<C> IndexMut<GridPos> for Grid<C> {
    fn index_mut(&mut self, GridPos { x, y }: GridPos) -> &mut Self::Output {
        &mut self.cells[y * self.width + x]
    }
}

impl<C> Grid<C> {
    pub(crate) fn new(width: usize, height: usize, fill: C) -> Self
    where
        C: Clone,
    {
        Self {
            cells: vec![fill; width * height],
            width,
            height,
        }
    }

    pub(crate) fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&C, GridPos)> {
        self.cells
            .iter()
            .enumerate()
            .map(|(idx, c)| (c, (idx % self.width, idx / self.width).into()))
    }

    pub(crate) fn iter_neighbors(
        &self,
        GridPos { x, y }: GridPos,
    ) -> impl Iterator<Item = GridPos> + use<'_, C> {
        [(-1, 0), (0, -1), (1, 0), (0, 1)]
            .into_iter()
            .filter_map(move |(dx, dy)| {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    Some((nx as usize, ny as usize).into())
                } else {
                    None
                }
            })
    }

    pub(crate) fn next_pos(&self, pos: &GridPos) -> Option<GridPos> {
        if pos.x + 1 < self.width {
            Some((pos.x + 1, pos.y).into())
        } else if pos.y + 1 < self.height {
            Some((0, pos.y + 1).into())
        } else {
            None
        }
    }

    pub(crate) fn count_row<F>(&self, y: usize, mut f: F) -> usize
    where
        F: FnMut(&C) -> bool,
    {
        (0..self.width).filter(|&x| f(&self[(x, y).into()])).count()
    }

    pub(crate) fn count_col<F>(&self, x: usize, mut f: F) -> usize
    where
        F: FnMut(&C) -> bool,
    {
        (0..self.height)
            .filter(|&y| f(&self[(x, y).into()]))
            .count()
    }

    pub(crate) fn count_neighbors<F>(&self, p: GridPos, mut f: F) -> usize
    where
        F: FnMut(&C) -> bool,
    {
        self.iter_neighbors(p).filter(|p| f(&self[*p])).count()
    }

    pub(crate) fn filtered_neighbors<F>(&self, p: GridPos, mut f: F) -> Vec<GridPos>
    where
        F: FnMut(&C) -> bool,
    {
        self.iter_neighbors(p).filter(|p| f(&self[*p])).collect()
    }

    pub(crate) fn map<F, T>(&self, mut f: F) -> Grid<T>
    where
        F: FnMut(&C, GridPos) -> T,
        T: Clone,
    {
        Grid {
            cells: self.iter().map(|(c, p)| f(c, p)).collect::<Vec<T>>(),
            width: self.width,
            height: self.height,
        }
    }
}

impl<C: Debug> Debug for Grid<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{:?}", self[(x, y).into()])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
