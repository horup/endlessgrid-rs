use endlessgrid::Grid;

pub fn main() {
    let mut grid = Grid::default() as Grid<(i32, i32)>;
    let size = 64;
    for y in 0..size {
        for x in 0..size {
            let p = (x, y);
            grid.insert(p, p);
        }
    }

    grid.cast_ray((31.5, 51.5), (32.5, 52.5), |x|{
        dbg!(x.cell);
        true
    });
}