//! Лабораторная работа №2
//! Вариант №5
//! Группа 0323

use common::*;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
                    let curve = get_bezier_points(&curve_pts, 20);
                    draw_connect(&mut buffer, &size, &curve, &BLACK);
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
