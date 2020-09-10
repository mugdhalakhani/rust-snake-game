extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent, ButtonEvent};
use piston::window::WindowSettings;
use piston::Button;
use piston::ButtonState;
use piston::Key;

use std::collections::LinkedList;
use std::vec::Vec;
use std::iter::FromIterator;

#[derive(Clone, PartialEq)]
enum Direction {
 Right, Left, Up, Down
}

pub struct Game {
    gl: GlGraphics, // OpenGL drawing backend.
    snake: Snake,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
        });

        self.snake.render(&mut self.gl, args);
    }

    fn update(&mut self) {
        self.snake.update();
    }

    fn pressed(&mut self, button: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match button {
            &Button::Keyboard(Key::Up)
                if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down)
                if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left)
                if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right)
                if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };
    }
}

struct Snake {
  body: LinkedList<(i32, i32)>,
  direction: Direction,
}

impl Snake {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        let squares: Vec<graphics::types::Rectangle> = self.body
        .iter()
        .map(|&(x,y)| -> graphics::types::Rectangle {
            graphics::rectangle::square(
                20.0 * x as f64,
                20.0 * y as f64,
                20_f64)
        })
        .collect();

        const PURPLE: [f32; 4] = [0.5, 0.0, 0.5, 1.0];
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares.into_iter()
            .for_each(|square| graphics::rectangle(PURPLE, square, transform, gl));
        });
    }

    fn update(&mut self) {
        let mut new_head = (*self.body.front().expect("Snake has no body")).clone();
        match self.direction {
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
        }
        self.body.push_front(new_head);
        self.body.pop_back().unwrap();
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spnake-game", [400, 400])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        snake: Snake {body: LinkedList::from_iter(vec![(9,9), (9,10)]), direction: Direction::Up},
    };
    let mut event_settings = EventSettings::new();
    event_settings.ups = 8;
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(_args) = e.update_args() {
            game.update();
        }

        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                game.pressed(&args.button);
            }
        }
    }
}