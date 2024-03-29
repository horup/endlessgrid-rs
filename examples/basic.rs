use std::time::Instant;

use egrid::*;

fn main() {
    let mut grid = EGrid::default() as EGrid<i32>;
    let instant = Instant::now();
    let size = 1024;
    for y in -size..size {
        for x in -size..size {
            grid.insert([x, y], 0);
        }
    }
    println!("Took {}ms to intialize", (Instant::now() - instant).as_millis());

    let instant = Instant::now();
    let size = 64;
    for y in -size..size {
        for x in  -size..size {
            *grid.get_mut([x,y]).unwrap() = 1;
        }
    }
    println!("Took {}micro to set", (Instant::now() - instant).as_micros());
}

//http://www.adammil.net/blog/v125_roguelike_vision_algorithms.htm#raycast