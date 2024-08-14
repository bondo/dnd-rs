use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct GridPos {
    pub(super) x: usize,
    pub(super) y: usize,
}

impl From<(usize, usize)> for GridPos {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

pub(super) struct Grid<C> {
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
    pub(super) fn new(width: usize, height: usize, fill: C) -> Self
    where
        C: Clone,
    {
        Self {
            cells: vec![fill; width * height],
            width,
            height,
        }
    }

    pub(super) fn width(&self) -> usize {
        self.width
    }

    pub(super) fn height(&self) -> usize {
        self.height
    }

    pub(super) fn iter(&self) -> GridIterator<C> {
        GridIterator {
            inner: self.cells.iter(),
        }
    }

    pub(super) fn neighbors(&self, GridPos { x, y }: GridPos) -> Vec<GridPos> {
        let mut neighbors: Vec<GridPos> = Vec::new();
        if x > 0 {
            neighbors.push((x - 1, y).into());
        }
        if y > 0 {
            neighbors.push((x, y - 1).into());
        }
        if x + 1 < self.width {
            neighbors.push((x + 1, y).into());
        }
        if y + 1 < self.height {
            neighbors.push((x, y + 1).into());
        }
        neighbors
    }

    pub(super) fn count_neighbors<F>(&self, p: GridPos, mut f: F) -> usize
    where
        F: FnMut(&C) -> bool,
    {
        self.neighbors(p)
            .into_iter()
            .filter(|p| f(&self[*p]))
            .count()
    }

    pub(super) fn filtered_neighbors<F>(&self, p: GridPos, mut f: F) -> Vec<GridPos>
    where
        F: FnMut(&C) -> bool,
    {
        self.neighbors(p)
            .into_iter()
            .filter(|p| f(&self[*p]))
            .collect()
    }

    pub(super) fn map<F, T>(&self, mut f: F) -> Grid<T>
    where
        F: FnMut(&C, GridPos) -> T,
        T: Clone,
    {
        Grid {
            cells: self
                .cells
                .iter()
                .enumerate()
                .map(|(idx, c)| f(c, (idx % self.width, idx / self.width).into()))
                .collect(),
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

pub(super) struct GridIterator<'a, C> {
    inner: std::slice::Iter<'a, C>,
}

impl<'a, C: Copy> Iterator for GridIterator<'a, C> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}
