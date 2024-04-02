use std::collections::HashMap;
use glam::Vec2;
use serde::{Deserialize, Serialize};
pub const CHUNK_SIZE: usize = 16;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct Index {
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

impl Index {
    pub fn chunk_index(&self) -> Index {
        Index {
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

/// An endless 2D grid of type `T` implemented using chunks
#[derive(Default)]
pub struct Grid<T> {
    chunks: HashMap<Index, Vec<Option<T>>>,
}

/// Struct used by the `cast_ray` for `Grid`
pub struct Visit<'a, T> {
    pub index:(i32, i32),
    pub t:&'a T,
    pub x:f32,
    pub y:f32,
    pub d:f32
}

impl<T: Clone> Grid<T> {
    /// Gets a immutable reference to `T`
    pub fn get(&self, index: impl Into<(i32, i32)>) -> Option<&T> {
        let index:(i32, i32) = index.into();
        let index = Index::from(index);
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get(&chunk_index)?;
        let cell = chunk.get(index.local_index())?;
        let cell = cell.as_ref()?;
        Some(cell)
    }

    /// Gets an mutable reference to `T`
    pub fn get_mut(&mut self, index: impl Into<(i32, i32)>) -> Option<&mut T> {
        let index:(i32, i32) = index.into();
        let index:Index = index.into();
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get_mut(&chunk_index)?;
        let cell = chunk.get_mut(index.local_index())?;
        let cell = cell.as_mut()?;
        Some(cell)
    }

    /// Insert `T`
    pub fn insert(&mut self, index: impl Into<(i32, i32)>, t: T) {
        let index:(i32, i32) = index.into();
        let index:Index = index.into();
        let chunk_index = index.chunk_index();
        let chunk = match self.chunks.get_mut(&chunk_index) {
            Some(chunk) => chunk,
            None => {
                let chunk = vec![None; CHUNK_SIZE * CHUNK_SIZE];
                self.chunks.insert(chunk_index, chunk);
                self.chunks.get_mut(&chunk_index).unwrap()
            }
        };
        if let Some(cell) = chunk.get_mut(index.local_index()) {
            *cell = Some(t);
        }
    }

    /// Perform the A-star algorithm
    pub fn astar<F:Fn((i32, i32), &T)->bool>(&self, start:(i32, i32), end:(i32, i32), visit:F) -> Option<Vec<(i32, i32)>> {
        let p = pathfinding::directed::astar::astar(&start, |(nx, ny)| {
            let (nx, ny) = (*nx, *ny);
            let mut vec:Vec<((i32, i32), i32)> = Vec::with_capacity(4);
            for p in [(nx - 1, ny), (nx + 1, ny), (nx, ny - 1), (nx, ny + 1)] {
                if let Some(tile) = self.get(p) {
                    if !visit(p, tile) {
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
    pub fn cast_ray<F:FnMut(Visit<T>)->bool>(&self, start:Vec2, end:Vec2, mut f:F) {
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
                    if f(Visit {index, t: cell, d:t, x:tile_x, y:tile_y }) {
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


impl<T:Serialize> Serialize for Grid<T> {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        Serialize::serialize(&self.chunks, serializer)
    }
}

impl<'de, T:Deserialize<'de>> Deserialize<'de> for Grid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let res = HashMap::deserialize(deserializer);
        match res {
            Ok(chunks) => return Ok(Self {
                chunks,
            }),
            Err(err) => return Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raycast_test() {}

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
    }

    #[test]
    fn grid_test() {
        let mut grid = Grid::default() as Grid<(i32, i32)>;
        let size = 64;
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
}
