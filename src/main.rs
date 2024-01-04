use std::{
    cell::RefCell, collections::VecDeque, process::exit, rc::Rc, time::Instant,
};

use macroquad::{
    prelude::{is_key_down, Color, KeyCode, GREEN, RED},
    shapes::draw_rectangle,
    window::next_frame,
};

use rand::{self, Rng};

trait Scene {
    fn update(&mut self);
    fn draw(&self, renderer: &Renderer);
}

const GRID_WIDTH: f32 = 20.;
const GRID_HEIGHT: f32 = 20.;
const SCREEN_WIDTH: f32 = 800.;
const SCREEN_HEIGHT: f32 = 800.;
const TICK_SPEED_MS: u128 = 250;

#[derive(PartialEq, Eq)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}
#[derive(PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

struct GameOver {}

impl Scene for GameOver {
    fn update(&mut self) {}

    fn draw(&self, _renderer: &Renderer) {
        todo!()
    }
}

struct Menu {}

impl Scene for Menu {
    fn update(&mut self) {}

    fn draw(&self, _renderer: &Renderer) {
        todo!()
    }
}

struct GameScene {
    direction: Direction,
    bodyparts: VecDeque<Position>,
    last_tick: Instant,
    head_position: Position,
    fruit_location: Position,
}
impl Scene for GameScene {
    fn update(&mut self) {
        self.handle_input();

        if self.last_tick.elapsed().as_millis() >= TICK_SPEED_MS {
            match self.direction {
                Direction::Up => self.head_position.y -= 1,
                Direction::Left => self.head_position.x -= 1,
                Direction::Down => self.head_position.y += 1,
                Direction::Right => self.head_position.x += 1,
            }

            if self.head_position.x < 0
                || self.head_position.x >= GRID_WIDTH as i32
                || self.head_position.y < 0
                || self.head_position.y >= GRID_HEIGHT as i32
            {
                exit(0);
            }

            if self.head_position == self.fruit_location {
                self.fruit_location = Self::new_fruit();
            } else {
                self.bodyparts.pop_front();
            }

            for bp in self.bodyparts.iter() {
                if &self.head_position == bp {
                    exit(0)
                }
            }
            self.bodyparts.push_back(Position {
                x: self.head_position.x,
                y: self.head_position.y,
            });

            self.last_tick = Instant::now();
        }
    }

    fn draw(&self, renderer: &Renderer) {
        for bp in self.bodyparts.iter(){
            renderer.draw_bodypart(bp);
        }

        renderer.draw_fruit(&self.fruit_location);
    }
}

impl GameScene {
    fn new() -> Self {
        let head_x = (GRID_WIDTH / 2_f32) as i32;
        let head_y = (GRID_HEIGHT / 2_f32) as i32;

        let mut bodyparts = VecDeque::new();
        bodyparts.push_back(Position {
            x: head_x,
            y: head_y,
        });

        let head_pos = Position {
            x: head_x,
            y: head_y,
        };

        let fruit_location = Self::new_fruit();

        Self {
            direction: Direction::Up,
            bodyparts,
            last_tick: Instant::now(),
            head_position: head_pos,
            fruit_location,
        }
    }

    fn new_fruit() -> Position {
        let x = rand::thread_rng().gen_range(0..GRID_WIDTH as i32 - 1);
        let y = rand::thread_rng().gen_range(0..GRID_HEIGHT as i32 - 1);

        Position { x, y }
    }

    fn handle_input(&mut self) {
        if is_key_down(KeyCode::W) && self.direction != Direction::Down {
            self.direction = Direction::Up;
        }

        if is_key_down(KeyCode::A) && self.direction != Direction::Right {
            self.direction = Direction::Left;
        }

        if is_key_down(KeyCode::S) && self.direction != Direction::Up {
            self.direction = Direction::Down;
        }

        if is_key_down(KeyCode::D) && self.direction != Direction::Left {
            self.direction = Direction::Right;
        }
    }
}

struct Game {
    renderer: Renderer,
    scenes: Vec<Rc<RefCell<dyn Scene>>>,
    active_scene: Option<Rc<RefCell<dyn Scene>>>,
}

impl Game {
    fn new() -> Self {
        Game {
            renderer: Renderer::new(),
            scenes: Vec::new(),
            active_scene: None,
        }
    }

    fn update(&mut self) {
        let scene = match &self.active_scene {
            Some(scene) => scene.try_borrow_mut(),
            None => panic!("`update` called without an active scene"),
        };

        match scene {
            Ok(mut scene) => scene.update(),
            Err(_) => panic!("Failed to borrow Scene"),
        }
    }

    fn draw(&self) {
        match &self.active_scene{
            Some(s) => s.borrow().draw(&self.renderer), 
            None => panic!("`draw` called without active scene."),
        }
    }

    fn add_scene(&mut self, scene: Rc<RefCell<dyn Scene>>){
        self.scenes.push(scene);
    }

    fn set_scene(&mut self, index: usize){
        self.active_scene = Some(Rc::clone(&self.scenes[index]));
    }
}

struct Renderer {
    cell_width: f32,
    cell_height: f32,
    object_width: f32,
    object_height: f32,
    object_gap_width: f32,
    object_gap_height: f32,
}

impl Renderer {
    fn new() -> Self {
        let cell_width = SCREEN_WIDTH / GRID_WIDTH;
        let cell_height = SCREEN_HEIGHT / GRID_HEIGHT;

        let object_gap_width = cell_width * 0.1;
        let object_gap_height = cell_height * 0.1;

        let body_width = cell_width - object_gap_width;
        let body_height = cell_height - object_gap_height;

        Renderer {
            cell_width,
            cell_height,
            object_width: body_width,
            object_height: body_height,
            object_gap_height,
            object_gap_width,
        }
    }

    fn draw_bodypart(&self, bp: &Position) {
        self.draw_rect_at_point(bp, GREEN);
    }

    fn draw_fruit(&self, f: &Position) {
        self.draw_rect_at_point(f, RED);
    }

    fn draw_rect_at_point(&self, p: &Position, c: Color) {
        let real_x = p.x as f32 * self.cell_width;
        let real_y = p.y as f32 * self.cell_height;

        let real_x = real_x + self.object_gap_width / 2.;
        let real_y = real_y + self.object_gap_height / 2.;

        draw_rectangle(
            real_x,
            real_y,
            self.object_width,
            self.object_height,
            c,
        );
    }
}

fn get_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Snek :þ".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}

#[macroquad::main(get_conf)]
async fn main() {
    let mut game = Game::new();

    let gamescene = GameScene::new(); 
    let gamescene = RefCell::new(gamescene);
    let gamescene = Rc::new(gamescene);

    game.add_scene(gamescene);

    game.set_scene(0);

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}
