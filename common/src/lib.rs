//! Общие ресурсы для лабораторных работ
//! Группа 0323

// TODO: Do input-checking (empty arrays, etc).

use winit::dpi::PhysicalSize;

pub type Point2D = (i32, i32);
pub type ColorRGB = (u8, u8, u8);

pub const BLACK: ColorRGB = (0x00, 0x00, 0x00);
pub const GRAY: ColorRGB = (0xde, 0xdd, 0xda);
pub const WHITE: ColorRGB = (0xff, 0xff, 0xff);
pub const RED: ColorRGB = (0xe0, 0x1b, 0x24);
pub const GREEN: ColorRGB = (0x2e, 0xc2, 0x7e);
pub const BLUE: ColorRGB = (0x35, 0x84, 0xe4);

// /// Moves data from raqote::DrawTarget into softbuffer::Surface
// pub fn draw_to_buffer(dt: &DrawTarget, buffer: &mut [u32], width: usize, height: usize) {
//     // Pixel data of DrawTarget is presented as:
//     // (A << 24) | (R << 16) | (G << 8) | B
//     let data = dt.get_data();

//     for y in 0..height {
//         for x in 0..width {
//             let pos: usize = y * width + x;

//             let value: u32 = {
//                 let red: u8 = (data[pos] >> 16) as u8;
//                 let green: u8 = (data[pos] >> 8) as u8;
//                 let blue: u8 = (data[pos] >> 0) as u8;
//                 ((blue as u32) | ((green as u32) << 8) | ((red as u32) << 16)) as u32
//             };

//             buffer[pos] = value;
//         }
//     }
// }

/// Clears screen with a color
pub fn draw_clear(buf: &mut [u32], dim: &PhysicalSize<u32>, col: &ColorRGB) {
    for y in 0..dim.height {
        for x in 0..dim.width {
            let index = y as usize * dim.width as usize + x as usize;
            buf[index] =
                (col.2 as u32) | ((col.1 as u32) << 8) | ((col.0 as u32) << 16);
        }
    }
}

/// Draws a pixel of a specified color into the buffer
pub fn draw_pixel(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    pos: &Point2D,
    col: &ColorRGB,
) {
    if pos.0 >= dim.width as i32
        || pos.1 >= dim.height as i32
        || pos.0 < 0
        || pos.1 < 0
    {
        return;
    }

    // Color is stored as (B | G << 8 | R << 16)
    buf[(pos.1 * dim.width as i32 + pos.0) as usize] =
        ((col.2 as u32) | ((col.1 as u32) << 8) | ((col.0 as u32) << 16))
            as u32;
}

/// Bresenham's line algorithm with variable tilt
/// * `slope` - tilt, value of `true` means it tilts downwards, else upwards
pub fn draw_line_slope(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    p0: &Point2D,
    p1: &Point2D,
    col: &ColorRGB,
    slope: bool,
) {
    let mut dx: i32 = p1.0 - p0.0;
    let mut dy: i32 = p1.1 - p0.1;

    if slope {
        // Slope down
        let mut yi: i32 = 1;

        if dy < 0 {
            yi = -1;
            dy = -dy;
        }

        let mut d: i32 = 2 * dy - dx;
        let mut y: i32 = p0.1;

        for x in p0.0..p1.0 {
            draw_pixel(buf, dim, &(x, y), col);
            if d > 0 {
                y += yi;
                d += 2 * (dy - dx);
            } else {
                d += 2 * dy;
            }
        }
    } else {
        // Slope up
        let mut xi: i32 = 1;

        if dx < 0 {
            xi = -1;
            dx = -dx;
        }

        let mut d: i32 = 2 * dx - dy;
        let mut x: i32 = p0.0;

        for y in p0.1..p1.1 {
            draw_pixel(buf, dim, &(x, y), col);
            if d > 0 {
                x += xi;
                d += 2 * (dx - dy);
            } else {
                d += 2 * dx;
            }
        }
    }
}

/// Bresenham's line algorithm with variable tilt and direction
pub fn draw_line(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    p0: &Point2D,
    p1: &Point2D,
    col: &ColorRGB,
) {
    // Compensate for any direction
    if (p1.1 - p0.1).abs() < (p1.0 - p0.0).abs() {
        if p0.0 > p1.0 {
            draw_line_slope(buf, dim, p1, p0, col, true);
        } else {
            draw_line_slope(buf, dim, p0, p1, col, true);
        }
    } else {
        if p0.1 > p1.1 {
            draw_line_slope(buf, dim, p1, p0, col, false);
        } else {
            draw_line_slope(buf, dim, p0, p1, col, false);
        }
    }
}

