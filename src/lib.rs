use std::collections::HashMap;

use glam::{IVec2, Vec2};

pub const CHUNK_SIZE: usize = 16;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct EIndex {
    x: u32,
    y: u32,
}
impl From<(i32, i32)> for EIndex {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: (i32::MAX as i64 + 1 + value.0 as i64) as u32,
            y: (i32::MAX as i64 + 1 + value.1 as i64) as u32,
        }
    }
}
impl From<[i32; 2]> for EIndex {
    fn from(value: [i32; 2]) -> Self {
        let tuple = (value[0], value[1]);
        tuple.into()
    }
}
impl From<IVec2> for EIndex {
    fn from(value: IVec2) -> Self {
        let tuple = (value.x, value.y);
        tuple.into()
    }
}

impl EIndex {
    pub fn chunk_index(&self) -> EIndex {
        EIndex {
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

pub struct ERay {
    pub origin: Vec2,
    pub dir: Vec2,
}

/// An endless 2D grid of type `T`
#[derive(Default)]
pub struct EGrid<T> {
    pub top_left:IVec2,
    pub bottom_right:IVec2,
    pub chunks: HashMap<EIndex, Vec<Option<T>>>,
}

pub struct Visit<'a, T> {
    pub index:(i32, i32),
    pub cell:&'a T,
    pub x:f32,
    pub y:f32,
    pub d:f32
}

impl<T: Clone> EGrid<T> {
    pub fn get(&self, index: impl Into<EIndex>) -> Option<&T> {
        let index: EIndex = index.into();
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get(&chunk_index)?;
        let cell = chunk.get(index.local_index())?;
        let cell = cell.as_ref()?;
        Some(cell)
    }

    pub fn get_mut(&mut self, index: impl Into<EIndex>) -> Option<&mut T> {
        let index: EIndex = index.into();
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get_mut(&chunk_index)?;
        let cell = chunk.get_mut(index.local_index())?;
        let cell = cell.as_mut()?;
        Some(cell)
    }

    pub fn insert(&mut self, index: impl Into<EIndex>, t: T) {
        let index: EIndex = index.into();
        let chunk_index = index.chunk_index();
        let chunk = match self.chunks.get_mut(&chunk_index) {
            Some(chunk) => chunk,
            None => {
                let chunk = vec![None; CHUNK_SIZE * CHUNK_SIZE];
                self.chunks.insert(chunk_index.clone(), chunk);
                self.chunks.get_mut(&chunk_index).unwrap()
            }
        };
        if let Some(cell) = chunk.get_mut(index.local_index()) {
            *cell = Some(t);
        }
    }

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
                // dt = ((tile + 1.0 ) * cell_size - pos) / dir;
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
                    if f(Visit {index, cell, d:t, x:tile_x, y:tile_y }) {
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
        } else {
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raycast_test() {}

    #[test]
    fn infindex() {
        let p1: EIndex = [0, 0].into();
        let p2: EIndex = (0, 0).into();
        assert_eq!(p1, p2);
        let p1: EIndex = [5, 3].into();
        let p2: EIndex = (5, 3).into();
        assert_eq!(p1, p2);
        let p1: EIndex = [1, 2].into();
        let p2: EIndex = (2, 1).into();
        assert_ne!(p1, p2);
        let p1: EIndex = [i32::MIN, i32::MAX].into();
        let p2: EIndex = (i32::MIN, i32::MAX).into();
        assert_eq!(p1, p2);

        let p1: EIndex = [0, 0].into();
        let p2: EIndex = [15, 15].into();
        assert_ne!(p1, p2);
        assert_eq!(p1.chunk_index(), p2.chunk_index());

        let p1: EIndex = [-7, -7].into();
        let p2: EIndex = [-9, -9].into();
        assert_ne!(p1, p2);
        assert_eq!(p1.chunk_index(), p2.chunk_index());
    }

    #[test]
    fn test() {
        let mut grid = EGrid::default() as EGrid<(i32, i32)>;
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
    }
}
