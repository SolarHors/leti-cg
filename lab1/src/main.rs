//! Лабораторная работа №1
//! Вариант №1
//! Группа 0323

use common::*;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
