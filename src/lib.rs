use std::collections::{hash_map::{Values, ValuesMut}, HashMap};
use glam::Vec2;
use serde::{Deserialize, Serialize};
pub const CHUNK_SIZE: usize = 16;

/// Index used internally to identify an element within a cell
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Index {
    x: u32,
    y: u32,
}

/// Index used internally to identify a chunk within a grid
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ChunkIndex {
    x: u32,
    y: u32,
}

impl From<(i32, i32)> for Index {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: (i32::MAX as i64 + 1 + value.0 as i64) as u32,
            y: (i32::MAX as i64 + 1 + value.1 as i64) as u32,
        }
    }
}
impl From<Index> for (i32, i32) {
    fn from(value: Index) -> Self {
        let x = value.x as i64 - i32::MAX as i64 - 1;
        let y = value.y as i64 - i32::MAX as i64 - 1;
        (x as i32, y as i32)
    }
}

impl Index {
    pub fn chunk_index(&self) -> ChunkIndex {
        ChunkIndex {
            x: self.x / CHUNK_SIZE as u32,
            y: self.y / CHUNK_SIZE as u32,
        }
    }
    
    pub fn local_index(&self) -> usize {
        let x = self.x as usize % CHUNK_SIZE;
        let y = self.y as usize % CHUNK_SIZE;
        y * CHUNK_SIZE + x
    }
}

impl ChunkIndex {
    pub fn index(&self) -> Index {
        Index {
            x: self.x * CHUNK_SIZE as u32,
            y: self.y * CHUNK_SIZE as u32
        }
    }
}

/// A `Chunk` of the `Grid`
#[derive(Serialize, Deserialize)]
pub struct Chunk<T> {
    index:ChunkIndex,
    len:u16,
    inner:Vec<Option<T>>
}

impl<T:Clone> Default for Chunk<T> {
    fn default() -> Self {
        Self { index:Index::from((0, 0)).chunk_index(), len:0, inner: Vec::new() }
    }
}

impl<T:Clone> Chunk<T> {
    /// Gets the top left index of the chunk
    pub fn top_left(&self) -> (i32, i32) {
        self.index.index().into()
    }

    /// Gets the bottom right index of the chunk
    pub fn bottom_right(&self) -> (i32, i32) {
        let p:(i32, i32) = self.index.index().into();
        (p.0 + CHUNK_SIZE as i32 - 1, p.1 + CHUNK_SIZE as i32 - 1)
    }

    /// Get length of the chunk, i.e. how many elements are in the chunk.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Clear all elements from the chunk
    pub fn clear(&mut self) {
        self.len = 0;
        self.inner = Vec::default();
    }

    /// Get element in chunk using local position within the chunk
    pub fn get_local(&self, local:usize) -> Option<&Option<T>> {
        self.inner.get(local)
    }

    /// Insert element into local position
    pub fn insert(&mut self, local:usize, t:T) {
        if self.inner.is_empty() {
            self.inner = vec![None; CHUNK_SIZE * CHUNK_SIZE];
            self.len = 0;
        }
        if self.inner[local].is_none() {
            self.len += 1;
        }
        self.inner[local] = Some(t);
    }

    /// Get element in chunk using local position within the `chunk`
    pub fn get_local_mut(&mut self, local:usize) -> Option<&mut T> {
        let m = self.inner.get_mut(local)?;
        m.as_mut()
    }
}

pub struct ChunkIter<'a, T> {
    index:usize,
    top_left:(i32, i32),
    iter:core::slice::Iter<'a, Option<T>>,
}
impl<'a, T> Iterator for ChunkIter<'a, T> {
    type Item = ((i32, i32), &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            if let Some(cell) = next {
                let index = (self.top_left.0 + self.index as i32 % CHUNK_SIZE as i32, self.top_left.1 + self.index as i32 / CHUNK_SIZE as i32);
                self.index += 1;
                return Some((index, cell));
            }
            self.index += 1;
            continue;
        }

        None
    }
}

pub struct ChunkIterMut<'a, T> {
    index:usize,
    top_left:(i32, i32),
    iter:core::slice::IterMut<'a, Option<T>>,
}
impl<'a, T> Iterator for ChunkIterMut<'a, T> {
    type Item = ((i32, i32), &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            if let Some(cell) = next {
                let index = (self.top_left.0 + self.index as i32 % CHUNK_SIZE as i32, self.top_left.1 + self.index as i32 / CHUNK_SIZE as i32);
                self.index += 1;
                return Some((index, cell));
            }
            self.index += 1;
            continue;
        }

        None
    }
}

impl<'a, T:Clone> IntoIterator for &'a Chunk<T> {
    type Item = ((i32, i32), &'a T);
    type IntoIter = ChunkIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            index:0,
            iter: self.inner.iter(),
            top_left: self.top_left()
        }
    }
}

