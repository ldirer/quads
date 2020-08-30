use std::env;
use image::{RgbaImage, ImageBuffer};
// I can use ::quadslib because I *named* my lib quadslib in cargo.toml.
use ::quadslib::{process_image, RGBAImage};


// Convert from our 'flat' RGBAImage to an `image` crate object and back.
// note this moves the values (avoids copying the raw data).
// I can't implement this as the 'From' trait unless I'm inside the 'lib' crate.
// But then it requires image, which is a dependency I don't want inside 'lib'.
fn rgba_to_lib_rgba_image(img: RgbaImage) -> RGBAImage{
        let width = img.width();
        let height = img.height();
        let data = img.into_raw();
        println!("width, height: {}, {}", width, height);
        RGBAImage::constructor(
            &data,
            width,
            height,
        )
}

fn lib_rgba_to_rgba_image(im: &RGBAImage) -> image::RgbaImage {
    RgbaImage::from(
        ImageBuffer::from_raw(im.width(), im.height(), im.data().clone())
            .expect("unable to convert image to ImageBuffer"),
    )
}

fn save_image(im: &RGBAImage, i: u32, total_error: f64) {
    println!("SAVING Iteration {}, error {}", i, total_error);
    lib_rgba_to_rgba_image(im).save(format!("output/step_{:06}.png", i)).unwrap();
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
            let rgba_im = rgba_to_lib_rgba_image(im.clone());
            process_image(rgba_im, n_iter, &save_image)
        }
    }

    println!("Done!");
}
