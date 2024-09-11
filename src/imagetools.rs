use ab_glyph::{Font, FontRef, Glyph, GlyphId};
use chrono::{Local, Timelike};
use std::f32::consts::PI;
use unicode_segmentation::UnicodeSegmentation;

use image::{DynamicImage, ImageReader, Rgba};
use imageproc::{
    drawing::{draw_filled_circle_mut, draw_polygon_mut, draw_text_mut},
    point::Point,
};

use crate::{input::Input, settings::Settings};

static FONT_DATA: &[u8] = include_bytes!("../JetbrainsMonoBold.ttf");

pub fn convert_image_from_path(path: &str) -> DynamicImage {
    let img = ImageReader::open(path).unwrap().decode().unwrap();

    //scale and flip
    let scale = f32::max(320.0 / img.width() as f32, 320.0 / img.height() as f32);
    let img2 = img.resize(
        f32::ceil(img.width() as f32 * scale) as u32,
        f32::ceil(img.height() as f32 * scale) as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let img3 = img2.crop_imm(0, 0, 320, 320);

    img3.flipv()
}

//pub fn convert_gif_from_path(path: &str) -> Vec<u8> {
//    // Open the file
//    let mut decoder = gif::DecodeOptions::new();
//    // Configure the decoder such that it will expand the image to RGBA.
//    decoder.set_color_output(gif::ColorOutput::RGBA);
//    // Read the file header
//    let file = File::open(path).unwrap();
//    let mut decoder = decoder.read_info(file).unwrap();
//    while let Some(frame) = decoder.read_next_frame().unwrap() {
//        // Process every frame
//    }
//    decoder.
//}

pub fn image_from_input(input: Input, settings: &Settings) -> Vec<u8> {
    let mut image = DynamicImage::new_rgba8(320, 320);

    if input.values.len() == 1 {
        draw_bars(
            &mut image,
            input.values[0],
            input.values[0],
            settings.left_bar,
            settings.right_bar,
        );
        draw_value(
            &mut image,
            &[&input.get_string_at(0)],
            settings.left_value,
            settings.right_value,
        );
        draw_title(
            &mut image,
            &[&input.get_title_at(0)],
            settings.left_title,
            settings.right_title,
        );
    } else if input.values.len() >= 2 {
        draw_bars(
            &mut image,
            input.values[0],
            input.values[1],
            settings.left_bar,
            settings.right_bar,
        );
        draw_value(
            &mut image,
            &[&input.get_string_at(0), &input.get_string_at(1)],
            settings.left_value,
            settings.right_value,
        );
        draw_title(
            &mut image,
            &[&input.get_title_at(0), &input.get_title_at(1)],
            settings.left_title,
            settings.right_title,
        );
    }
    if input.time || settings.show_time {
        draw_time(&mut image, settings.time);
    }

    //test:
    //draw_title(&mut image,&[&input.overlay],settings.left_title,settings.left_title);

    let image = image.fliph();
    let image = image.flipv();
    //image.save("/tmp/test.png").unwrap();
    image.into_bytes()
}

fn draw_bars(
    image: &mut DynamicImage,
    left_val: f32,
    right_val: f32,
    left_col: Rgba<u8>,
    right_col: Rgba<u8>,
) {
    let width = 33.0;
    let black = Rgba([0, 0, 0, 255]);
    let grey = Rgba([30, 30, 30, 255]);
    //normal range: 0-80
    let cr = (320.0 - width) / 2.0; //between outer 320 and inner 240
    let tr = 1520.0 / 2.0;
    //left
    let left_ratio = ((left_val.min(100.) as f32 - 0.0) / 120.0)
        .max(0.0)
        .min(1.0);
    let left_theta = left_ratio * PI / 2.0;
    let lcw = cr * left_theta.cos();
    let lch = cr * left_theta.sin();
    let ltw = tr * left_theta.cos();
    let lth = tr * left_theta.sin();

    //right
    let right_ratio = ((right_val.min(100.) as f32 - 0.0) / 120.0)
        .max(0.0)
        .min(1.0);
    let right_theta = right_ratio * PI / 2.0;
    let rcw = cr * right_theta.cos();
    let rch = cr * right_theta.sin();
    let rtw = tr * right_theta.cos();
    let rth = tr * right_theta.sin();

    //outer loop
    draw_filled_circle_mut(image, (160, 160), 160, grey);

    draw_polygon_mut(
        image,
        &[
            Point { x: 160, y: 160 },
            Point {
                x: 160 - ltw as i32,
                y: 160 - lth as i32,
            },
            Point {
                x: 160 - ltw as i32,
                y: 160 + lth as i32,
            },
        ],
        left_col,
    );

    draw_polygon_mut(
        image,
        &[
            Point { x: 160, y: 160 },
            Point {
                x: 160 + rtw as i32,
                y: 160 - rth as i32,
            },
            Point {
                x: 160 + rtw as i32,
                y: 160 + rth as i32,
            },
        ],
        right_col,
    );

    draw_filled_circle_mut(image, (160, 160), 160 - width as i32, black);

    //ends
    draw_filled_circle_mut(
        image,
        ((160.0 - lcw) as i32, (160.0 - lch) as i32),
        (width / 2.0) as i32,
        left_col,
    );
    draw_filled_circle_mut(
        image,
        ((160.0 - lcw) as i32, (160.0 + lch) as i32),
        (width / 2.0) as i32,
        left_col,
    );
    draw_filled_circle_mut(
        image,
        ((160.0 + rcw) as i32, (160.0 - rch) as i32),
        (width / 2.0) as i32,
        right_col,
    );
    draw_filled_circle_mut(
        image,
        ((160.0 + rcw) as i32, (160.0 + rch) as i32),
        (width / 2.0) as i32,
        right_col,
    );
}

fn draw_time(image: &mut DynamicImage, col: Rgba<u8>) {
    let font = FontRef::try_from_slice(FONT_DATA).expect("Error constructing Font");

    let scale = 50.0;
    let ch = Local::now();
    let val = format!("{}:{:0>2}", ch.time().hour(), ch.time().minute());
    let x: i32 = 160 - get_width(&val, &font, scale) as i32 / 2;
    draw_text_mut(image, col, x, 45, scale, &font, &val);
}
fn draw_value(image: &mut DynamicImage, vals: &[&str], left_col: Rgba<u8>, right_col: Rgba<u8>) {
    let font = FontRef::try_from_slice(FONT_DATA).expect("Error constructing Font");

    if vals.len() == 1 {
        let scale = 80.0;
        let val = truncate(vals[0], 6); //max 6
        let width = get_width(&val, &font, scale);
        let x: i32 = 160 - width / 2;

        draw_text_mut(image, left_col, x, 110, scale, &font, &val);
    } else if vals.len() >= 2 {
        let scale = 65.0;
        let val0 = truncate(vals[0], 4); //max 6
        let val1 = truncate(vals[1], 4);
        let width0 = get_width(&val0, &font, scale);
        let width1 = get_width(&val1, &font, scale);
        let x0 = 105 - width0 / 2;
        let x1 = 215 - width1 / 2;
        draw_text_mut(image, left_col, x0, 120, scale, &font, &val0);
        draw_text_mut(image, right_col, x1, 120, scale, &font, &val1);
    }
}

fn draw_title(image: &mut DynamicImage, vals: &[&str], left_col: Rgba<u8>, right_col: Rgba<u8>) {
    let font = FontRef::try_from_slice(FONT_DATA).expect("Error constructing Font");

    if vals.len() == 1 {
        let scale = 40.0;
        let val = truncate(vals[0], 6); //max 6
        let x = 160 - get_width(&val, &font, scale) / 2;
        draw_text_mut(image, left_col, x, 190, scale, &font, &val);
    } else if vals.len() >= 2 {
        let scale = 40.0;
        let val0 = truncate(vals[0], 4); //max 6
        let val1 = truncate(vals[1], 4);
        let width0 = get_width(&val0, &font, scale);
        let width1 = get_width(&val1, &font, scale);
        let x0 = 105 - width0 / 2;
        let x1 = 215 - width1 / 2;
        draw_text_mut(image, left_col, x0, 190, scale, &font, &val0);
        draw_text_mut(image, right_col, x1, 190, scale, &font, &val1);
    }
}

fn truncate(string: &str, len: usize) -> String {
    let gr = string.graphemes(true).collect::<Vec<&str>>();
    if gr.len() > len {
        return gr[..len].join("");
    }
    string.to_owned()
}

fn get_width(string: &str, font: &impl Font, scale: f32) -> i32 {
    //calc size of 1 char
    let g = GlyphId('A' as u16);
    let a = font.glyph_bounds(&Glyph {
        id: g,
        scale: scale.into(),
        position: (0.0, 0.0).into(),
    });

    a.width() as i32 * string.graphemes(true).count() as i32
}