/// Draw coordinate axes from given point
pub fn draw_origin(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    org: &Point2D,
    col: &ColorRGB,
) {
    draw_line(buf, dim, &(org.0, 0), &(org.0, dim.height as i32 - 1), col);
    draw_line(buf, dim, &(0, org.1), &(dim.width as i32 - 1, org.1), col);
}

/// Bresenham's algorithm for circle generation
pub fn draw_circle(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    org: &Point2D,
    rad: i32,
    col: &ColorRGB,
) {
    if rad < 2 {
        draw_pixel(buf, dim, org, col);
        return;
    }

    let mut x: i32 = 0;
    let mut y: i32 = rad;
    let mut d: i32 = 3 - 2 * rad;

    while x <= y {
        draw_pixel(buf, dim, &(org.0 + x, org.1 + y), col);
        draw_pixel(buf, dim, &(org.0 - x, org.1 + y), col);
        draw_pixel(buf, dim, &(org.0 + x, org.1 - y), col);
        draw_pixel(buf, dim, &(org.0 - x, org.1 - y), col);
        draw_pixel(buf, dim, &(org.0 + y, org.1 + x), col);
        draw_pixel(buf, dim, &(org.0 - y, org.1 + x), col);
        draw_pixel(buf, dim, &(org.0 + y, org.1 - x), col);
        draw_pixel(buf, dim, &(org.0 - y, org.1 - x), col);

        x += 1;

        if d > 0 {
            y -= 1;
            d += 4 * (x - y) + 10;
        } else {
            d += 4 * x + 6;
        }
    }
}

/// Draw a line between all given points and connect the ends
pub fn draw_polygon(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    poly: &[Point2D],
    col: &ColorRGB,
) {
    let len: usize = poly.len();

    if len < 2 {
        return;
    }

    for i in 0..len - 1 {
        draw_line(buf, dim, &poly[i], &poly[i + 1], col);
    }
    draw_line(buf, dim, &poly[len - 1], &poly[0], col);
}

/// Draw a line between all given points
pub fn draw_connect(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    pts: &[Point2D],
    col: &ColorRGB,
) {
    let len: usize = pts.len();

    if len < 2 {
        return;
    }

    for i in 0..len - 1 {
        draw_line(buf, dim, &pts[i], &pts[i + 1], col);
    }
}

/// Draws circles around points
pub fn draw_points(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    pts: &Vec<Point2D>,
    col: &ColorRGB,
) {
    for p in pts {
        draw_circle(buf, dim, p, 4, col);
    }
}

/// Applies De Casteljau's algorithm in order to return a point
/// on the Bézier curve at a given position
/// * `pts` - points describing the curve
/// * `pos` - position on the curve, must be between 0 and 1
pub fn de_casteljau(pts: &Vec<Point2D>, pos: f32) -> Point2D {
    let n: usize = pts.len();
    let mut points: Vec<Point2D> = pts.clone();

    for r in 1..n {
        for i in 0..(n - r) {
            let this: Point2D = points[i];
            let next: Point2D = points[i + 1];

            // Interpolate point
            points[i] = (
                ((1.0 - pos) * this.0 as f32 + pos * next.0 as f32) as i32,
                ((1.0 - pos) * this.1 as f32 + pos * next.1 as f32) as i32,
            );
        }
    }

    points[0]
}

/// Returns the points of a Bézier curve with given precision
/// * `pts` - points describing the curve
/// * `prc` - precision, must be greater than 1
pub fn get_bezier_points(pts: &Vec<Point2D>, prc: u32) -> Vec<Point2D> {
    if pts.len() < 3 || prc < 1 {
        return Vec::new();
    }

    let mut midpoints: Vec<Point2D> = Vec::new();

    // Iterate from 0.0 to 1.0 with prc as step
    for i in 0..prc + 1 {
        // This gets exact floats
        let pos: f32 = (i as f32) / prc as f32;
        midpoints.push(de_casteljau(pts, pos));
    }

    midpoints
}
