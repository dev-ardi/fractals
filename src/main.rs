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
    third_branch: bool,
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
        third_branch: true,
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
                    config.third_branch = !config.third_branch;
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

            let Leaves {
                down_leaves,
                up_leaves,
                right_leaves,
                left_leaves,
            } = std::mem::take(&mut self.leaves);

            // The down leaves become right + left
            // The up leaves become right + left
            // The left leaves become up + down
            // The right leaves become up + down

            let len = up_leaves.len();

            // Let's do some swaparoos

            let mut new_down = right_leaves;
            let mut new_up = left_leaves;
            let mut new_right = up_leaves;
            let mut new_left = down_leaves;

            // We don't need to reserve more space because extend already does it for us.
            new_down.extend_from_slice(&new_up);
            new_right.extend_from_slice(&new_left);
            // This part is trickier: We take the old slice, what formerly was the other and copy
            // it here
            // down was right
            new_left.extend_from_slice(&new_down[0..len]);
            // right was up
            new_up.extend_from_slice(&new_right[0..len]);

            self.leaves = Leaves {
                down_leaves: new_down,
                up_leaves: new_up,
                right_leaves: new_right,
                left_leaves: new_left,
            };

            eprintln!(" done in {:?}", t0.elapsed());
        }

        self.remaining -= 1;
        // TODO: Use normal coordinates
        for curr in &mut self.leaves.right_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr -= 1;
        }
        for curr in &mut self.leaves.left_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr += 1;
        }
        for curr in &mut self.leaves.right_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr -= WIDTH;
        }
        for curr in &mut self.leaves.right_leaves {
            buf[*curr * 4..*curr * 4 + 4].fill(255);
            *curr += WIDTH;
        }
    }
}
