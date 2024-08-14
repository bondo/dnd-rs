use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

pub(super) struct Grid<C> {
    cells: Vec<C>,
    width: usize,
    height: usize,
}

impl<C> Index<(usize, usize)> for Grid<C> {
    type Output = C;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[y * self.width + x]
    }
}

impl<C> IndexMut<(usize, usize)> for Grid<C> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
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

    #[allow(dead_code)]
    pub(super) fn iter(&self) -> GridIterator<C> {
        GridIterator { grid: self, i: 0 }
    }

    pub(super) fn neighbors(&self, (x, y): (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        if y > 0 {
            neighbors.push((x, y - 1));
        }
        if x + 1 < self.width {
            neighbors.push((x + 1, y));
        }
        if y + 1 < self.height {
            neighbors.push((x, y + 1));
        }
        neighbors
    }

    pub(super) fn count_neighbors<F>(&self, p: (usize, usize), mut f: F) -> usize
    where
        F: FnMut(&C) -> bool,
    {
        self.neighbors(p)
            .into_iter()
            .filter(|p| f(&self[*p]))
            .count()
    }

    pub(super) fn filtered_neighbors<F>(&self, p: (usize, usize), mut f: F) -> Vec<(usize, usize)>
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
        F: FnMut(&C, (usize, usize)) -> T,
        T: Clone,
    {
        Grid {
            cells: self
                .cells
                .iter()
                .enumerate()
                .map(|(idx, c)| f(c, (idx % self.width, idx / self.width)))
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
                write!(f, "{:?}", self[(x, y)])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub(super) struct GridIterator<'a, C> {
    grid: &'a Grid<C>,
    i: usize,
}

impl<'a, C: Copy> Iterator for GridIterator<'a, C> {
    type Item = (usize, usize, C);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.grid.cells.len() {
            let x = self.i % self.grid.width;
            let y = self.i / self.grid.width;
            let cell = self.grid.cells[self.i];
            self.i += 1;
            Some((x, y, cell))
        } else {
            None
        }
    }
}
