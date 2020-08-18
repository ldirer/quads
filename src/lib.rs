use wasm_bindgen::prelude::*;
use web_sys::console;


// Essentially a remake of: https://github.com/fogleman/Quads
// Input: a picture
// Split it into 4 quadrants.
// Assign average color of quadrant to pixels, compute quadrant errors
// Quadrant with largest error (among all previous quadrants) is chosen to reapply the process.
// Repeat N times.

// A 'quadtree leaf' is a rectangle, we just need top-left bottom-right
// I didn't formally keep the idea of quadtree in the code, we just manipulate rectangles.


#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RGBAImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl RGBAImage {
    fn get_pix_offset(&self, i: usize, j: usize) -> usize {
        (j * self.width as usize + i) * 4
    }

    fn get_pixel(&self, i: usize, j: usize) -> RGBATuple {
        let offset = self.get_pix_offset(i, j);
        // we need to copy values into a fixed-size array.
        let mut pixel: [u8; 4] = [0, 0, 0, 0];
        pixel.copy_from_slice(&self.data[offset..(offset + 4)]);
        pixel
    }

    fn put_pixel(&mut self, i: usize, j: usize, c: RGBATuple) {
        let offset = self.get_pix_offset(i, j);
        for (idx, channel_value) in c.iter().enumerate() {
            self.data[offset + idx] = channel_value.clone();
        }
    }

    // used by server side to convert to/from image crate formats.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

#[wasm_bindgen]
impl RGBAImage {
    pub fn constructor(data: &[u8], width: u32, height: u32) -> RGBAImage {
        assert_eq!(data.len() as u32, width * height * 4);
        RGBAImage {
            data: data.to_vec(),
            width,
            height,
        }
    }

    pub fn data_as_pointer(&self) -> *const u8 {
        // we expose a pointer so that javascript can directly access webassembly memory.
        self.data.as_ptr()
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }
    pub fn height(&self) -> u32 {
        self.height as u32
    }
}

type RGBATuple = [u8; 4];

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
    // If the rectangle can't be split further, return an empty list.
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
        ].into_iter()
            // We filter to keep only non-empty rectangles (some might have no width or height).
            .filter(|r| r.top_left.x != r.bottom_right.x && r.top_left.y != r.bottom_right.y)
            // splits have to be smaller than original. (If the rectangle is 1x1 we can't split it, we want an empty list)
            .filter(|r| r.width() != self.width() || r.height() != self.height()).collect()
    }

    fn width(&self) -> u32 {
        (self.bottom_right.x - self.top_left.x) as u32
    }

    fn height(&self) -> u32 {
        (self.bottom_right.y - self.top_left.y) as u32
    }
}

