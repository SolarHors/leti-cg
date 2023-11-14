//! Лабораторная работа №1
//! Вариант №1
//! Группа 0323

use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

type Point2D = (i32, i32);
type ColorRGB = (u8, u8, u8);

// const BLACK: ColorRGB = (0x00, 0x00, 0x00);
const GRAY: ColorRGB = (0xde, 0xdd, 0xda);
const WHITE: ColorRGB = (0xff, 0xff, 0xff);
// const RED: ColorRGB = (0xe0, 0x1b, 0x24);
// const GREEN: ColorRGB = (0x2e, 0xc2, 0x7e);
const BLUE: ColorRGB = (0x35, 0x84, 0xe4);

// /// Moves data from softbuffer::DrawTarget into raqote::Surface
// fn draw_to_buffer(dt: &DrawTarget, buffer: &mut [u32], width: usize, height: usize) {
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
fn draw_clear(buf: &mut [u32], dim: &PhysicalSize<u32>, col: &ColorRGB) {
    for y in 0..dim.height {
        for x in 0..dim.width {
            let index = y as usize * dim.width as usize + x as usize;
            buf[index] = (col.2 as u32) | ((col.1 as u32) << 8) | ((col.0 as u32) << 16);
        }
    }
}

/// Draws a pixel of a specified color into the buffer
fn draw_pixel(buf: &mut [u32], dim: &PhysicalSize<u32>, pos: &Point2D, col: &ColorRGB) {
    if pos.0 >= dim.width as i32 || pos.1 >= dim.height as i32 || pos.0 < 0 || pos.1 < 0 {
        return;
    }

    // Color is stored as (B | G << 8 | R << 16)
    buf[(pos.1 * dim.width as i32 + pos.0) as usize] =
        ((col.2 as u32) | ((col.1 as u32) << 8) | ((col.0 as u32) << 16)) as u32;
}

/// Bresenham's line algorithm with variable tilt
/// * `slope` - tilt, value of `true` means it tilts downwards, else upwards
fn draw_line_slope(
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
fn draw_line(buf: &mut [u32], dim: &PhysicalSize<u32>, p0: &Point2D, p1: &Point2D, col: &ColorRGB) {
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

/// Draw origin point
fn draw_origin(buf: &mut [u32], dim: &PhysicalSize<u32>, org: &Point2D, col: &ColorRGB) {
    draw_line(buf, dim, &(org.0, 0), &(org.0, dim.height as i32 - 1), col);
    draw_line(buf, dim, &(0, org.1), &(dim.width as i32 - 1, org.1), col);
}

/// Draw a line between all given points
fn draw_polygon(buf: &mut [u32], dim: &PhysicalSize<u32>, poly: &[Point2D], col: &ColorRGB) {
    let len: usize = poly.len();
    for i in 0..len - 1 {
        draw_line(buf, dim, &poly[i], &poly[i + 1], col);
    }
    draw_line(buf, dim, &poly[len - 1], &poly[0], col);
}

/// Change the scale of a polygon
fn scale_polygon(poly: &mut [Point2D], sfx: i32, sfy: i32) {
    let len: usize = poly.len();
    for i in 0..len {
        poly[i].0 = poly[0].0 + ((poly[i].0 - poly[0].0) * sfx);
        poly[i].1 = poly[0].1 + ((poly[i].1 - poly[0].1) * sfy);
    }
}

fn main() {
    println!("Use mouse wheel to change scale.\nLeft click to set object's origin.\n");

    // Origin point for the 2D shape
    let mut origin: Point2D = (25, 25);
    // Scale of the 2D shape
    let mut scale: i32 = 1;
    // Mouse cursor position
    let mut cursor_pos: Point2D = (0, 0);

    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title("CG Lab1-Task1")
        .with_inner_size(LogicalSize::new(800., 600.))
        .with_resizable(true);

    let window = window_builder.build(&event_loop).unwrap();

    let context = unsafe { Context::new(&window) }.unwrap();
    let mut surface = unsafe { Surface::new(&context, &window) }.unwrap();

    let _ = event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let size = window.inner_size();

                surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();

                // Define a triangle
                let mut triangle: [Point2D; 3] = [
                    (origin.0, origin.1),
                    (origin.0 + 60, origin.1 + 60),
                    (origin.0, origin.1 + 140),
                ];

                // Draw everything
                draw_clear(&mut buffer, &size, &WHITE);
                draw_origin(&mut buffer, &size, &origin, &GRAY);
                scale_polygon(&mut triangle, scale, scale);
                draw_polygon(&mut buffer, &size, &triangle, &BLUE);

                buffer.present().unwrap();
            }

            // Capture mouse cursor position inside of the window
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
            } if window_id == window.id() => {
                // Update mouse cursor position
                cursor_pos = (position.x as i32, position.y as i32);
            }

            // Capture mouse button press events
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                window_id,
            } if window_id == window.id() => {
                // Update the origin point
                if state == ElementState::Released && button == MouseButton::Left {
                    origin = (cursor_pos.0, cursor_pos.1);
                    println!("Origin point updated: {:?}", origin);
                    window.request_redraw();
                }
            }

            // Capture mouse wheel events
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                window_id,
            } if window_id == window.id() => {
                if let MouseScrollDelta::LineDelta(_, ld) = delta {
                    // Change object scale
                    scale += ld as i32;
                    window.request_redraw();
                }
            }

            // Exit the program
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }

            _ => {}
        }
    });
}
