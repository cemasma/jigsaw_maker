use image::{open as OpenImage, Pixel, RgbaImage};
use imageproc::drawing;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;

use clap::{arg, value_parser, Command};

const RED: image::Rgba<u8> = image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]);
const BLACK: image::Rgba<u8> = image::Rgba([0 as u8, 0 as u8, 0 as u8, 255 as u8]);

struct Point {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone)]
enum ConnectionType {
    IN,
    OUT,
}

impl Distribution<ConnectionType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ConnectionType {
        match rng.gen_range(0..=1) {
            0 => ConnectionType::IN,
            _ => ConnectionType::OUT,
        }
    }
}

fn main() {
    let matches = Command::new("Jigsaw Maker")
        .version("1.0")
        .about("Generates jigsaw pieces by given image input")
        .arg(arg!(-f --file <FILE>).required(true))
        .arg(
            arg!(--column <COLUMN>)
                .default_value("4")
                .value_parser(value_parser!(u32)),
        )
        .arg(
            arg!(--row <ROW>)
                .default_value("4")
                .value_parser(value_parser!(u32)),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").expect("required");
    let column_count = *matches.get_one::<u32>("column").expect("required");
    let row_count = *matches.get_one::<u32>("row").expect("required");

    let img = OpenImage(file_path)
        .expect("You should enter a valid image file")
        .into_rgb8();

    let (width, height) = img.dimensions();

    let row_size = height / row_count;
    let column_size = width / column_count;

    let curve_height: f32 = (column_size + row_size) as f32 * 0.09398;
    let curve_width = ((column_size + row_size) as f32 * 0.05639) as u32;

    let grid: HashMap<String, Vec<String>> = prepare_grid_hash_map(column_count, row_count);

    let (curves, curve_margin): (HashMap<String, Vec<Point>>, u32) =
        prepare_curve_points(column_size, row_size, curve_width, curve_height);

    for (key, paths) in grid.into_iter() {
        let coordinates: Vec<&str> = key.split(":").collect();

        let mut extra_column_size = 0;
        let mut extra_row_size = 0;
        let mut starting_x = 0;
        let mut starting_y = 0;
        for val in paths.iter() {
            if val == "RIGHT_OUT" || val == "LEFT_OUT" {
                extra_column_size += curve_margin;

                if val == "LEFT_OUT" {
                    starting_x = curve_margin;
                }
            } else if val == "TOP_OUT" || val == "BOTTOM_OUT" {
                extra_row_size += curve_margin;

                if val == "TOP_OUT" {
                    starting_y = curve_margin;
                }
            }
        }

        let mut imgbuf = RgbaImage::from_pixel(
            column_size + extra_column_size,
            row_size + extra_row_size,
            image::Rgba([0; 4]),
        );

        let mut val_map: HashMap<&str, bool> = HashMap::new();
        for val in paths.iter() {
            let curve_points = curves.get(val).unwrap();

            val_map.insert(val, true);

            let mut extra_x = 0;
            let mut extra_y = 0;

            if val == "TOP_OUT" {
                extra_x = starting_x;
            } else if val == "LEFT_OUT" {
                extra_y = starting_y;
            } else {
                extra_x = starting_x;
                extra_y = starting_y;
            }

            for point in curve_points.iter() {
                let pixel = imgbuf.get_pixel_mut(point.x + extra_x, point.y + extra_y);
                *pixel = BLACK;
            }
        }

        let ref_space_x_start = (column_size / 2) - curve_width as u32 + starting_x;
        let ref_space_x_end = (column_size / 2) + curve_width as u32 + starting_x;

        for x in starting_x..(column_size + starting_x) {
            if !((val_map.contains_key("TOP_IN") || val_map.contains_key("TOP_OUT"))
                && x >= ref_space_x_start
                && x <= ref_space_x_end)
            {
                let pixel = imgbuf.get_pixel_mut(x, starting_y);
                *pixel = BLACK;
            }

            if !((val_map.contains_key("BOTTOM_IN") || val_map.contains_key("BOTTOM_OUT"))
                && x >= ref_space_x_start
                && x <= ref_space_x_end)
            {
                let pixel = imgbuf.get_pixel_mut(x, row_size - 1 + starting_y);
                *pixel = BLACK;
            }
        }

        let ref_space_y_start = (row_size / 2) - curve_width as u32 + starting_y;
        let ref_space_y_end = (row_size / 2) + curve_width as u32 + starting_y;
        for y in starting_y..(row_size + starting_y) {
            if !((val_map.contains_key("LEFT_IN") || val_map.contains_key("LEFT_OUT"))
                && y >= ref_space_y_start
                && y <= ref_space_y_end)
            {
                let pixel = imgbuf.get_pixel_mut(starting_x, y);
                *pixel = BLACK;
            }

            if !((val_map.contains_key("RIGHT_IN") || val_map.contains_key("RIGHT_OUT"))
                && y >= ref_space_y_start
                && y <= ref_space_y_end)
            {
                let pixel = imgbuf.get_pixel_mut(column_size - 1 + starting_x, y);
                *pixel = BLACK;
            }
        }

        let mut x = starting_x + 1;
        let mut y = starting_y;

        while y < (row_size + starting_y) {
            if y >= ref_space_y_start && y <= ref_space_y_end {
                y = y + 1;
                continue;
            }

            if x >= ref_space_x_start && x <= ref_space_x_end {
                x = x + 1;
                continue;
            }

            let pixel = imgbuf.get_pixel_mut(x, y);
            let rgba = pixel.to_rgba();
            if rgba == BLACK {
                y = y + 1;
                x = starting_x + 1;

                continue;
            }

            *pixel = RED;

            x = x + 1;
        }

        for x in ref_space_x_start..ref_space_x_end + 1 {
            let mut y = ref_space_y_start;

            loop {
                let pixel = imgbuf.get_pixel_mut(x, y);
                let rgba = pixel.to_rgba();
                if rgba == BLACK {
                    break;
                }
                *pixel = RED;
                y = y + 1;
            }
        }

        for x in ref_space_x_start..ref_space_x_end + 1 {
            let mut y = ref_space_y_start;

            loop {
                let pixel = imgbuf.get_pixel_mut(x, y);
                let rgba = pixel.to_rgba();
                if rgba == BLACK {
                    break;
                }
                *pixel = RED;
                y = y - 1;
            }
        }

        for y in ref_space_y_start..ref_space_y_end + 1 {
            let mut x = ref_space_x_start;

            loop {
                let pixel = imgbuf.get_pixel_mut(x, y);
                let rgba = pixel.to_rgba();
                if rgba == BLACK {
                    break;
                }
                *pixel = RED;
                x = x + 1;
            }
        }

        for y in ref_space_y_start..ref_space_y_end + 1 {
            let mut x = ref_space_x_start;

            loop {
                let pixel = imgbuf.get_pixel_mut(x, y);
                let rgba = pixel.to_rgba();
                if rgba == BLACK {
                    break;
                }
                *pixel = RED;
                x = x - 1;
            }
        }

        let (gridx, gridy): (u32, u32) = (
            coordinates[0].parse().unwrap(),
            coordinates[1].parse().unwrap(),
        );

        for x in 0..(column_size + extra_column_size) {
            for y in 0..(row_size + extra_row_size) {
                let pixel = imgbuf.get_pixel_mut(x, y);
                let rgba = pixel.to_rgba();

                if rgba != RED {
                    continue;
                }

                let x_axis_from_src = (gridx * column_size) + x - starting_x;
                let y_axis_from_src = (gridy * row_size) + y - starting_y;

                let pixel_src = img.get_pixel(x_axis_from_src, y_axis_from_src);

                *pixel = image::Rgba([pixel_src[0], pixel_src[1], pixel_src[2], 255 as u8]);
            }
        }

        fs::create_dir_all("out").unwrap();
        imgbuf.save(format!("out/{}.png", key)).unwrap();
    }
}

fn prepare_grid_hash_map(column_count: u32, row_count: u32) -> HashMap<String, Vec<String>> {
    let mut roads: HashMap<String, bool> = HashMap::new();
    let mut grid: HashMap<String, Vec<String>> = HashMap::new();
    for x in 0..column_count {
        for y in 0..row_count {
            let neighbors = get_neighbors(x, y, column_count, row_count);

            for neighbor in neighbors {
                let (k1, k2) = hash_connection(x, y, neighbor.x, neighbor.y);

                let key1 = format!("{}:{}", x, y);
                let key2 = format!("{}:{}", neighbor.x, neighbor.y);
                if !roads.contains_key(k1.as_str()) && !roads.contains_key(k2.as_str()) {
                    let connection_type: ConnectionType = rand::random();

                    let mut decided_value_1 = "";
                    let mut decided_value_2 = "";
                    if x == neighbor.x {
                        (decided_value_1, decided_value_2) = match y.cmp(&neighbor.y) {
                            Ordering::Less => ("BOTTOM", "TOP"),
                            Ordering::Greater => ("TOP", "BOTTOM"),
                            Ordering::Equal => ("", ""),
                        }
                    } else if y == neighbor.y {
                        (decided_value_1, decided_value_2) = match x.cmp(&neighbor.x) {
                            Ordering::Less => ("RIGHT", "LEFT"),
                            Ordering::Greater => ("LEFT", "RIGHT"),
                            Ordering::Equal => ("", ""),
                        }
                    }

                    let (decided_value_1, decided_value_2) = match connection_type {
                        ConnectionType::IN => (
                            [decided_value_1, "_IN"].join(""),
                            [decided_value_2, "_OUT"].join(""),
                        ),
                        ConnectionType::OUT => (
                            [decided_value_1, "_OUT"].join(""),
                            [decided_value_2, "_IN"].join(""),
                        ),
                    };

                    roads.insert(k1, true);
                    grid.entry(key1.to_string())
                        .or_insert(Vec::new())
                        .push(decided_value_1.to_string());

                    grid.entry(key2.to_string())
                        .or_insert(Vec::new())
                        .push(decided_value_2.to_string());
                }
            }
        }
    }

    return grid;
}

fn prepare_curve_points(
    column_size: u32,
    row_size: u32,
    curve_width: u32,
    curve_height: f32,
) -> (HashMap<String, Vec<Point>>, u32) {
    let mut curve_points: HashMap<String, Vec<Point>> = HashMap::new();

    let mut imgbuf = RgbaImage::from_pixel(column_size, row_size, image::Rgba([0; 4]));

    let start = (0 as f32, ((row_size / 2) - curve_width) as f32);
    let end = (0 as f32, ((row_size / 2) + curve_width) as f32);
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0 + curve_height, start.1),
        (end.0 + curve_height, end.1),
        RED,
    );

    let mut curve_margin = start.0 as u32;
    loop {
        let pixel = imgbuf.get_pixel(curve_margin, row_size / 2);

        if pixel.to_rgba() == RED {
            break;
        }
        curve_margin = curve_margin + 1;
    }

    curve_points.insert(
        "LEFT_IN".to_string(),
        collect_points(imgbuf, 0, 0, column_size, row_size),
    );

    let mut imgbuf = RgbaImage::from_pixel(column_size, row_size, image::Rgba([0; 4]));
    let start = (
        (column_size - 1) as f32,
        ((row_size / 2) - curve_width) as f32,
    );
    let end = (
        (column_size - 1) as f32,
        ((row_size / 2) + curve_width) as f32,
    );
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0 - curve_height as f32, start.1),
        (end.0 - curve_height as f32, end.1),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "RIGHT_IN".to_string(),
        collect_points(imgbuf, 0, 0, column_size, row_size),
    );

    let mut imgbuf = RgbaImage::from_pixel(column_size, row_size, image::Rgba([0; 4]));
    let start = (((column_size / 2) - curve_width) as f32, 0 as f32);
    let end = (((column_size / 2) + curve_width) as f32, 0 as f32);
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0, start.1 + curve_height),
        (end.0, end.1 + curve_height),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "TOP_IN".to_string(),
        collect_points(imgbuf, 0, 0, column_size, row_size),
    );

    let mut imgbuf = RgbaImage::from_pixel(column_size, row_size, image::Rgba([0; 4]));
    let start = (
        ((column_size / 2) - curve_width) as f32,
        (row_size - 1) as f32,
    );
    let end = (
        ((column_size / 2) + curve_width) as f32,
        (row_size - 1) as f32,
    );
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0, start.1 - curve_height),
        (end.0, end.1 - curve_height),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "BOTTOM_IN".to_string(),
        collect_points(imgbuf, 0, 0, column_size, row_size),
    );

    let mut imgbuf = RgbaImage::from_pixel(
        column_size + curve_height as u32,
        row_size,
        image::Rgba([0; 4]),
    );
    let start = (curve_margin as f32, ((row_size / 2) - curve_width) as f32);
    let end = (curve_margin as f32, ((row_size / 2) + curve_width) as f32);
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0 - curve_height as f32, start.1),
        (start.0 - curve_height as f32, end.1),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "LEFT_OUT".to_string(),
        collect_points(imgbuf, 0, 0, column_size + curve_height as u32, row_size),
    );

    let mut imgbuf = RgbaImage::from_pixel(
        column_size + curve_height as u32,
        row_size,
        image::Rgba([0; 4]),
    );
    let start = (
        (column_size - 1) as f32,
        ((row_size / 2) - curve_width) as f32,
    );
    let end = (
        (column_size - 1) as f32,
        ((row_size / 2) + curve_width) as f32,
    );
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0 + curve_height as f32, start.1),
        (start.0 + curve_height as f32, end.1),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "RIGHT_OUT".to_string(),
        collect_points(imgbuf, 0, 0, column_size + curve_height as u32, row_size),
    );

    let mut imgbuf = RgbaImage::from_pixel(
        column_size,
        row_size + curve_height as u32,
        image::Rgba([0; 4]),
    );
    let start = (
        ((column_size / 2) - curve_width) as f32,
        curve_margin as f32,
    );
    let end = (
        ((column_size / 2) + curve_width) as f32,
        curve_margin as f32,
    );
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0, start.1 - curve_height as f32),
        (end.0, end.1 - curve_height as f32),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "TOP_OUT".to_string(),
        collect_points(imgbuf, 0, 0, column_size, row_size + curve_height as u32),
    );

    let mut imgbuf = RgbaImage::from_pixel(
        column_size,
        row_size + curve_height as u32,
        image::Rgba([0; 4]),
    );
    let start = (
        ((column_size / 2) - curve_width) as f32,
        (row_size - 1) as f32,
    );
    let end = (
        ((column_size / 2) + curve_width) as f32,
        (row_size - 1) as f32,
    );
    imgbuf = drawing::draw_cubic_bezier_curve(
        &imgbuf,
        start,
        end,
        (start.0, start.1 + curve_height as f32),
        (end.0, end.1 + curve_height as f32),
        image::Rgba([255 as u8, 0 as u8, 0 as u8, 255 as u8]),
    );

    curve_points.insert(
        "BOTTOM_OUT".to_string(),
        collect_points(imgbuf, 0, 0, column_size, row_size + curve_height as u32),
    );

    return (curve_points, curve_margin);
}

fn collect_points(source_imgbuff: RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32) -> Vec<Point> {
    let mut point_vec = Vec::new();
    for column_index in x1..x2 {
        for row_index in y1..y2 {
            let rgb = source_imgbuff.get_pixel(column_index, row_index).to_rgb();

            if rgb[0] == RED[0] {
                point_vec.push(Point {
                    x: column_index,
                    y: row_index,
                });
            }
        }
    }

    return point_vec;
}

fn get_neighbors(x: u32, y: u32, column_count: u32, row_count: u32) -> Vec<Point> {
    let mut neighbors: Vec<Point> = Vec::new();

    if x != 0 {
        neighbors.push(Point { x: x - 1, y });
    }

    if y != 0 {
        neighbors.push(Point { x, y: y - 1 });
    }

    if x + 1 < column_count {
        neighbors.push(Point { x: x + 1, y });
    }

    if y + 1 < row_count {
        neighbors.push(Point { x, y: y + 1 });
    }

    return neighbors;
}

fn hash_connection(x: u32, y: u32, z: u32, t: u32) -> (String, String) {
    return (
        format!("{}:{}|{}:{}", x, y, z, t),
        format!("{}:{}|{}:{}", z, t, x, y),
    );
}