impl<'a, T:Clone> IntoIterator for &'a mut Chunk<T> {
    type Item = ((i32, i32), &'a mut T);
    type IntoIter = ChunkIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let top_left = self.top_left();
        Self::IntoIter {
            index:0,
            iter: self.inner.iter_mut(),
            top_left: top_left
        }
    }
}

/// An endless 2D grid of type `T` implemented using chunks
#[derive(Default, Serialize, Deserialize)]
pub struct Grid<T> {
    chunks: HashMap<ChunkIndex, Chunk<T>>,
}

impl<'a, T> IntoIterator for &'a Grid<T> {
    type Item = &'a Chunk<T>;

    type IntoIter = Values<'a, ChunkIndex, Chunk<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks.values()
    }
}


impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type Item = &'a mut Chunk<T>;

    type IntoIter = ValuesMut<'a, ChunkIndex, Chunk<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks.values_mut()
    }
}

/// Struct used by the `Grid::cast_ray`
pub struct RayVisit<'a, T> {
    /// Current index of the cell being visited
    pub index:(i32, i32),

    /// The cell being visited
    pub cell:&'a T,

    /// Current position of the ray
    pub pos:(f32, f32),
   
    // Distance traveled by the ray
    pub d:f32
}

/// Struct used by the `Grid::astar`
pub struct AStarVisit<'a, T> {
    /// Current index of the cell being visited
    pub index:(i32, i32),

    /// The cell being visited
    pub cell:&'a T,
}

impl<T: Clone> Grid<T> {
    /// Gets length of the grid, aka. how many cells there are
    pub fn len(&self) -> usize {
        let mut len = 0;
        self.chunks.values().for_each(|x|len += x.len());
        len
    }
    /// Gets a immutable reference to `T`
    pub fn get(&self, index: impl Into<(i32, i32)>) -> Option<&T> {
        let index:(i32, i32) = index.into();
        let index = Index::from(index);
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get(&chunk_index)?;
        let cell = chunk.get_local(index.local_index())?;
        let cell = cell.as_ref()?;
        Some(cell)
    }

    /// Gets an mutable reference to `T`
    pub fn get_mut(&mut self, index: impl Into<(i32, i32)>) -> Option<&mut T> {
        let index:(i32, i32) = index.into();
        let index:Index = index.into();
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get_mut(&chunk_index)?;
        chunk.get_local_mut(index.local_index())
    }

    /// Insert `T`
    pub fn insert(&mut self, index: impl Into<(i32, i32)>, t: T) {
        let index:(i32, i32) = index.into();
        let index:Index = index.into();
        let chunk_index = index.chunk_index();
        let chunk = match self.chunks.get_mut(&chunk_index) {
            Some(chunk) => chunk,
            None => {
                let mut chunk = Chunk::default();
                chunk.index = chunk_index;
                self.chunks.insert(chunk_index, chunk);
                self.chunks.get_mut(&chunk_index).unwrap()
            }
        };
        let local = index.local_index();
        chunk.insert(local, t);
    }

    /// Perform the A-star algorithm
    /// `F is a function which returns `false` when path is blocked and `true` when not blocked
    pub fn astar<F:Fn(AStarVisit<T>)->bool>(&self, start:impl Into<(i32, i32)>, end:impl Into<(i32, i32)>, visit:F) -> Option<Vec<(i32, i32)>> {
        let start = start.into();
        let end = end.into();
        let p = pathfinding::directed::astar::astar(&start, |(nx, ny)| {
            let (nx, ny) = (*nx, *ny);
            let mut vec:Vec<((i32, i32), i32)> = Vec::with_capacity(4);
            for p in [(nx - 1, ny), (nx + 1, ny), (nx, ny - 1), (nx, ny + 1)] {
                if let Some(tile) = self.get(p) {
                    if !visit(AStarVisit {
                        index: p,
                        cell: tile,
                    }) {
                        vec.push((p, 1));
                    }
                }
            }
            vec
        }, |(nx, ny)|{
            let (vx, vy) = ((nx - end.0).abs(), (ny - end.1).abs());
            vx + vy
        }, |n|{
            n == &end
        });
        if let Some((vec, _)) = p {
            return Some(vec);
        }

        None
    }

