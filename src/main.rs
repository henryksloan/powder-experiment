#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::Rng;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Empty,
    Sand,
    Water,
    Stone,
}

impl Kind {
    pub fn color(&self) -> [u8; 4] {
        match *self {
            Self::Empty => [0, 0, 0, 0],
            Self::Sand => [0xC2, 0xB2, 0x80, 0xFF],
            Self::Water => [0x00, 0x96, 0xFF, 0xFF],
            Self::Stone => [0xCC, 0xCC, 0xCC, 0xFF],
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Particle {
    kind: Kind,
    touched: bool, // To prevent updating the same logical particle multiple times per update
}

impl Particle {
    pub fn empty(&self) -> bool {
        self.kind == Kind::Empty
    }
}

impl Default for Particle {
    fn default() -> Particle {
        Particle {
            kind: Kind::Empty,
            touched: false,
        }
    }
}

struct World {
    particles: [[Particle; 320]; 240],
    clock: bool,
}

impl World {
    fn new() -> Self {
        Self {
            particles: [[Particle::default(); 320]; 240],
            clock: false,
        }
    }

    fn update(&mut self) {
        self.clock = !self.clock;
        let mut rng = rand::thread_rng();

        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                if self.particles[y][x].touched == self.clock {
                    continue;
                }
                self.particles[y][x].touched = !self.particles[y][x].touched;

                match self.particles[y][x].kind {
                    Kind::Empty | Kind::Stone => {}
                    Kind::Sand => {
                        if (y as u32) < HEIGHT - 1 {
                            if self.particles[y + 1][x].empty() {
                                self.particles[y + 1][x] = self.particles[y][x];
                                self.particles[y][x] = Particle::default();
                            } else {
                                let new_y = y + 1;
                                let new_x = x as i32 + (rng.gen::<bool>() as i32 * 2 - 1);
                                if new_x >= 0 && new_x < WIDTH as i32 {
                                    let new_x = new_x as usize;
                                    if self.particles[new_y][new_x].empty() {
                                        self.particles[new_y][new_x] = self.particles[y][x];
                                        self.particles[y][x] = Particle::default();
                                    }
                                }
                            }
                        }
                    }
                    Kind::Water => {
                        if (y as u32) < HEIGHT - 1 {
                            if self.particles[y + 1][x].empty() {
                                self.particles[y + 1][x] = self.particles[y][x];
                                self.particles[y][x] = Particle::default();
                            } else {
                                let new_y = y + 1;
                                let new_x = x as i32 + (rng.gen::<bool>() as i32 * 2 - 1);
                                if new_x >= 0 && new_x < WIDTH as i32 {
                                    let new_x = new_x as usize;
                                    if self.particles[new_y][new_x].empty() {
                                        self.particles[new_y][new_x] = self.particles[y][x];
                                        self.particles[y][x] = Particle::default();
                                    } else if self.particles[y][new_x].empty() {
                                        self.particles[y][new_x] = self.particles[y][x];
                                        self.particles[y][x] = Particle::default();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            let particle = &self.particles[y][x];

            let rgba = if particle.kind != Kind::Empty {
                particle.kind.color()
            } else {
                [0x00, 0x00, 0x00, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn set_pixel(&mut self, (x, y): (usize, usize), kind: Kind) {
        if x < WIDTH as usize && y < HEIGHT as usize {
            self.particles[y][x] = Particle {
                kind,
                touched: self.clock,
            };
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
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
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    let mut paused = false;
    let mut selected_kind = Kind::Sand;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
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

            if input.key_pressed(VirtualKeyCode::Space) {
                paused = !paused;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            if input.key_pressed(VirtualKeyCode::Key1) {
                selected_kind = Kind::Sand;
            } else if input.key_pressed(VirtualKeyCode::Key2) {
                selected_kind = Kind::Water;
            } else if input.key_pressed(VirtualKeyCode::Key3) {
                selected_kind = Kind::Stone;
            }

            let left_click = input.mouse_held(0);
            let right_click = input.mouse_held(1);
            if left_click || right_click {
                let click_kind = if left_click {
                    selected_kind
                } else {
                    Kind::Empty
                };

                if let Some(Ok(pixel_pos)) =
                    input.mouse().map(|pos| pixels.window_pos_to_pixel(pos))
                {
                    let (pixel_x, pixel_y) = (pixel_pos.0 as i32, pixel_pos.1 as i32);
                    for x_off in -1..=1 {
                        for y_off in -1..=1 {
                            world.set_pixel(
                                ((pixel_x + x_off) as usize, (pixel_y + y_off) as usize),
                                click_kind,
                            );
                        }
                    }
                }
            }

            // Update internal state and request a redraw
            if !paused {
                world.update();
            }

            window.request_redraw();
        }
    });
}
