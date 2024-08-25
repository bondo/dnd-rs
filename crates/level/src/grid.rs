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

    pub(crate) fn iter(&self) -> GridIterator<C> {
        GridIterator {
            inner: Box::new(
                self.cells
                    .iter()
                    .enumerate()
                    .map(|(idx, c)| (c, (idx % self.width, idx / self.width).into())),
            ),
        }
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

    pub(crate) fn neighbors(&self, GridPos { x, y }: GridPos) -> Vec<GridPos> {
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

    #[cfg(test)]
    pub(crate) fn count_row<F>(&self, y: usize, mut f: F) -> usize
    where
        F: FnMut(&C) -> bool,
    {
        (0..self.width).filter(|&x| f(&self[(x, y).into()])).count()
    }

    #[cfg(test)]
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
        self.neighbors(p)
            .into_iter()
            .filter(|p| f(&self[*p]))
            .count()
    }

    pub(crate) fn filtered_neighbors<F>(&self, p: GridPos, mut f: F) -> Vec<GridPos>
    where
        F: FnMut(&C) -> bool,
    {
        self.neighbors(p)
            .into_iter()
            .filter(|p| f(&self[*p]))
            .collect()
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

pub(crate) struct GridIterator<'a, C> {
    inner: Box<dyn Iterator<Item = (&'a C, GridPos)> + 'a>,
}

impl<'a, C> Iterator for GridIterator<'a, C> {
    type Item = (&'a C, GridPos);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
