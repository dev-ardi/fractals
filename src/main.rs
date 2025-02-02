use std::f64;
use std::io::Write;
use std::time::{Duration, Instant};

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

const T_X_0: f64 = WIDTH as f64 / 2.0;
const T_Y_0: f64 = -(HEIGHT as f64 / 2.0);

const INITIAL_COORDS: Coords = Coords { x: 0.0, y: T_Y_0 };

const S0: f64 = 1.0;

#[derive(Debug, Clone, Copy)]
struct UserConfig {
    /// How many times to run this per frame
    iters: usize,
    branch_on: f64,
    /// Pixels
    t_x_1: f64,
    /// Pixels
    t_y_1: f64,
    scale: f64,
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
        t_x_1: T_X_0,
        t_y_1: T_Y_0,
        scale: S0,
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
                } else if input.key_held(KeyCode::ArrowUp) {
                    config.t_y_1 -= 10.0;
                    refresh = true;
                } else if input.key_held(KeyCode::ArrowDown) {
                    config.t_y_1 += 10.0;
                    refresh = true;
                } else if input.key_held(KeyCode::ArrowLeft) {
                    config.t_x_1 -= 10.0;
                    refresh = true;
                } else if input.key_held(KeyCode::ArrowRight) {
                    config.t_x_1 += 10.0;
                    refresh = true;
                } else if input.key_pressed(KeyCode::KeyZ) {
                    config.scale *= 1.33;
                    refresh = true;
                } else if input.key_pressed(KeyCode::KeyX) {
                    config.scale /= 1.33;
                    refresh = true;
                } else if input.key_pressed(KeyCode::KeyT) {
                    config.branches += 1;
                    if config.branches == 5 {
                        config.branches = 1;
                    }
                    refresh = true;
                } else if input.key_held(KeyCode::KeyA) {
                    config.iters = config.iters.saturating_add(1).clamp(1, 40);
                } else if input.key_held(KeyCode::KeyS) {
                    config.iters = config.iters.saturating_sub(1).clamp(1, 40);
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
    last_length: f64,
    remaining: u32,
    config: UserConfig,
    leaves: Leaves,
    iter_number: usize,
}

#[derive(Debug, Clone, Copy)]
struct Coords {
    x: f64,
    y: f64,
}

impl Coords {
    // From virtual cords to real cords
    fn translate_coords(self, config: &UserConfig) -> Coords {
        Coords {
            x: self.x * config.scale - config.t_x_1,
            y: self.y * config.scale - config.t_y_1,
        }
    }

    // From real cords to virtual coords
    fn translate_coords_invert(self, config: &UserConfig) -> Coords {
        // This comes from basic algebraic transformations of translate_coords
        Coords {
            x: (self.x + config.t_x_1) / config.scale,
            y: (self.y + config.t_y_1) / config.scale,
        }
    }
}

#[derive(Debug, Default)]
struct Leaves {
    down_leaves: Vec<Coords>,
    up_leaves: Vec<Coords>,
    right_leaves: Vec<Coords>,
    left_leaves: Vec<Coords>,
}

impl RenderState {
    fn new_def(config: UserConfig) -> Self {
        Self {
            last_length: HEIGHT as f64 / 2.0,
            remaining: (HEIGHT as f64 / 2.0 * config.scale) as u32,
            config,
            leaves: Leaves {
                down_leaves: vec![INITIAL_COORDS],
                up_leaves: vec![],
                right_leaves: vec![],
                left_leaves: vec![],
            },
            iter_number: 0,
        }
    }

