extern crate image;
extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::thread;
use std::env;
use std::sync::Arc;
use std::ops::Range;

use image::{ImageBuffer, Luma};
use object_space::{ObjectSpace, ObjectSpaceKey, ObjectSpaceRange, TreeObjectSpace};

fn main() {
    let mut args = env::args();

    let thread_count = args.nth(1)
        .and_then(|input| input.parse::<i32>().ok())
        .unwrap_or(4);

    let dim = args.nth(2)
        .and_then(|input| input.parse::<u32>().ok())
        .unwrap_or(512);

    let max = args.nth(3)
        .and_then(|input| input.parse::<i32>().ok())
        .unwrap_or(1000);
    // mandelbrot(dim, max);
    run(dim, max, thread_count)
}

fn run(dim: u32, iter_count: i32, thread_count: i32) {
    let space = Arc::new(TreeObjectSpace::new());

    // create 4 worker threads
    for _ in 0..thread_count {
        let space_clone = space.clone();
        thread::spawn(move || {
            dummy(space_clone, dim, iter_count);
        });
    }

    let chunk_size = 128;
    let mut task_count = dim / chunk_size;
    let mut markers: Vec<_> = (0..task_count).map(|i| chunk_size * i).collect();
    if dim % chunk_size != 0 {
        task_count += 1;
        markers.push(dim);
    }

    for i in 0..(task_count - 1) as usize {
        for j in 0..(task_count - 1) as usize {
            let clone = space.clone();
            clone.write(Task {
                finished: false,
                row_range: markers[i]..markers[i + 1],
                col_range: markers[j]..markers[j + 1],
            });
        }
    }

    let mut buffer = ImageBuffer::new(dim, dim);

    for _ in 0..((task_count - 1) * (task_count - 1)) {
        let clone = space.clone();
        clone.take_key::<Task>("finished", &true);
    }
    space.take_all::<Pixel>().for_each(|pixel| {
        let brightness = if pixel.iter_count < iter_count {
            255
        } else {
            0
        };
        let color = Luma { data: [brightness] };
        buffer.put_pixel(pixel.col, pixel.row, color)
    });

    buffer.save("mandelbrot.png").unwrap();
}

fn dummy(space: Arc<TreeObjectSpace>, dim: u32, max: i32) {
    loop {
        let task = space.take_key::<Task>("finished", &false);
        let row_range = task.row_range;
        let col_range = task.col_range;

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
                space.write(Pixel {
                    col,
                    row,
                    iter_count,
                })
            }
        }

        space.write(Task {
            finished: true,
            row_range: row_range,
            col_range: col_range,
        });
    }
}

fn mandelbrot(dim: u32, max: u32) {
    let mut buffer = ImageBuffer::new(dim, dim);
    let row_range = 0..dim;
    let col_range = 0..dim;
    for row in row_range {
        for col in col_range.clone() {
            let c_re = ((col as f64) - (dim as f64) / 2.0) * 4.0 / (dim as f64);
            let c_im = ((row as f64) - (dim as f64) / 2.0) * 4.0 / (dim as f64);
            let mut x = 0.0;
            let mut y = 0.0;
            let mut i = 0;
            while x * x + y * y < 4.0 && i < max {
                let x_new = x * x - y * y + c_re;
                y = 2.0 * x * y + c_im;
                x = x_new;
                i += 1;
            }

            if i >= max {
                buffer.put_pixel(col, row, Luma { data: [0] });
            } else {
                buffer.put_pixel(col, row, Luma { data: [255] });
            }
        }
    }
    buffer.save("mandelbrot.png").unwrap();
}

#[derive(Serialize, Deserialize)]
struct Task {
    finished: bool,
    row_range: Range<u32>,
    col_range: Range<u32>,
}

#[derive(Serialize, Deserialize)]
struct Pixel {
    col: u32,
    row: u32,
    iter_count: i32,
}