fn average_value(r: &Rectangle, im: &RGBAImage) -> [u8; 4] {
    let mut avg: [u64; 4] = [0, 0, 0, 0];
    for x in r.top_left.x..r.bottom_right.x {
        for y in r.top_left.y..r.bottom_right.y {
            let [r, g, b, a] = im.get_pixel(x, y);
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

fn error_image(r: &Rectangle, im: &RGBAImage) -> f64 {
    let mut error = [0, 0, 0, 0];
    let avg_color = average_value(r, im);
    for x in r.top_left.x..r.bottom_right.x {
        for y in r.top_left.y..r.bottom_right.y {
            let rgba = im.get_pixel(x, y);
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
    output: &mut RGBAImage,
    im: &RGBAImage,
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
                    avg
                    // [30, 30, 30, 255]
                } else {
                    avg
                };
                output.put_pixel(x, y, c)
            }
        }
    }
}


#[wasm_bindgen]
pub struct ImageApproximation {
    im: RGBAImage,
    im_result: RGBAImage,
    pub max_iter: u32,

    rectangles: Vec<(f64, Rectangle)>,
    pub current_iter: u32,
    pub previous_error: f64,
}

#[wasm_bindgen]
impl ImageApproximation {
    pub fn constructor(im: RGBAImage, max_iter: u32) -> Self {
        let im_result = im.clone();
        // error, rect pairs
        let rectangles = vec![(
            0 as f64,
            Rectangle {
            top_left: Point { x: 0, y: 0 },
            bottom_right: Point {
                x: (im.width() - 1) as usize,
                y: (im.height() - 1) as usize,
            },
        },
        )];

        Self {
            im,
            im_result,
            max_iter,
            rectangles,
            current_iter: 0,
            previous_error: -1.,
        }
    }
}


// remove elements until one meets the `predicate` condition, then return it.
fn remove_until<T> (values: &mut Vec<T>, predicate: fn(&T) -> bool) -> Option<T> {
    if values.len() == 0 {
        return None
    }
    let mut candidate = values.remove(values.len() - 1);

    while !predicate(&candidate) {
        if values.len() == 0 {
            return None
        }
        candidate = values.remove(values.len() - 1);
    }
    Some(candidate)
}

#[wasm_bindgen]
impl ImageApproximation {

    pub fn im_result_data_as_pointer(&self) -> *const u8 {
        self.im_result.data_as_pointer()
    }

    // I tried to return values but the rust-wasm connection wasn't too happy about it. I yield!
    // return value indicates whether we are done or not
    pub fn next(&mut self) -> bool {
        // console::log_1(&"next from rust".into());
        const ERROR_RATE_SAVE_FRAME: f64 = 10.000001;
        // I had a recursive call instead of a loop but had 'max call stack sized exceeded' in wasm
        loop {
            if self.current_iter >= self.max_iter {
                return true
            }
            self.current_iter += 1;

            // we could filter so that we don't filter rectangles smaller than X pixels in width/height.
            let maybe_rect = remove_until(&mut self.rectangles, |(_e, _r)| true); // r.width() > 5 && r.height() > 5);
            let r: Rectangle;
            match maybe_rect {
                None => {
                    println!("rectangles list is empty, ran out of candidates!");
                    // console::log_1(&"rectangles list is empty, ran out of candidates!".into());
                    return true
                },
                Some((_e, rect)) => r = rect,
            }

            let splits = r.split();

            // incremental drawing of results.
            draw_rectangles(&mut self.im_result, &self.im, &splits);

            // Split the rectangle and add to the list of candidates (at index determined by error)
            for s in splits.into_iter() {
                let s_area = (s.width() * s.height()) as f64;
                // The error is not *exactly* the same as in fogleman's code. maybe an 'off by one' difference
                let s_error = error_image(&s, &self.im) * s_area.powf(0.25);
                // binary search Err is returned when the element is not found, contains the index where we should insert
                let idx = self.rectangles
                    .binary_search_by(|pair| {
                        pair.0
                            .partial_cmp(&s_error)
                            .expect("couldn't compare f64 values (NaN?)")
                    })
                    .unwrap_or_else(|x| x);
                self.rectangles.insert(idx, (s_error, s));
            }

            let total_error: f64 = self.rectangles
                .iter()
                .map(|pair| pair.0 * pair.1.width() as f64 * pair.1.height() as f64)
                .sum::<f64>()
                / (self.im.width() as f64 * self.im.height() as f64);
            println!(
                "Iteration {}, error {}, previous_error {}",
                self.current_iter, total_error, self.previous_error
            );
            if self.previous_error == -1. || self.previous_error - total_error > ERROR_RATE_SAVE_FRAME {
                self.previous_error = total_error;
                return false
            } else if self.previous_error == total_error || total_error == 0. {
                println!("Not making progress on error, returning early at iteration {}", self.current_iter);
                return true
            }
        }
    }
}

// This is a function for a server-side program only.
// Essentially I had this function signature but couldn't make it work with wasm-bindgen.
pub fn process_image(im: RGBAImage, n_iter: u32, callback: &dyn Fn(&RGBAImage, u32, f64)) {
    let mut approx = ImageApproximation::constructor(im, n_iter);
    while !approx.next() {
        callback(&approx.im_result, approx.current_iter, approx.previous_error);
    }
    // we want to apply the callback on the final state
    callback(&approx.im_result, approx.current_iter, approx.previous_error);
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_rectangle_standard_split() {
        let r = Rectangle {
            top_left: Point { x: 100, y: 200 },
            bottom_right: Point { x: 1000, y: 1000 },
        };

        let mut splits = r.split();

        splits.sort_by(|r1, r2| r1.top_left.cmp(&r2.top_left));

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
        );
    }

    #[test]
    fn test_empty_rectangle_split() {
        let r = Rectangle {
            top_left: Point { x: 100, y: 200 },
            bottom_right: Point { x: 101, y: 201 },
        };

        let splits = r.split();
        assert_eq!(splits, vec![]);
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


    #[test]
    fn test_remove_until() {
        let mut values = vec![1, 2, 12, 4, 1, 2, -2];
        let item = remove_until(&mut values, |&v| v > 4);
        assert_eq!(item, Some(12));
        assert_eq!(values, vec![1, 2]);

        let item = remove_until(&mut values, |&v| v > 4);
        assert_eq!(item, None);


        let mut empty: Vec<i32> = vec![];
        let item = remove_until(&mut empty, |&_v| true);
        assert_eq!(item, None);
    }
}