    fn swaparoo(&mut self) {
        let t0 = Instant::now();
        eprint!("Creating new nodes...");
        self.iter_number += 1;
        // Now we need to calculate how many pixels will the total segment take

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
                down_leaves.extend_from_slice(&self.leaves.down_leaves);
                up_leaves.extend_from_slice(&self.leaves.up_leaves);
                right_leaves.extend_from_slice(&self.leaves.right_leaves);
                left_leaves.extend_from_slice(&self.leaves.left_leaves);
            }
            if self.config.branches == 4 {
                right_leaves.extend_from_slice(&self.leaves.left_leaves);
                left_leaves.extend_from_slice(&self.leaves.right_leaves);
                down_leaves.extend_from_slice(&self.leaves.up_leaves);
                up_leaves.extend_from_slice(&self.leaves.down_leaves);
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

    fn next(&mut self, buf: &mut [u8]) {
        if self.remaining == 0 {
            self.last_length /= 2.0;
            self.remaining = (self.last_length * self.config.scale) as u32;
            if self.remaining == 0 {
                std::thread::sleep(Duration::from_millis(16));
                return;
            }

            self.swaparoo();
        }

        self.remaining -= 1;

        // Need to translate coordinates to real pixels.
        // 1 - Change of coordinates to top left
        // 2 - Apply zoom
        // 3 - Don't forget that a pixel is rgba
        //
        // What does it mean to be zoomed in? It's just like if we scaled the world by the zoom.
        // What does it mean to move right? It's like we moved the world left, etc.
        // Note that zooming and moving are dependent.
        // If we zoom before moving the move would be relative to the view
        // If we zoom after moving the move would be absolute - The more we zoom the bigger the
        // movement will be.
        //
        // An optimization that I can make (but I won't for now) is to store the coords after
        // transforming, since we know that the transforms won't change throughout the life of Self.

        // Real Coords
        let min = Coords { x: 0.0, y: 0.0 }.translate_coords_invert(&self.config);
        let max = Coords {
            x: WIDTH as f64,
            y: HEIGHT as f64,
        }
        .translate_coords_invert(&self.config);

        // This delta is len / (len * scale).
        let delta = 1.0 / self.config.scale;

        self.leaves
            .left_leaves
            .iter_mut()
            .map(|coords| {
                coords.x -= delta;
                *coords
            })
            .chain(self.leaves.right_leaves.iter_mut().map(|coords| {
                coords.x += delta;
                *coords
            }))
            .chain(self.leaves.up_leaves.iter_mut().map(|coords| {
                coords.y -= delta;
                *coords
            }))
            .chain(self.leaves.down_leaves.iter_mut().map(|coords| {
                coords.y += delta;
                *coords
            }))
            // Filter out of bounds
            .map(|coord| coord.translate_coords(&self.config))
            .filter(|coord| {
                coord.x < min.x || coord.x > max.x || coord.y < min.y || coord.y > max.y
            })
            // Render index
            .for_each(|coords| {
                let colors = [
                    // [0, 0, 0, 0],
                    // [0, 0, 0, 0],
                    // [0, 0, 0, 0],
                    // [0, 0, 0, 0],
                    // [0, 0, 0, 0],
                    // [0, 0, 0, 0],
                    [140, 4, 40, 255], // Carmin [140, 4, 40, 255], // Carmin
                    [140, 4, 40, 255], // Carmin
                    [140, 4, 40, 255], // Carmin
                    [115, 0, 13, 255],
                    [222, 87, 123, 255],
                    [140, 4, 40, 255],   // Carmin
                    [222, 87, 123, 255], // Rojo raro
                    [222, 87, 123, 255],
                    [140, 4, 40, 255], // Carmin
                    [115, 0, 13, 255],
                    [140, 4, 40, 255],    // Carmin
                    [222, 87, 123, 255],  // Rojo raro
                    [230, 119, 184, 255], // Rosa
                ];
                let current_color = colors[self.iter_number % colors.len()]; // Transform coords to index
                let idx = (coords.y * WIDTH as f64 + coords.x).round_ties_even() as usize;
                match buf.get_mut(idx * 4..idx * 4 + 4) {
                    Some(mut slice) => slice.write_all(&current_color).unwrap(),
                    None => {
                        // dbg!(coords, self.config, min, max);
                        // panic!("Out of range {idx}");
                    }
                }
            });
    }
}
