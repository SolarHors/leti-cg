//! Лабораторная работа №3
//! Вариант №2
//! Группа 0323

use common::*;
use nalgebra::*;
// use raqote::*;
use softbuffer::{Buffer, Context, Surface};
use std::num::NonZeroU32;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{
        ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// Defines which axis is being modfied
enum ModifiedAxis {
    X,
    Y,
    Z,
}

// /// Moves data from raqote::DrawTarget into softbuffer::Surface
// pub fn draw_to_buffer(
//     dt: &DrawTarget,
//     buffer: &mut Buffer,
//     width: usize,
//     height: usize,
// ) {
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
//                 ((blue as u32) | ((green as u32) << 8) | ((red as u32) << 16))
//                     as u32
//             };

//             buffer[pos] = value;
//         }
//     }
// }

fn get_xyz_rotation_matrices(
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    (
        // Rotation along X
        Matrix4::from_rows(&[
            RowVector4::new(1., 0., 0., 0.),
            RowVector4::new(0., angle_x.cos(), -angle_x.sin(), 0.),
            RowVector4::new(0., angle_x.sin(), angle_x.cos(), 0.),
            RowVector4::new(0., 0., 0., 1.),
        ]),
        // Rotation along Y
        Matrix4::from_rows(&[
            RowVector4::new(angle_y.cos(), 0., angle_y.sin(), 0.),
            RowVector4::new(0., 1., 0., 0.),
            RowVector4::new(-angle_y.sin(), 0., angle_y.sin(), 0.),
            RowVector4::new(0., 0., 0., 1.),
        ]),
        // Rotation along Z
        Matrix4::from_rows(&[
            RowVector4::new(angle_z.cos(), -angle_z.sin(), 0., 0.),
            RowVector4::new(angle_z.sin(), angle_z.cos(), 0., 0.),
            RowVector4::new(0., 0., 1., 0.),
            RowVector4::new(0., 0., 0., 1.),
        ]),
    )
}

/// Returns points that have been projected
/// to represent the rotation
fn get_projected_points(
    pts: &Vec<Matrix4x1<f32>>,
    prj_mat: &Matrix2x4<f32>,
    cen_pos: &Point2<f32>,
    rot_x: &Matrix4<f32>,
    rot_y: &Matrix4<f32>,
    rot_z: &Matrix4<f32>,
) -> Vec<Vector2<f32>> {
    let mut result: Vec<Vector2<f32>> = Vec::new();

    for pt in pts {
        // Apply rotation along all axes
        let mut rotated = rot_y * pt;
        rotated = rot_x * rotated;
        rotated = rot_z * rotated;

        // Project points
        let projected = prj_mat * rotated;

        result.push(Vector2::new(
            projected[(0, 0)] + cen_pos.x,
            projected[(1, 0)] + cen_pos.y,
        ))
    }

    result
}

/// Creates a bilinear surface from projected points
fn make_bilinear_surface(pts: &Vec<Vector2<f32>>) -> Matrix6<Vector2<f32>> {
    let mut result: Matrix6<Vector2<f32>> = Matrix6::default();

    // Value w and u are from 0.0 to 1.0 with step 0.2
    for i in 0usize..6 {
        let w: f32 = i as f32 * 0.2;

        for j in 0usize..6 {
            let u: f32 = j as f32 * 0.2;

            // Apply bilinear surface equation
            let point = pts[0] * ((1. - u) * (1. - w))
                + pts[1] * ((1. - u) * w)
                + pts[2] * (u * (1. - w))
                + pts[3] * (u * w);

            result[(i, j)] = point;
        }
    }

    result
}

/// Draws the surface
fn draw_bilinear_surface(
    buf: &mut [u32],
    dim: &PhysicalSize<u32>,
    surf: &Matrix6<Vector2<f32>>,
) {
    for i in 0..surf.nrows() - 1 {
        for j in 0..surf.ncols() {
            draw_line(
                buf,
                dim,
                &(surf[(i, j)].x as i32, surf[(i, j)].y as i32),
                &(surf[(i + 1, j)].x as i32, surf[(i + 1, j)].y as i32),
                &GRAY,
            );
            draw_line(
                buf,
                dim,
                &(surf[(j, i)].x as i32, surf[(j, i)].y as i32),
                &(surf[(j, i + 1)].x as i32, surf[(j, i + 1)].y as i32),
                &GRAY,
            );
        }
    }
    for i in 0..surf.nrows() {
        for j in 0..surf.ncols() {
            draw_circle(
                buf,
                dim,
                &(surf[(i, j)].x as i32, surf[(i, j)].y as i32),
                2,
                &BLUE,
            );
        }
    }
}

fn main() {
    println!(
        "Left click to add new point.\nRight click to remove last point.\nX, Y, Z keys to edit appropriate axes.\nMouse wheel to change angle.\n"
    );

    // Mouse cursor position
    let mut cursor_pos: Point2<f32> = Point2::new(0., 0.);

    // Central position
    let center_pos: Point2<f32> = Point2::new(400., 300.);

    // Which axis is being modified
    let mut modified_axis: ModifiedAxis = ModifiedAxis::X;

    // Points for the surface
    // Each point is a column of 4 values:
    //    X, Y, Z, and whatever the last one is,
    //    its just 1.0 by default
    let mut surf_points: Vec<Matrix4x1<f32>> = Vec::new();

    // Rotation angles of the surface: X, Y, Z
    let mut surf_rot: (f32, f32, f32) = (0., 0., 0.);

    // Define the projection matrix
    let proj_matrix = Matrix2x4::from_rows(&[
        RowVector4::new(1., 0., 0., 0.),
        RowVector4::new(0., 1., 0., 0.),
    ]);

    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title("CG Lab3-Task2")
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
                // let mut draw =
                //     DrawTarget::new(size.width as i32, size.height as i32);

                // Get rotation matrices
                let (rot_x, rot_y, rot_z) = get_xyz_rotation_matrices(
                    surf_rot.0, surf_rot.1, surf_rot.2,
                );

                // Get projected points
                let surf_proj_points = get_projected_points(
                    &surf_points,
                    &proj_matrix,
                    &center_pos,
                    &rot_x,
                    &rot_y,
                    &rot_z,
                );

                // Fill in the surface

                // Convert points for my draw functions
                let mut converted_proj_pts: Vec<(i32, i32)> = Vec::new();
                for p in &surf_proj_points {
                    converted_proj_pts.push((p.x as i32, p.y as i32));
                }

                draw_clear(&mut buffer, &size, &WHITE);

                if converted_proj_pts.len() == 4 {
                    let surf_fill_points =
                        make_bilinear_surface(&surf_proj_points);
                    // draw_line(
                    //     &mut buffer,
                    //     &size,
                    //     &converted_proj_pts[2],
                    //     &converted_proj_pts[3],
                    //     &BLACK,
                    // );
                    draw_bilinear_surface(
                        &mut buffer,
                        &size,
                        &surf_fill_points,
                    );
                } else if converted_proj_pts.len() >= 2 {
                    draw_line(
                        &mut buffer,
                        &size,
                        &converted_proj_pts[0],
                        &converted_proj_pts[1],
                        &BLACK,
                    );
                }

                draw_points(&mut buffer, &size, &converted_proj_pts, &RED);

                buffer.present().unwrap();
            }

            // Capture mouse cursor position inside of the window
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
            } if window_id == window.id() => {
                // Update mouse cursor position
                cursor_pos.x = position.x as f32;
                cursor_pos.y = position.y as f32;
            }

            // Capture mouse button press events
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                window_id,
            } if window_id == window.id() => {
                if state == ElementState::Released {
                    match button {
                        MouseButton::Left => {
                            // Add surface points, but
                            // no more than for 2 lines
                            if surf_points.len() < 4 {
                                println!(
                                    "New point added at ({}, {}, {})",
                                    cursor_pos.x,
                                    cursor_pos.y,
                                    (cursor_pos.x - cursor_pos.y).abs()
                                );

                                surf_points.push(Matrix4x1::new(
                                    cursor_pos.x - center_pos.x,
                                    cursor_pos.y - center_pos.y,
                                    (cursor_pos.x - cursor_pos.y).abs(),
                                    1.,
                                ));

                                window.request_redraw();
                            } else {
                                println!("Cannot add more than 4 points");
                            }
                        }
                        MouseButton::Right => {
                            surf_points.pop();
                            surf_rot = (0., 0., 0.);
                            println!("Last point was removed");
                            window.request_redraw();
                        }
                        _ => {}
                    }
                }
            }

            // Capture keyboard events
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                window_id,
            } if window_id == window.id() => {
                if input.state == ElementState::Released {
                    if let Some(key) = input.virtual_keycode {
                        match key {
                            VirtualKeyCode::X => {
                                println!("Rotating along X");
                                modified_axis = ModifiedAxis::X;
                            }
                            VirtualKeyCode::Y => {
                                println!("Rotating along Y");
                                modified_axis = ModifiedAxis::Y;
                            }
                            VirtualKeyCode::Z => {
                                println!("Rotating along Z");
                                modified_axis = ModifiedAxis::Z;
                            }
                            _ => {}
                        }
                    }
                } else {
                    if let Some(key) = input.virtual_keycode {
                        match key {
                            _ => {}
                        }
                    }
                }
            }

            // Capture mouse wheel events
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                window_id,
            } if window_id == window.id() => {
                if let MouseScrollDelta::LineDelta(_, ld) = delta {
                    // Change rotation along an axis
                    match modified_axis {
                        ModifiedAxis::X => {
                            if ld >= 0. {
                                surf_rot.0 += 0.10;
                            } else {
                                surf_rot.0 -= 0.10;
                            }
                        }
                        ModifiedAxis::Y => {
                            if ld >= 0. {
                                surf_rot.1 += 0.10;
                            } else {
                                surf_rot.1 -= 0.10;
                            }
                        }
                        ModifiedAxis::Z => {
                            if ld >= 0. {
                                surf_rot.2 += 0.10;
                            } else {
                                surf_rot.2 -= 0.10;
                            }
                        }
                    }
                    // println!("Angle changed {:?}", surf_rot);
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

/*
NOTE: Ideas on implementation:

User defines two boundary curves in 2d.
One of the curves is offset on the Z axis.
They get connected, making a surface.
To make a grid, use De Casteljau's algorithm for curves
or lerp for lines in order to evenly divide boundaries
into points to draw half of the grid, next do the same
for the lines that connect the corners to draw the other
half of the grid.
To have a smooth transition between grid curves
lerp between the lowest and highest points (flatten
the z axis).

How to project 3D stuff?
How to rotate the surface?
*/
