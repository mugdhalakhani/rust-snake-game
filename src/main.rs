extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

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
    rows: u32,
    cols: u32,
    just_ate: bool,
    square_size: u32,
    food: Food,
    score: u32,
}

impl Game {
    fn new(gl:GlGraphics, snake: Snake, rows: u32, cols: u32, just_ate: bool, square_size: u32, score: u32) -> Game {
      let mut game = Game {
        gl: gl,
        snake: snake,
        rows: rows,
        cols: cols,
        just_ate: just_ate,
        square_size: square_size,
        food: Food {x:1, y:1},
        score:score,
      };
      game.create_random_food();
      game
    }

    fn create_random_food(&mut self) {
      if self.just_ate {
        // Create a new food randomly on the board such that it doesn't
        // lie on the snake
        use rand::Rng;
        use rand::thread_rng;

        let mut generator = thread_rng();
        loop {
          let new_x = generator.gen_range(0, self.cols);
          let new_y = generator.gen_range(0, self.rows);
          if !self.snake.collides(new_x, new_y) {
            self.food = Food {x: new_x, y: new_y};
            break;
          }
        }
      }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
        });

        self.snake.render(&mut self.gl, args);
        self.food.render(&mut self.gl, args, self.square_size);
    }

    fn update(&mut self) -> bool {
        if !self.snake.update(self.just_ate, self.cols, self.rows) {
          return false;
        }

        if self.just_ate {
          self.score += 1;
          self.just_ate = false;
        }

        self.just_ate = self.food.update(&self.snake);
        self.create_random_food();
        true
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
  body: LinkedList<(u32, u32)>,
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

    fn update(&mut self, just_ate: bool, cols: u32, rows:u32) -> bool {
        let mut head = (*self.body.front().expect("Snake has no head!")).clone();

        match self.direction {
            Direction::Left => 
              if head.0 > 0 {
                head.0 -= 1;
              } else {
                head.0 = cols - 1;
              },
            Direction::Right =>
            if head.0 < cols-1 {
              head.0 += 1;
            } else {
              head.0 = 0
            },
            Direction::Up =>
            if head.1 > 0 {
              head.1 -= 1;
            } else {
              head.1 = rows-1
            },
            Direction::Down =>
            if head.1 < rows-1 {
              head.1 += 1;
            } else {
              head.1 = 0
            },
        }

        // If updated snake collides with itself, fail update.
        if self.collides(head.0, head.1) {
          return false;
        }

        self.body.push_front(head);
        if !just_ate {
          self.body.pop_back().unwrap();
        }
        true
    }

    fn collides(&self, x:u32, y:u32) -> bool {
      self.body.iter().any(|part| x == part.0 && y == part.1)
    }
}

struct Food {
  x: u32,
  y: u32,
}

impl Food {
  // If snake eats food in this update call, return true
  fn update(&mut self, snake: &Snake) -> bool {
    let front = snake.body.front().unwrap();
    if front.0 == self.x && front.1 == self.y {
      true
    } else {
      false
    }
  }

  fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
    const BLACK : [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    let x = self.x * width;
    let y = self.y * width;

    let square = graphics::rectangle::square(x as f64, y as f64, 20_f64);
    gl.draw(args.viewport(), |c,gl| {
      let transform = c.transform;
      graphics::rectangle(BLACK, square, transform, gl)
    });
  }
}

fn main() {
    let opengl = OpenGL::V3_2;

    const COLS: u32 = 20;
    const ROWS: u32 = 20;
    const CELL_SIZE: u32 = 20;
    let width = COLS * CELL_SIZE;
    let height = ROWS * CELL_SIZE;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("snake-game", [width, height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game::new(
        GlGraphics::new(opengl),
        Snake {body: LinkedList::from_iter((vec![(ROWS/2, ROWS/2), (COLS/2+1,COLS/2)]).into_iter()), direction: Direction::Up},
        ROWS,
        COLS,
        /* just_ate= */ false,
        CELL_SIZE,
        0);
    let mut event_settings = EventSettings::new();
    event_settings.ups = 8;
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(_args) = e.update_args() {
            if !game.update() {
                println!("Game Over! score is: {}", game.score);
                break;
            }
        }

        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                game.pressed(&args.button);
            }
        }
    }
}