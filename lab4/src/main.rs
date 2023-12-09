//! Лабораторная работа №4
//! Вариант №3
//! Группа 0323

use common::*;
use rand::Rng;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// Cohen–Sutherland's outcodes for directions
const INSIDE: u8 = 0b0000;
const LEFT: u8 = 0b0001;
const RIGHT: u8 = 0b0010;
const BOTTOM: u8 = 0b0100;
const TOP: u8 = 0b1000;

// Get outcode of the given point relative to a rectangle
fn get_outcode(point: &Point2D, rect: &(Point2D, Point2D)) -> u8 {
    let mut result: u8 = INSIDE;

    // To the left or right of rect
    if point.0 < rect.0 .0 {
        result |= LEFT;
    } else if point.0 > rect.1 .0 {
        result |= RIGHT;
    }

    // Below or above rect
    if point.1 < rect.0 .1 {
        result |= BOTTOM;
    } else if point.1 > rect.1 .1 {
        result |= TOP;
    }

    result
}

/// Cohen–Sutherland clipping algorithm
/// Returns the clipped segment of a line
fn get_clipped_line(
    line: &(Point2D, Point2D),
    rect: &(Point2D, Point2D),
) -> Option<(Point2D, Point2D)> {
    let mut cline: (Point2D, Point2D) = line.clone();
    let mut outcode0: u8 = get_outcode(&cline.0, rect);
    let mut outcode1: u8 = get_outcode(&cline.1, rect);
    let mut accept: bool = false;

    loop {
        if (outcode0 | outcode1) == 0 {
            // Both points are inside of the rectangle
            accept = true;
            break;
        } else if (outcode0 & outcode1) != 0 {
            // Both points are outside of the rectangle
            break;
        } else {
            // At least one endpoint is inside the rectangle
            let mut point: Point2D = (0, 0);

            // Get the outside endpoint
            let outoutcode = if outcode1 > outcode0 {
                outcode1
            } else {
                outcode0
            };

            // Find intersection point
            if (outoutcode & TOP) != 0 {
                // Point is above the rectangle
                point.0 = cline.0 .0
                    + (cline.1 .0 - cline.0 .0) * (rect.1 .1 - cline.0 .1)
                        / (cline.1 .1 - cline.0 .1);
                point.1 = rect.1 .1;
            } else if (outoutcode & BOTTOM) != 0 {
                // Point is below the rectangle
                point.0 = cline.0 .0
                    + (cline.1 .0 - cline.0 .0) * (rect.0 .1 - cline.0 .1)
                        / (cline.1 .1 - cline.0 .1);
                point.1 = rect.0 .1;
            } else if (outoutcode & RIGHT) != 0 {
                // Point is right of the rectangle
                point.1 = cline.0 .1
                    + (cline.1 .1 - cline.0 .1) * (rect.1 .0 - cline.0 .0)
                        / (cline.1 .0 - cline.0 .0);
                point.0 = rect.1 .0;
            } else if (outoutcode & LEFT) != 0 {
                // Point is left of the rectangle
                point.1 = cline.0 .1
                    + (cline.1 .1 - cline.0 .1) * (rect.0 .0 - cline.0 .0)
                        / (cline.1 .0 - cline.0 .0);
                point.0 = rect.0 .0;
            }

            // Move outside point to intersection and start next pass
            if outoutcode == outcode0 {
                cline.0 = point;
                outcode0 = get_outcode(&cline.0, rect);
            } else {
                cline.1 = point;
                outcode1 = get_outcode(&cline.1, rect);
            }
        }
    }

    // Return the segment inside the window
    // if such a segment exists
    if accept {
        return Some(cline);
    } else {
        return None;
    }
}

/// Draws line segments
fn draw_segments(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    lines: &[(Point2D, Point2D)],
    col: &ColorRGB,
) {
    for l in lines {
        draw_line(buf, dim, &l.0, &l.1, col);
    }
}

/// Generates a random point
fn get_rng_point(width: i32, height: i32) -> Point2D {
    (
        rand::thread_rng().gen_range(10..width),
        rand::thread_rng().gen_range(10..height),
    )
}

#[allow(unused_assignments)]
fn main() {
    println!("Scroll mouse wheel to change rectangle size.\n");

    // Generated line segments
    let mut lines: Vec<(Point2D, Point2D)> = Vec::new();
    // Points, describing the rectangular selection -
    // first point is on lower left, second is higher right
    let mut rect: (Point2D, Point2D) = ((200, 100), (600, 500));
    // Size of the selection
    let mut rect_size: (i32, i32) = (100, 100);
    // Center of the window
    let mut center: Point2D = (400, 300);

    // Fill lines with random points
    for _ in 0u8..12 {
        lines.push((get_rng_point(800, 600), get_rng_point(800, 600)));
    }

    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title("CG Lab4-Task3")
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

                // Update window center
                center = ((size.width / 2) as i32, (size.height / 2) as i32);

                // Update rectangle coordinates
                rect = (
                    (center.0 - rect_size.0, center.1 - rect_size.1),
                    (center.0 + rect_size.0, center.1 + rect_size.1),
                );

                // Clear buffer
                draw_clear(&mut buffer, &size, &WHITE);

                // Draw randomly generated lines
                draw_segments(&mut buffer, &size, &lines, &GRAY);

                // Find and draw clipped line segments
                for l in &lines {
                    if let Some(v) = get_clipped_line(&l, &rect) {
                        draw_line(&mut buffer, &size, &v.0, &v.1, &GREEN);
                        draw_circle(&mut buffer, &size, &v.0, 3, &BLUE);
                        draw_circle(&mut buffer, &size, &v.1, 3, &BLUE);
                    }
                }

                // Draw rectangle
                draw_polygon(
                    &mut buffer,
                    &size,
                    &[
                        rect.0,
                        (rect.0 .0, rect.1 .1),
                        rect.1,
                        (rect.1 .0, rect.0 .1),
                    ],
                    &BLACK,
                );

                buffer.present().unwrap();
            }

            // Capture mouse wheel events
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                window_id,
            } if window_id == window.id() => {
                if let MouseScrollDelta::LineDelta(_, ld) = delta {
                    // Change selection square size
                    if ld < 0. && rect_size.0 > 10 && rect_size.1 > 10 {
                        rect_size.0 -= 4;
                        rect_size.1 -= 4;
                    } else {
                        rect_size.0 += 4;
                        rect_size.1 += 4;
                    }
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
