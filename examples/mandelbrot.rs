extern crate image;
extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::ops::Range;
use std::sync::Arc;
use std::thread;
use std::vec::Vec;

use image::{ImageBuffer, Luma};
use object_space::{ObjectSpace, TreeObjectSpace};

fn main() {
    let mut args = env::args();
    args.next();

    let thread_count = args.next()
        .and_then(|input| input.parse::<i32>().ok())
        .unwrap_or(4);

    let dim = args.next()
        .and_then(|input| input.parse::<u32>().ok())
        .unwrap_or(1024);

    let max = args.next()
        .and_then(|input| input.parse::<i32>().ok())
        .unwrap_or(1000);

    run(dim, max, thread_count)
}

fn run(dim: u32, iter_count: i32, thread_count: i32) {
    let space = Arc::new(TreeObjectSpace::new());

    // create worker threads
    for _ in 0..thread_count {
        let space_clone = space.clone();
        thread::spawn(move || {
            mandelbrot(space_clone, dim, iter_count);
        });
    }

    let task_count = thread_count as u32;
    let chunk_size = dim / task_count;
    let mut markers: Vec<_> = (0..task_count).map(|i| chunk_size * i).collect();
    markers.push(dim);

    for i in 0..task_count as usize {
        for j in 0..task_count as usize {
            let clone = space.clone();
            clone.write(Task {
                row_range: markers[i]..markers[i + 1],
                col_range: markers[j]..markers[j + 1],
            });
        }
    }

    let mut buffer = ImageBuffer::new(dim, dim);
    for _ in 0..(task_count * task_count) {
        let clone = space.clone();
        let vec = clone.take::<Vec<Pixel>>();
        for pixel in &vec {
            let color = if pixel.iter_count == iter_count {
                0
            } else {
                255
            };
            buffer.put_pixel(pixel.col, pixel.row, Luma { data: [color] });
        }
    }

    buffer.save("mandelbrot.png").unwrap();
}

fn mandelbrot(space: Arc<TreeObjectSpace>, dim: u32, max: i32) {
    loop {
        let task = space.take::<Task>();
        let row_range = task.row_range;
        let col_range = task.col_range;
        let mut result = Vec::new();

        for row in row_range.clone() {
            for col in col_range.clone() {
                let c_re = ((col as f64) - (dim as f64) / 2.0) * 4.0 / (dim as f64);
                let c_im = ((row as f64) - (dim as f64) / 2.0) * 4.0 / (dim as f64);
                let mut x = 0.0;
                let mut y = 0.0;
                let mut iter_count = 0;
                while x * x + y * y < 4.0 && iter_count < max {
                    let x_new = x * x - y * y + c_re;
                    y = 2.0 * x * y + c_im;
                    x = x_new;
                    iter_count += 1;
                }
                result.push(Pixel {
                    col,
                    row,
                    iter_count,
                });
            }
        }

        space.write(result);
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    row_range: Range<u32>,
    col_range: Range<u32>,
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
struct Pixel {
    col: u32,
    row: u32,
    iter_count: i32,
}