    /// Casts a ray from `start` to `end` and call a function `F` for each cell visited
    /// 
    /// The ray will be traced until `F` returns `false` or untill `end` has been reached
    pub fn cast_ray<F:FnMut(RayVisit<T>)->bool>(&self, start:impl Into<(f32, f32)>, end:impl Into<(f32, f32)>, mut f:F) {
        let start:(f32, f32) = start.into();
        let end:(f32, f32) = end.into();
        let start:Vec2 = start.into();
        let end:Vec2 = end.into();
        fn get_helper(cell_size:f32, pos:f32, dir:f32) -> (f32, f32, f32, f32) {
            let tile = (pos / cell_size).floor();// + 1.0;
            let dtile;
            let dt;
            if dir > 0.0 {
                dtile = 1.0;
                dt = ((tile + 1.0) * cell_size - pos) / dir;
            } else {
                dtile = -1.0;
                dt = (tile  * cell_size - pos) / dir;
            }
    
            (tile, dtile, dt, dtile * cell_size / dir)
        }
        let v = end - start;
        let dir = v.normalize_or_zero();
        if dir.length() == 0.0 {
            return;
        }
        let (mut tile_x, dtile_x, mut dt_x, ddt_x) = get_helper(1.0, start.x, dir.x);
        let (mut tile_y, dtile_y, mut dt_y, ddt_y) = get_helper(1.0, start.y, dir.y);
    
        let mut t = 0.0;
        if dir.x*dir.x + dir.y*dir.y > 0.0 {
            loop {
                if v.length() < t {
                    break;
                }
                let index = (tile_x as i32, tile_y as i32);
                if let Some(cell) = self.get(index) {
                    if !f(RayVisit {index, cell, d:t, pos:(tile_x, tile_y) }) {
                        break;
                    }
                } else {
                    break;
                }
                if dt_x < dt_y {
                    tile_x += dtile_x;
                    let dt = dt_x;
                    t += dt;
                    dt_x = dt_x + ddt_x - dt;
                    dt_y -= dt;
                } else {
                    tile_y += dtile_y;
                    let dt = dt_y;
                    t += dt;
                    dt_x -= dt;
                    dt_y = dt_y + ddt_y - dt;
                }
            }
        } 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_test() {
        let p1: Index = (0, 0).into();
        let p2: Index = (0, 0).into();
        assert_eq!(p1, p2);
        let p1: Index = (5, 3).into();
        let p2: Index = (5, 3).into();
        assert_eq!(p1, p2);
        let p1: Index = (1, 2).into();
        let p2: Index = (2, 1).into();
        assert_ne!(p1, p2);
        let p1: Index = (i32::MIN, i32::MAX).into();
        let p2: Index = (i32::MIN, i32::MAX).into();
        assert_eq!(p1, p2);

        let p1: Index = (0, 0).into();
        let p2: Index = (15, 15).into();
        assert_ne!(p1, p2);
        assert_eq!(p1.chunk_index(), p2.chunk_index());

        let p1: Index = (-7, -7).into();
        let p2: Index = (-9, -9).into();
        assert_ne!(p1, p2);
        assert_eq!(p1.chunk_index(), p2.chunk_index());

        let p1: Index = (CHUNK_SIZE as i32 * 10, CHUNK_SIZE as i32 * 10).into();
        let p2 = p1.chunk_index().index();
        assert_eq!(p1, p2);
        let p1: Index = (CHUNK_SIZE as i32 * -10, CHUNK_SIZE as i32 * -10).into();
        let p2 = p1.chunk_index().index();
        assert_eq!(p1, p2);

        let p1 = (1024, 5431);
        let p2:Index = p1.into();
        let p2:(i32, i32) = p2.into();
        assert_eq!(p1, p2);

        let p1 = (-1024, -5431);
        let p2:Index = p1.into();
        let p2:(i32, i32) = p2.into();
        assert_eq!(p1, p2);
    }

    #[test]
    fn chunk_test() {
        #[derive(Clone)]
        struct Test;
        let mut chunk = Chunk::default() as Chunk<Test>;
        assert_eq!(chunk.inner.len(), 0);
        assert_eq!(chunk.len(), 0);
        chunk.insert(0, Test);
        assert_eq!(chunk.inner.len(), CHUNK_SIZE * CHUNK_SIZE);
        assert_eq!(chunk.len(), 1);
        chunk.insert(0, Test);
        assert_eq!(chunk.len(), 1);
        chunk.insert(1, Test);
        assert_eq!(chunk.len(), 2);
        chunk.clear();
        assert_eq!(chunk.len(), 0);
        assert_eq!(chunk.inner.len(), 0);

        let chunk = Chunk::default() as Chunk<Test>;
        assert_eq!(chunk.top_left(), (0, 0));
        assert_eq!(chunk.bottom_right(), (CHUNK_SIZE as i32 -1, CHUNK_SIZE as i32 -1));

        let mut chunk = Chunk::default() as Chunk<Test>;
        let p = (-1024, -1024);
        let chunk_index = Index::from(p).chunk_index();
        chunk.index = chunk_index;

        assert_eq!(chunk.top_left(), p);
        assert_eq!(chunk.bottom_right(), (p.0 + CHUNK_SIZE as i32 - 1, p.1 + CHUNK_SIZE as i32 -1 ));
        //assert_eq!(chunk.bottom_right(), (CHUNK_SIZE as i32 -1, CHUNK_SIZE as i32 -1));
    }

    #[test]
    fn grid_test() {
        let mut grid = Grid::default() as Grid<(i32, i32)>;
        let size = 33;
        for y in -size..size {
            for x in -size..size {
                let p = (x, y);
                grid.insert(p, p);
                let p2 = grid.get(p).unwrap();
                assert_eq!(&p, p2);
                grid.get_mut(p).unwrap().0 = 0;
                grid.get_mut(p).unwrap().1 = 0;
                let p2 = grid.get_mut(p).unwrap();
                assert_eq!(p2, &mut (0, 0));
            }
        }

        let bincoded = bincode::serialize(&grid).unwrap();
        let grid2:Grid<(i32, i32)> = bincode::deserialize(&bincoded).unwrap();
        for y in -size..size {
            for x in -size..size {
                let p = (x, y);
                let g1 = grid.get(p);
                let g2 = grid2.get(p);
                assert_eq!(g1, g2);
            }
        }
    }

    #[test]
    fn grid_test2() {
        let mut grid = Grid::default() as Grid<(i32, i32)>;
        let size = 64;
        for y in 0..size {
            for x in 0..size {
                let p = (x, y);
                grid.insert(p, p);
            }
        }
        assert_eq!(grid.len(), size as usize * size as usize);
    }

    #[test]
    fn grid_test3() {
        let mut grid = Grid::default() as Grid<(i32, i32)>;
        let size: i32 = 33;
        let mut inserted = 0;
        for y in -size..size {
            for x in -size..size {
                let p = (x, y);
                grid.insert(p, p);
                inserted += 1;
            }
        }

        let mut read = 0;
        for chunk in &grid {
            for (p, cell) in chunk {
                assert_eq!(p, *cell);
                read += 1;
            }
        }
        assert_eq!(read, inserted);

        let mut read = 0;
        for chunk in &mut grid {
            for (_, cell) in chunk {
                *cell = Default::default();
                assert_eq!((0,0), *cell);
                read += 1;
            }
        }
        assert_eq!(read, inserted);
    }

    #[test]
    fn grid_test4() {
        let mut grid = Grid::default() as Grid<(i32, i32)>;
        let values = vec![(14, 0), (-1011,32), (-6654,-213), (5543,123), (65645, 12312), (0, 0)];
        for v in values.iter() {
            grid.insert(v.to_owned(), v.to_owned());
        }
        
        let mut count = 0;
        for chunk in &grid {
            for cell in chunk {
                assert_eq!(cell.0.to_owned(), cell.1.to_owned());
                count += 1;
            }
        }

        assert_eq!(count, values.len());
        assert_eq!(grid.len(), values.len());
    }

    #[test]
    fn grid_serde_test() {
        let mut grid = Grid::default() as Grid<(i32, i32)>;
        let size = 64;
        for y in -size..size {
            for x in -size..size {
                let p = (x, y);
                grid.insert(p, p);
            }
        }

        let bincoded = bincode::serialize(&grid).unwrap();
        let grid2:Grid<(i32, i32)> = bincode::deserialize(&bincoded).unwrap();
        for y in -size..size {
            for x in -size..size {
                let p = (x, y);
                let g1 = grid.get(p);
                let g2 = grid2.get(p);
                assert_eq!(g1, g2);
            }
        }
    }

    #[test]
    fn raycast_test() {
        let mut grid = Grid::default() as Grid<bool>;
        for y in 0..8 {
            for x in 0..8 {
                grid.insert((x, y), x == 4 && y == 4);
            }
        }
        let mut last_hit = (0, 0);
        let mut last_pos_before_hit = (0.0, 0.0);
        grid.cast_ray((0.5, 0.5), (7.5, 7.5), |v| {
            if *v.cell {
                last_hit = v.index;
                return false;
            } 
            last_pos_before_hit = v.pos;

            true
        });

        assert_eq!(last_hit, (4, 4));
        assert_eq!(last_pos_before_hit, (3.0, 4.0));
    }

    #[test]
    fn astar_test() {
        let mut grid = Grid::default() as Grid<bool>;
        for y in 0..8 {
            for x in 0..8 {
                grid.insert((x, y), x == 4 && y != 7);
            }
        }

        assert!(!(*grid.get((4,7)).unwrap()));

        let path = grid.astar((0,0), (7,0), |x|{
            *x.cell
        });

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(*path.last().unwrap(), (7,0));
        assert_eq!(*path.first().unwrap(), (0,0));
        assert!(path.iter().any(|x| *x == (4,7)));
    }
}
