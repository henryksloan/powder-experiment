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

const GRID_WIDTH: u32 = 320;
const GRID_HEIGHT: u32 = 240;

const TOOLBAR_HEIGHT: u32 = 30;

const WIN_WIDTH: u32 = GRID_WIDTH;
const WIN_HEIGHT: u32 = GRID_HEIGHT + TOOLBAR_HEIGHT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Empty,
    Sand,
    Gravel,
    Water,
    Stone,
}

impl Kind {
    pub fn color(&self) -> [u8; 4] {
        match *self {
            Self::Empty => [0, 0, 0, 0],
            Self::Sand => [0xC2, 0xB2, 0x80, 0xFF],
            Self::Gravel => [0x60, 0x60, 0x60, 0xFF],
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

        let x_ord_hack: Vec<usize> = if self.clock {
            (0..GRID_WIDTH as usize).collect()
        } else {
            ((0..GRID_WIDTH as usize).rev()).collect()
        };
        for y in (0..GRID_HEIGHT as usize).rev() {
            for &x in &x_ord_hack {
                if self.particles[y][x].touched == self.clock {
                    continue;
                }
                self.particles[y][x].touched = !self.particles[y][x].touched;

                match self.particles[y][x].kind {
                    Kind::Empty | Kind::Stone => {}
                    Kind::Sand => {
                        if (y as u32) < GRID_HEIGHT - 1 {
                            if self.particles[y + 1][x].empty()
                                || self.particles[y + 1][x].kind == Kind::Water
                            {
                                let self_kind = self.particles[y][x];
                                self.particles[y][x] = self.particles[y + 1][x];
                                self.particles[y + 1][x] = self_kind;
                            } else {
                                let new_y = y + 1;
                                let new_x = x as i32 + (rng.gen::<bool>() as i32 * 2 - 1);
                                if new_x >= 0 && new_x < GRID_WIDTH as i32 {
                                    let new_x = new_x as usize;
                                    if self.particles[new_y][new_x].empty()
                                        || self.particles[new_y][new_x].kind == Kind::Water
                                    {
                                        let self_kind = self.particles[y][x];
                                        self.particles[y][x] = self.particles[new_y][new_x];
                                        self.particles[new_y][new_x] = self_kind;
                                    }
                                }
                            }
                        }
                    }
                    Kind::Gravel => {
                        if (y as u32) < GRID_HEIGHT - 1 {
                            if self.particles[y + 1][x].empty()
                                || self.particles[y + 1][x].kind == Kind::Water
                            {
                                let self_kind = self.particles[y][x];
                                self.particles[y][x] = self.particles[y + 1][x];
                                self.particles[y + 1][x] = self_kind;
                            }
                        }
                    }
                    Kind::Water => {
                        let down_valid = y < GRID_HEIGHT as usize - 1;
                        if down_valid && self.particles[y + 1][x].empty() {
                            self.particles[y + 1][x] = self.particles[y][x];
                            self.particles[y][x] = Particle::default();
                        } else {
                            // TODO: Rename and refactor this
                            let new_y = y + 1;
                            let (x_off, x_check_off) = {
                                let n = rng.gen_range(1..3);
                                let sign = rng.gen::<bool>() as i32 * 2 - 1;
                                (n * sign, (n - 1) * sign)
                            };
                            let new_x1 = x as i32 + x_off;
                            let check_x1 = x as i32 + x_check_off;
                            let new_x1_valid = new_x1 >= 0 && new_x1 < GRID_WIDTH as i32;

                            let x_off = rng.gen::<bool>() as i32 * 2 - 1;
                            let new_x4 = x as i32 - x_off;
                            let new_x4_valid = new_x4 >= 0 && new_x4 < GRID_WIDTH as i32;

                            let (x_off, x_check_off) = {
                                let n = rng.gen_range(2..5);
                                let sign = rng.gen::<bool>() as i32 * 2 - 1;
                                (n * sign, (n - 1) * sign)
                            };
                            let new_x5 = x as i32 + x_off;
                            let check_x5 = x as i32 + x_check_off;
                            let new_x5_valid = new_x5 >= 0 && new_x5 < GRID_WIDTH as i32;
                            if down_valid
                                && new_x1_valid
                                && self.particles[new_y][new_x1 as usize].empty()
                                && self.particles[new_y][check_x1 as usize].kind == Kind::Water
                            {
                                self.particles[new_y][new_x1 as usize] = self.particles[y][x];
                                self.particles[y][x] = Particle::default();
                            } else if new_x4_valid && self.particles[y][new_x4 as usize].empty() {
                                self.particles[y][new_x4 as usize] = self.particles[y][x];
                                self.particles[y][x] = Particle::default();
                            } else if down_valid
                                && new_x5_valid
                                && self.particles[y][new_x5 as usize].empty()
                                && self.particles[new_y][check_x5 as usize].kind == Kind::Water
                            {
                                self.particles[y][new_x5 as usize] = self.particles[y][x];
                                self.particles[y][x] = Particle::default();
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame
            .chunks_exact_mut(4)
            .skip((WIN_WIDTH * (WIN_HEIGHT - GRID_HEIGHT)) as usize)
            .enumerate()
        {
            let x = i % GRID_WIDTH as usize;
            let y = i / GRID_WIDTH as usize;

            let particle = &self.particles[y][x];

            let rgba = if particle.kind != Kind::Empty {
                particle.kind.color()
            } else {
                [0x00, 0x00, 0x00, 0xFF]
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn set_pixel(&mut self, (x, y): (usize, usize), kind: Kind) {
        if x < GRID_WIDTH as usize
            && y < GRID_HEIGHT as usize
            && (kind == Kind::Empty || self.particles[y][x].empty())
        {
            self.particles[y][x] = Particle {
                kind,
                touched: self.clock,
            };
        }
    }
}

const NUM_KEYS: [VirtualKeyCode; 10] = {
    use VirtualKeyCode::*;
    [Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0]
};
const TOOLBAR_KINDS: [Kind; 4] = {
    use Kind::*;
    [Sand, Gravel, Water, Stone]
};

struct Toolbar {}

impl Toolbar {
    fn draw(&self, frame: &mut [u8], selected_kind: Kind) {
        for (i, pixel) in frame
            .chunks_exact_mut(4)
            .take((WIN_WIDTH * TOOLBAR_HEIGHT) as usize)
            .enumerate()
        {
            let x = i % WIN_WIDTH as usize;
            let y = i / WIN_WIDTH as usize;

            let part_size = (WIN_WIDTH / 10) as usize;
            let part_gap = 4;
            let top_gap = 5;
            let which_part = x / part_size;
            let x_in_part = x % part_size;

            let do_color = (y > top_gap && y < TOOLBAR_HEIGHT as usize - top_gap)
                && (x_in_part >= part_gap && x_in_part < part_size - part_gap);

            let mut rgba = [0x00, 0x00, 0x00, 0xFF];
            if which_part < TOOLBAR_KINDS.len() {
                let which_kind = TOOLBAR_KINDS[which_part];
                if which_kind == selected_kind && !do_color {
                    rgba = [0x7f, 0x00, 0x00, 0xFF];
                } else if do_color {
                    rgba = which_kind.color();
                }
            }
            pixel.copy_from_slice(&rgba);
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIN_WIDTH as f64, WIN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Powder simulation test")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIN_WIDTH, WIN_HEIGHT, surface_texture)?
    };
    let mut world = World::new();
    let toolbar = Toolbar {};

    let mut paused = false;
    let mut selected_kind = Kind::Sand;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            toolbar.draw(pixels.get_frame(), selected_kind);
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
            } else if input.key_pressed(VirtualKeyCode::F) {
                paused = true;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            let num_key_pressed_index = NUM_KEYS.iter().position(|&key| input.key_pressed(key));
            if let Some(num_key_pressed_index) = num_key_pressed_index {
                if num_key_pressed_index < TOOLBAR_KINDS.len() {
                    selected_kind = TOOLBAR_KINDS[num_key_pressed_index];
                }
            }

            let left_click = input.mouse_held(0);
            let right_click = input.mouse_held(1);

            if left_click || right_click {
                if input.mouse_pressed(0) {
                    if let Some(Ok((pixel_x, pixel_y))) = input
                        .mouse()
                        .map(|mouse_pos| pixels.window_pos_to_pixel(mouse_pos))
                    {
                        if pixel_y < TOOLBAR_HEIGHT as usize {
                            let which_part = pixel_x / (WIN_WIDTH as usize / 10);
                            if which_part < TOOLBAR_KINDS.len() {
                                selected_kind = TOOLBAR_KINDS[which_part];
                            }
                        }
                    }
                }

                let click_kind = if left_click {
                    selected_kind
                } else {
                    Kind::Empty
                };

                let (mouse_cell, mouse_prev_cell) = input
                    .mouse()
                    .map(|(mx, my)| {
                        let (dx, dy) = input.mouse_diff();
                        let prev_x = mx - dx;
                        let prev_y = my - dy;

                        let (mx_i, my_i) = pixels
                            .window_pos_to_pixel((mx, my))
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        let (px_i, py_i) = pixels
                            .window_pos_to_pixel((prev_x, prev_y))
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        (
                            (mx_i as isize, my_i as isize),
                            (px_i as isize, py_i as isize),
                        )
                    })
                    .unwrap_or_default();

                for pixel_pos in line_drawing::Bresenham::new(mouse_prev_cell, mouse_cell) {
                    let (pixel_x, pixel_y) = (pixel_pos.0 as i32, pixel_pos.1 as i32);
                    for x_off in -1..=1 {
                        for y_off in -1..=1 {
                            world.set_pixel(
                                (
                                    (pixel_x + x_off) as usize,
                                    (pixel_y + y_off - TOOLBAR_HEIGHT as i32) as usize,
                                ),
                                click_kind,
                            );
                        }
                    }
                }
            }

            // Update internal state and request a redraw
            if !paused || input.key_pressed(VirtualKeyCode::F) {
                world.update();
            }

            window.request_redraw();
        }
    });
}
