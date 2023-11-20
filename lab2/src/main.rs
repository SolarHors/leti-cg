//! Лабораторная работа №2
//! Вариант №5
//! Группа 0323

use common::*;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// Applies De Casteljau's algorithm in order to return a point
/// on the curve at a given position
/// * `pos` - position on the curve, must be between 0 and 1
fn get_curve_point(pts: &Vec<Point2D>, pos: f32) -> Point2D {
    let n: usize = pts.len() - 1;
    let mut points: Vec<Point2D> = pts.clone();

    for r in 1..=n {
        for i in 0..=(n - r) {
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

/// Draws a Bézier curve guided by provided points with given precision
/// * `prc` - precision (must be 100 instead of 0.01 for example)
fn draw_curve(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    pts: &Vec<Point2D>,
    prc: u32,
    col: &ColorRGB,
) {
    if pts.len() < 3 || prc < 1 {
        return;
    }

    let mut midpoints: Vec<Point2D> = Vec::new();

    // Iterate from 0.0 to 1.0 with prc as step
    for i in 0..prc + 1 {
        // This gets exact floats
        let pos: f32 = (i as f32) / prc as f32;
        midpoints.push(get_curve_point(pts, pos));
    }

    draw_connect(buf, dim, &midpoints, col);
}

/// Draws circles around curve points
fn draw_points(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    pts: &Vec<Point2D>,
    col: &ColorRGB,
) {
    for p in pts {
        draw_circle(buf, dim, p, 4, col);
    }
}

fn main() {
    println!(
        "Left click to add new point.\nRight click to remove last point.\n"
    );

    // Mouse cursor position
    let mut cursor_pos: Point2D = (0, 0);
    // Curve defining points
    let mut curve_pts: Vec<Point2D> = Vec::new();

    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title("CG Lab2-Task5")
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

                // Draw everything
                draw_clear(&mut buffer, &size, &WHITE);

                if curve_pts.len() > 0 {
                    draw_connect(&mut buffer, &size, &curve_pts, &GRAY);
                    draw_points(&mut buffer, &size, &curve_pts, &GREEN);
                    draw_curve(&mut buffer, &size, &curve_pts, 24, &BLACK);
                }

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
                if state == ElementState::Released {
                    match button {
                        MouseButton::Left => {
                            println!("New point added at {:?}.", cursor_pos);
                            curve_pts.push(cursor_pos);
                            window.request_redraw();
                        }
                        MouseButton::Right => {
                            println!("Last added point removed.");
                            curve_pts.pop();
                            window.request_redraw();
                        }
                        _ => {}
                    }
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
