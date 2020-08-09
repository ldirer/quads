use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgba, RgbaImage};
use std::collections::VecDeque;
use std::env;

// Essentially a remake of: https://github.com/fogleman/Quads
// Input: a picture
// Split it into 4 quadrants.
// Assign average color of quadrant to pixels, compute quadrant errors
// Quadrant with largest error (among all previous quadrants) is chosen to reapply the process.
// Repeat N times.

// A 'quadtree leaf' is essentially a rectangle, we just need top-left bottom-right
// I didn't 'formally' keep the idea of quadtree in the code, we just manipulate rectangles.

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

impl Rectangle {
    // Split the rectangle and return the 4 child rectangles.
    fn split(&self) -> Vec<Rectangle> {
        let x_min = self.top_left.x;
        let y_min = self.top_left.y;

        let x_max = self.bottom_right.x;
        let y_max = self.bottom_right.y;

        let x_mid = (x_min + x_max) / 2;
        let y_mid = (y_min + y_max) / 2;

        vec![
            Rectangle {
                top_left: self.top_left.clone(),
                bottom_right: Point { x: x_mid, y: y_mid },
            },
            Rectangle {
                top_left: Point {
                    x: self.top_left.x,
                    y: y_mid,
                },
                bottom_right: Point { x: x_mid, y: y_max },
            },
            Rectangle {
                top_left: Point {
                    x: x_mid,
                    y: self.top_left.y,
                },
                bottom_right: Point { x: x_max, y: y_mid },
            },
            Rectangle {
                top_left: Point { x: x_mid, y: y_mid },
                bottom_right: self.bottom_right.clone(),
            },
        ]
    }

    fn width(&self) -> u32 {
        (self.bottom_right.x - self.top_left.x) as u32
    }

    fn height(&self) -> u32 {
        (self.bottom_right.y - self.top_left.y) as u32
    }
}

fn average_value(r: &Rectangle, im: &RgbaImage) -> [u8; 4] {
    let mut avg: [u64; 4] = [0, 0, 0, 0];
    for x in r.top_left.x..r.bottom_right.x {
        for y in r.top_left.y..r.bottom_right.y {
            let [r, g, b, a] = im.get_pixel(x as u32, y as u32).0;
            avg[0] += r as u64;
            avg[1] += g as u64;
            avg[2] += b as u64;
            avg[3] += a as u64;
        }
    }

    let pixel_count = (r.width() as u64) * (r.height() as u64);
    [
        (avg[0] / pixel_count) as u8,
        (avg[1] / pixel_count) as u8,
        (avg[2] / pixel_count) as u8,
        (avg[3] / pixel_count) as u8,
    ]
}

fn error_image(r: &Rectangle, im: &RgbaImage) -> f64 {
    let mut error = [0, 0, 0, 0];
    let avg_color = average_value(r, im);
    for x in r.top_left.x..r.bottom_right.x {
        for y in r.top_left.y..r.bottom_right.y {
            let rgba = im.get_pixel(x as u32, y as u32).0;
            let error_incr: Vec<i64> = rgba
                .iter()
                .zip(avg_color.iter())
                .map(|(a, b)| (*a as i64 - *b as i64).pow(2))
                .collect();
            // just adding arrays
            for i in 0..4 {
                error[i] += error_incr[i];
            }
        }
    }

    let pixel_count = ((r.width() as u64) * (r.height() as u64)) as i64;

    // we use luminance coefficients weighting for each channel error. Trying to reproduce fogleman's results
    error
        .iter()
        .map(|color_error| (*color_error as f64 / pixel_count as f64).sqrt())
        .zip(&[0.2989, 0.5870, 0.1140, 0.])
        .map(|(color_error, weight)| color_error as f64 * weight)
        .sum()
}

// Draw rectangles on the given image.
// This assumes that only 'leaves' rectangles are present in the iterator.
// To put it differently: we don't draw twice the same pixel.
// As to why IntoIterator: https://stackoverflow.com/a/39676011/3914041
fn draw_rectangles<'a, T: IntoIterator<Item = &'a Rectangle>>(
    output: &mut RgbaImage,
    im: &RgbaImage,
    rs: T,
) {
    for r in rs.into_iter() {
        let avg = average_value(r, im);
        for x in r.top_left.x..r.bottom_right.x {
            for y in r.top_left.y..r.bottom_right.y {
                let c = if x == r.top_left.x
                    || x == r.bottom_right.x
                    || y == r.top_left.y
                    || y == r.bottom_right.y
                {
                    Rgba([0, 0, 0, 255])
                } else {
                    Rgba(avg)
                };
                output.put_pixel(x as u32, y as u32, c)
            }
        }
    }
}

