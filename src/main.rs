#![deny(clippy::all)]
#![forbid(unsafe_code)]
use chip8::Chip8;
use once_cell::unsync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use log::error;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), Error> {
    let keys = Lazy::new(|| {
        let mut hash_map: HashMap<VirtualKeyCode, u8> = HashMap::new();

        hash_map.insert(VirtualKeyCode::Key1, 1);
        hash_map.insert(VirtualKeyCode::Key2, 2);
        hash_map.insert(VirtualKeyCode::Key3, 3);
        hash_map.insert(VirtualKeyCode::Key4, 0xC);
        hash_map.insert(VirtualKeyCode::Q, 4);
        hash_map.insert(VirtualKeyCode::W, 5);
        hash_map.insert(VirtualKeyCode::E, 6);
        hash_map.insert(VirtualKeyCode::R, 0xD);
        hash_map.insert(VirtualKeyCode::A, 7);
        hash_map.insert(VirtualKeyCode::S, 8);
        hash_map.insert(VirtualKeyCode::D, 9);
        hash_map.insert(VirtualKeyCode::F, 0xE);
        hash_map.insert(VirtualKeyCode::Z, 0xA);
        hash_map.insert(VirtualKeyCode::X, 0x0);
        hash_map.insert(VirtualKeyCode::C, 0xB);
        hash_map.insert(VirtualKeyCode::V, 0xF);
        return hash_map;
    });

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(64 as f64, 32 as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64 as u32, 32 as u32, surface_texture)?
    };
    let mut key: Option<u8> = None;
    let mut chip8: Chip8 = Chip8::setup(&Path::new("chip8-test-suite.ch8"));
    let mut instructions: u32 = 0;
    let mut time = Instant::now();
    let mut other_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if instructions < 700001 {
            if chip8.run_instruction(&mut key, &mut other_time) {
                window.request_redraw();
            }
            instructions += 1;
        }
        if time.elapsed() >= Duration::new(1, 0) {
            instructions = 0;
            time = Instant::now();
        }
        if let Event::RedrawRequested(_) = event {
            let buffer: Vec<u8> = chip8.draw();

            pixels.get_frame().copy_from_slice(&buffer);
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
            for (x, k) in keys.iter() {
                if input.key_pressed(*x) || input.key_held(*x) {
                    key = Some(*k);
                }
            }
        }
    });
}
