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
                eprint!("Rendering {} nodes... ", state.leaves.len());
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
    leaves: Vec<Substate>,
    config: UserConfig,
}

impl RenderState {
    fn new_def(config: UserConfig) -> Self {
        Self::new(
            Substate {
                direction: Direction::YPos,
                last: WIDTH / 2,
            },
            HEIGHT as u16 / 2,
            config,
        )
    }
    fn new(initial: Substate, length: u16, config: UserConfig) -> Self {
        Self {
            last_length: length,
            remaining: length,
            leaves: vec![initial],
            config,
        }
    }
    fn next(&mut self, buf: &mut [u8]) {
        if self.remaining == 0 {
            let t0 = Instant::now();
            eprint!("Creating new nodes...");
            let len = self.leaves.len();
            self.last_length /= 2;
            self.remaining = self.last_length;
            if self.last_length == 0 {
                return;
            }

            self.leaves.reserve(len);
            for i in 0..len {
                let mut curr = self.leaves[i];
                match curr.direction {
                    Direction::XPos | Direction::XNeg => {
                        curr.direction = Direction::YPos;
                        self.leaves.push(curr);
                        curr.direction = Direction::YNeg;
                        self.leaves[i] = curr;
                    }
                    Direction::YPos | Direction::YNeg => {
                        curr.direction = Direction::XPos;
                        self.leaves.push(curr);
                        curr.direction = Direction::XNeg;
                        self.leaves[i] = curr;
                    }
                }
            }
            eprintln!(" done in {:?}", t0.elapsed());
        }

        self.remaining -= 1;
        for leaf in &mut self.leaves {
            leaf.render(buf, self.config);
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct Substate {
    direction: Direction,
    last: usize,
}

impl Substate {
    fn render(&mut self, buf: &mut [u8], config: UserConfig) {
        let curr = match self.direction {
            Direction::XPos => self.last + 1,
            Direction::XNeg => self.last.saturating_sub(1),
            Direction::YPos => self.last + WIDTH,
            Direction::YNeg => self.last.saturating_sub(WIDTH),
        };
        if (curr * 4 + 4) > buf.len() {
            return;
        }
        buf[curr * 4..curr * 4 + 4].fill(255);
        self.last = curr;
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    XPos,
    YPos,
    XNeg,
    YNeg,
}