fn process_image(im: &RgbaImage, n_iter: u32) {
    const ERROR_RATE_SAVE_FRAME: f64 = 10.;
    // error, rect pairs
    let mut rectangles = vec![(
        0 as f64,
        Rectangle {
            top_left: Point { x: 0, y: 0 },
            bottom_right: Point {
                x: (im.width() - 1) as usize,
                y: (im.height() - 1) as usize,
            },
        },
    )];

    let mut im_result = im.clone();

    let mut previous_error = -1.;
    for i in 0..n_iter {
        // We don't want to pick rectangles we can't split, so we draw until we get a big enough one.
        let mut r: Rectangle = Rectangle {
            top_left: Point { x: 0, y: 0 },
            bottom_right: Point { x: 0, y: 0 },
        };
        while !(r.height() > 5 && r.width() > 5) {
            if rectangles.len() == 0 {
                println!("rectangles list is empty, ran out of candidates!");
                break;
            }
            let (error, rect) = rectangles.remove(rectangles.len() - 1);
            r = rect;
        }

        let mut splits = r.split();

        // incremental drawing of results.
        draw_rectangles(&mut im_result, im, &splits);

        // Split the rectangle and add to the list of candidates (at index determined by error)
        for s in splits.into_iter() {
            let s_area = (s.width() * s.height()) as f64;
            // The error is not *exactly* the same as in fogleman's code. maybe an 'off by one' difference
            let s_error = error_image(&s, im) * s_area.powf(0.25);
            // binary search Err is returned when the element is not found, contains the index where we should insert
            let idx = rectangles
                .binary_search_by(|pair| {
                    pair.0
                        .partial_cmp(&s_error)
                        .expect("couldn't compare f64 values (NaN?)")
                })
                .unwrap_or_else(|x| x);
            rectangles.insert(idx, (s_error, s));
        }

        let total_error: f64 = rectangles
            .iter()
            .map(|pair| pair.0 * pair.1.width() as f64 * pair.1.height() as f64)
            .sum::<f64>()
            / (im.width() as f64 * im.height() as f64);
        println!(
            "Iteration {}, error {}, previous_error {}",
            i, total_error, previous_error
        );
        if previous_error == -1. || previous_error - total_error > ERROR_RATE_SAVE_FRAME {
            println!("SAVING Iteration {}, error {}", i, total_error);
            im_result.save(format!("output/step_{:06}.png", i)).unwrap();
            previous_error = total_error;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("running program with args: {:?}", args);
    let target_fname = &args[1];
    let n_iter: u32 = args[2].parse::<u32>().unwrap();

    let maybe_img = image::open(target_fname);
    match maybe_img {
        Err(e) => println!("error opening file {:#?}", e),
        Ok(img) => {
            println!("success reading image!");
            let im = img.as_rgba8().expect("error: unable to read image as rgba");
            process_image(im, n_iter)
        }
    }

    println!("Done!");
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_rectangle_split() {
        let r = Rectangle {
            top_left: Point { x: 100, y: 200 },
            bottom_right: Point { x: 1000, y: 1000 },
        };

        let mut splits = r.split();

        splits.sort_by(|r1, r2| r1.top_left.cmp(&r2.top_left));
        // splits.sort_by(|r1, r2| r1.bottom_right.x.cmp(&r2.bottom_right.x));

        assert_eq!(
            splits,
            vec![
                Rectangle {
                    top_left: Point { x: 100, y: 200 },
                    bottom_right: Point { x: 550, y: 600 },
                },
                Rectangle {
                    top_left: Point { x: 100, y: 600 },
                    bottom_right: Point { x: 550, y: 1000 },
                },
                Rectangle {
                    top_left: Point { x: 550, y: 200 },
                    bottom_right: Point { x: 1000, y: 600 },
                },
                Rectangle {
                    top_left: Point { x: 550, y: 600 },
                    bottom_right: Point { x: 1000, y: 1000 },
                },
            ],
        )
    }

    #[test]
    fn test_rect_basics() {
        let r = Rectangle {
            top_left: Point { x: 10, y: 20 },
            bottom_right: Point { x: 100, y: 50 },
        };

        assert_eq!(r.width(), 90);
        assert_eq!(r.height(), 30);
    }
}
