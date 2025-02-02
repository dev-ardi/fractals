use std::time::Instant;

use pixels::wgpu::Color;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

#[derive(Debug, Clone, Copy)]
struct UserConfig {
    /// How many times to run this per frame
    iters: usize,
    branch_on: f64,
    /// Pixels
    offset_x: f64,
    /// Pixels
    offset_y: f64,
    zoom: f64,
    // 1-4
    branches: u8,
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
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
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };
    pixels.clear_color(Color::BLACK);

    let mut config = UserConfig {
        iters: 10,
        offset_x: 0.0,
        offset_y: 0.0,
        zoom: 1.0,
        branch_on: 0.5,
        // 1-4
        branches: 3,
    };
    let mut state = RenderState::new_def(config);
    event_loop
        .run(|event, window_target| {
            // Draw the current frame
            if let Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } = event
            {
                let t0 = Instant::now();
                eprint!("Rendering {} nodes... ", state.leaves.left_leaves.len() * 4);
                for _ in 0..config.iters {
                    state.next(pixels.frame_mut());
                }
                eprintln!("done in: {:?}", t0.elapsed());
                pixels.render().unwrap();
            }

            // Handle input events
            if input.update(&event) {
                let mut refresh = false;
                // Close events
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    window_target.exit();
                } else if input.key_pressed(KeyCode::ArrowUp) {
                    config.offset_y -= 1.0;
                    refresh = true;
                } else if input.key_pressed(KeyCode::ArrowDown) {
                    config.offset_y += 1.0;
                    refresh = true;
                } else if input.key_pressed(KeyCode::ArrowLeft) {
                    config.offset_x -= 1.0;
                    refresh = true;
                } else if input.key_pressed(KeyCode::ArrowRight) {
                    config.offset_x += 1.0;
                    refresh = true;
                } else if input.key_pressed(KeyCode::KeyZ) {
                    config.zoom += 0.1;
                    refresh = true;
                } else if input.key_pressed(KeyCode::KeyX) {
                    config.offset_x -= 0.1;
                    refresh = true;
                } else if input.key_pressed(KeyCode::KeyT) {
                    config.branches += 1;
                    if config.branches == 5 {
                        config.branches = 1;
                    }
                    refresh = true;
                }

                if refresh {
                    pixels.frame_mut().fill(0);
                    state = RenderState::new_def(config);
                }

                if let Some(size) = input.window_resized() {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                // world.update();
            }
            window.request_redraw();
        })
        .unwrap();
}

struct RenderState {
    last_length: u16,
    remaining: u16,
    config: UserConfig,
    leaves: Leaves,
}

#[derive(Debug, Default)]
struct Leaves {
    down_leaves: Vec<usize>,
    up_leaves: Vec<usize>,
    right_leaves: Vec<usize>,
    left_leaves: Vec<usize>,
}

impl RenderState {
    fn new_def(config: UserConfig) -> Self {
        Self {
            last_length: HEIGHT as u16 / 2,
            remaining: HEIGHT as u16 / 2,
            config,
            leaves: Leaves {
                down_leaves: vec![WIDTH / 2],
                up_leaves: vec![],
                right_leaves: vec![],
                left_leaves: vec![],
            },
        }
    }

    fn next(&mut self, buf: &mut [u8]) {
        if self.remaining == 0 {
            let t0 = Instant::now();
            eprint!("Creating new nodes...");
            self.last_length /= 2;
            self.remaining = self.last_length;
            if self.last_length == 0 {
                return;
            }

            let mut down_leaves = vec![];
            let mut up_leaves = vec![];
            let mut right_leaves = vec![];
            let mut left_leaves = vec![];

            down_leaves.extend_from_slice(&self.leaves.left_leaves);
            up_leaves.extend_from_slice(&self.leaves.right_leaves);
            left_leaves.extend_from_slice(&self.leaves.down_leaves);
            right_leaves.extend_from_slice(&self.leaves.up_leaves);

            if self.config.branches >= 2 {
                down_leaves.extend_from_slice(&self.leaves.right_leaves);
                up_leaves.extend_from_slice(&self.leaves.left_leaves);
                right_leaves.extend_from_slice(&self.leaves.down_leaves);
                left_leaves.extend_from_slice(&self.leaves.up_leaves);

                if self.config.branches >= 3 {
                    down_leaves.extend_from_slice(&self.leaves.up_leaves);
                    up_leaves.extend_from_slice(&self.leaves.down_leaves);
                    right_leaves.extend_from_slice(&self.leaves.left_leaves);
                    left_leaves.extend_from_slice(&self.leaves.right_leaves);
                }
                if self.config.branches == 4 {
                    down_leaves.extend_from_slice(&self.leaves.down_leaves);
                    up_leaves.extend_from_slice(&self.leaves.up_leaves);
                    right_leaves.extend_from_slice(&self.leaves.right_leaves);
                    left_leaves.extend_from_slice(&self.leaves.left_leaves);
                }
            }

            self.leaves = Leaves {
                down_leaves,
                up_leaves,
                right_leaves,
                left_leaves,
            };

            eprintln!(" done in {:?}", t0.elapsed());
        }

        self.remaining -= 1;
        // TODO: Use normal coordinates
        for curr in &mut self.leaves.left_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr -= 1;
        }
        for curr in &mut self.leaves.right_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr += 1;
        }
        for curr in &mut self.leaves.up_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr -= WIDTH;
        }
        for curr in &mut self.leaves.down_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr += WIDTH;
        }
    }
}
