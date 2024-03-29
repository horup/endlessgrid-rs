use std::collections::HashMap;

use glam::{IVec2, Vec2};

pub const CHUNK_SIZE:usize = 16;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct EIndex {
    x:u32,
    y:u32
}
impl From<(i32, i32)> for EIndex {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x:(i32::MAX as i64 + 1 + value.0 as i64) as u32, 
            y:(i32::MAX as i64 + 1 + value.1 as i64) as u32
        }
    }
}
impl From<[i32;2]> for EIndex {
    fn from(value: [i32;2]) -> Self {
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
        EIndex { x: self.x / CHUNK_SIZE as u32, y: self.y / CHUNK_SIZE as u32 }
    }
    pub fn local_index(&self) -> usize {
        let x = self.x as usize % CHUNK_SIZE;
        let y = self.y as usize % CHUNK_SIZE;
        y * CHUNK_SIZE + x
    }
}

pub struct ERay {
    pub origin:Vec2,
    pub dir:Vec2
}

/// An endless 2D grid of type `T`
#[derive(Default)]
pub struct EGrid<T> {
    pub chunks:HashMap<EIndex, Vec<Option<T>>>
}

impl<T:Clone> EGrid<T> {
    pub fn get(&self, index:impl Into<EIndex>) -> Option<&T> {
        let index:EIndex = index.into();
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get(&chunk_index)?;
        let cell = chunk.get(index.local_index())?;
        let cell = cell.as_ref()?;
        Some(cell)
    }

    pub fn get_mut(&mut self, index:impl Into<EIndex>) -> Option<&mut T> {
        let index:EIndex = index.into();
        let chunk_index = index.chunk_index();
        let chunk = self.chunks.get_mut(&chunk_index)?;
        let cell = chunk.get_mut(index.local_index())?;
        let cell = cell.as_mut()?;
        Some(cell)
    }

    pub fn insert(&mut self, index:impl Into<EIndex>, t:T) {
        let index:EIndex = index.into();
        let chunk_index = index.chunk_index();
        let chunk = match self.chunks.get_mut(&chunk_index) {
            Some(chunk) => chunk,
            None => {
                let chunk = vec![None;CHUNK_SIZE*CHUNK_SIZE];
                self.chunks.insert(chunk_index.clone(), chunk);
                self.chunks.get_mut(&chunk_index).unwrap()
            },
        };
        if let Some(cell) = chunk.get_mut(index.local_index()) {
            *cell = Some(t);
        }
    }

    pub fn cast_ray<F>(&self, visit:F, ray:&ERay) where F:Fn()->bool {
        
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raycast_test() {
    }

    #[test]
    fn infindex() {
        let p1:EIndex = [0, 0].into();
        let p2:EIndex = (0, 0).into();
        assert_eq!(p1, p2);
        let p1:EIndex = [5, 3].into();
        let p2:EIndex = (5, 3).into();
        assert_eq!(p1, p2);
        let p1:EIndex = [1, 2].into();
        let p2:EIndex = (2, 1).into();
        assert_ne!(p1, p2);
        let p1:EIndex = [i32::MIN, i32::MAX].into();
        let p2:EIndex = (i32::MIN, i32::MAX).into();
        assert_eq!(p1, p2);

        let p1:EIndex = [0, 0].into();
        let p2:EIndex = [15, 15].into();
        assert_ne!(p1, p2);
        assert_eq!(p1.chunk_index(), p2.chunk_index());

        let p1:EIndex = [-7, -7].into();
        let p2:EIndex = [-9, -9].into();
        assert_ne!(p1, p2);
        assert_eq!(p1.chunk_index(), p2.chunk_index());
    }

    #[test]
    fn test() {
        let mut grid = EGrid::default() as EGrid<(i32, i32)>;
        let size = 64;
        for y in -size..size {
            for x in -size..size {
                let p = (x,y);
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
