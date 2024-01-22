#![windows_subsystem = "windows"]
use core::panic;
use std::{
    cell::RefCell, collections::VecDeque, process::exit, rc::Rc, time::Instant,
};

use macroquad::{
    prelude::{
        is_key_down, is_mouse_button_down, mouse_position, Color, KeyCode,
        GREEN, RED, WHITE,
    },
    shapes::draw_rectangle,
    text::draw_text,
    window::next_frame,
};

use rand::{self, Rng};

trait Scene {
    fn update(&mut self) -> Option<SwapScene>;
    fn draw(&self, renderer: &Renderer);
    fn reset(&mut self);
}

const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;
const SCREEN_WIDTH: f32 = 800.;
const SCREEN_HEIGHT: f32 = 800.;
const TICK_SPEED_MS: u128 = 250;

#[derive(PartialEq, Eq, Clone)]
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

enum SwapScene {
    _StartMenu,
    Game,
    GameOver,
}

struct Button {
    pos: Position,
    width: i32,
    height: i32,
    label: String,
    on_click: fn() -> Option<SwapScene>,
}

impl Button {
    #[allow(clippy::cast_possible_truncation)]
    fn is_mouse_over_button(&self) -> bool {
        let (mx, my) = mouse_position();

        (mx as i32) > self.pos.x
            && (mx as i32) < self.pos.x + self.width
            && (my as i32) > self.pos.y
            && (my as i32) < self.pos.y + self.height
    }
}

struct GameOver {
    restart_button: Button,
    exit_button: Button,
}

impl GameOver {
    fn new() -> Self {
        let restart_button = Button {
            pos: Position { x: 250, y: 100 },
            width: 300,
            height: 100,
            label: "Restart Game".to_owned(),
            on_click: || Some(SwapScene::Game),
        };

        let exit_button = Button {
            pos: Position { x: 250, y: 300 },
            width: 300,
            height: 100,
            label: "Exit Game".to_owned(),
            on_click: || (exit(0)),
        };
        Self {
            restart_button,
            exit_button,
        }
    }
}

impl Scene for GameOver {
    fn update(&mut self) -> Option<SwapScene> {
        let active_button: Option<&Button> =
            if self.restart_button.is_mouse_over_button() {
                Some(&self.restart_button)
            } else if self.exit_button.is_mouse_over_button() {
                Some(&self.exit_button)
            } else {
                None
            };

        if is_mouse_button_down(macroquad::prelude::MouseButton::Left) {
            active_button.and_then(|b| (b.on_click)())
        } else {
            None
        }
    }

    fn draw(&self, _renderer: &Renderer) {
        Renderer::draw_button(&self.restart_button);
        Renderer::draw_button(&self.exit_button);
    }

    fn reset(&mut self) {}
}

struct Menu {
    start_button: Button,
    exit_button: Button,
}

impl Menu {
    fn new() -> Self {
        let start_button = Button {
            pos: Position { x: 250, y: 100 },
            width: 300,
            height: 100,
            label: "Start Game".to_owned(),
            on_click: || Some(SwapScene::Game),
        };

        let exit_button = Button {
            pos: Position { x: 250, y: 300 },
            width: 300,
            height: 100,
            label: "Exit Game".to_owned(),
            on_click: || exit(0),
        };
        Self {
            start_button,
            exit_button,
        }
    }
}

impl Scene for Menu {
    fn update(&mut self) -> Option<SwapScene> {
        let mut active_button: Option<&Button> = None;

        if self.start_button.is_mouse_over_button() {
            active_button = Some(&self.start_button);
        } else if self.exit_button.is_mouse_over_button() {
            active_button = Some(&self.exit_button);
        }

        if is_mouse_button_down(macroquad::prelude::MouseButton::Left) {
            active_button.and_then(|b| (b.on_click)())
        } else {
            None
        }
    }

    fn draw(&self, _renderer: &Renderer) {
        Renderer::draw_button(&self.start_button);
        Renderer::draw_button(&self.exit_button);
    }
    fn reset(&mut self) {}
}

struct GameScene {
    direction: Direction,
    bodyparts: VecDeque<Position>,
    last_tick: Instant,
    head_position: Position,
    fruit_location: Position,
    next_direction: Direction,
}
impl Scene for GameScene {
    fn update(&mut self) -> Option<SwapScene> {
        self.handle_input();
        
        if self.last_tick.elapsed().as_millis() >= TICK_SPEED_MS {

            self.direction = self.next_direction.clone(); 

            match self.direction {
                Direction::Up => self.head_position.y -= 1,
                Direction::Left => self.head_position.x -= 1,
                Direction::Down => self.head_position.y += 1,
                Direction::Right => self.head_position.x += 1,
            }

            if self.head_position.x < 0
                || self.head_position.x >= GRID_WIDTH
                || self.head_position.y < 0
                || self.head_position.y >= GRID_HEIGHT
            {
                return Some(SwapScene::GameOver);
            }

            if self.head_position == self.fruit_location {
                self.fruit_location = Self::new_fruit();
                while self.bodyparts.contains(&self.fruit_location) {
                    self.fruit_location = Self::new_fruit();
                }
            } else {
                self.bodyparts.pop_front();
            }

            for bp in &self.bodyparts {
                if &self.head_position == bp {
                    return Some(SwapScene::GameOver);
                }
            }
            self.bodyparts.push_back(Position {
                x: self.head_position.x,
                y: self.head_position.y,
            });


            self.last_tick = Instant::now();
        }
        None
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.draw_head(&self.head_position);
        for bp in self.bodyparts.range(..self.bodyparts.len()-1) {
            renderer.draw_bodypart(bp);
        }

        renderer.draw_fruit(&self.fruit_location);
    }
    fn reset(&mut self) {
        let head_x = GRID_WIDTH / 2;
        let head_y = GRID_HEIGHT / 2;

        self.bodyparts = VecDeque::new();
        self.bodyparts.push_back(Position {
            x: head_x,
            y: head_y,
        });

        self.head_position = Position {
            x: head_x,
            y: head_y,
        };

        self.fruit_location = Self::new_fruit();
        while self.bodyparts.contains(&self.fruit_location) {
            self.fruit_location = Self::new_fruit();
        }
        self.direction = Direction::Up;
        self.next_direction = Direction::Up;
    }
}

impl GameScene {
    fn new() -> Self {
        let head_x = GRID_WIDTH / 2;
        let head_y = GRID_HEIGHT / 2;

        let mut bodyparts = VecDeque::new();
        bodyparts.push_back(Position {
            x: head_x,
            y: head_y,
        });

        let head_pos = Position {
            x: head_x,
            y: head_y,
        };

        let mut fruit_location = Self::new_fruit();
        while bodyparts.contains(&fruit_location) {
            fruit_location = Self::new_fruit();
        }

        Self {
            direction: Direction::Up,
            bodyparts,
            last_tick: Instant::now(),
            head_position: head_pos,
            fruit_location,
            next_direction: Direction::Up, 
        }
    }

    fn new_fruit() -> Position {
        let x = rand::thread_rng().gen_range(0..GRID_WIDTH - 1);
        let y = rand::thread_rng().gen_range(0..GRID_HEIGHT - 1);

        Position { x, y }
    }

    fn handle_input(&mut self) {
        if is_key_down(KeyCode::W) && self.direction != Direction::Down {
            self.next_direction = Direction::Up;
        }

        if is_key_down(KeyCode::A) && self.direction != Direction::Right {
            self.next_direction = Direction::Left;
        }

        if is_key_down(KeyCode::S) && self.direction != Direction::Up {
            self.next_direction = Direction::Down;
        }

        if is_key_down(KeyCode::D) && self.direction != Direction::Left {
            self.next_direction = Direction::Right;
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
        Self {
            renderer: Renderer::new(),
            scenes: Vec::new(),
            active_scene: None,
        }
    }

    fn update(&mut self) {
        let swap = self.active_scene.as_mut().map_or_else(
            || panic!("Update called without active scene"),
            |scene| {
                scene.try_borrow_mut().map_or_else(
                    |_| panic!("Fatal Error: Failed to borrow scene"),
                    |mut scene| scene.update(),
                )
            },
        );

        if let Some(s) = swap {
            match s {
                SwapScene::_StartMenu => self.set_scene(0),
                SwapScene::Game => {
                    self.set_scene(1);
                    self.active_scene.as_mut().map_or_else(
                        || panic!("Unreachable"),
                        |scene| {
                            scene.try_borrow_mut().map_or_else(
                                |_| panic!("Failed to borrow mut"),
                                |mut scene| scene.reset(),
                            );
                        },
                    );
                }
                SwapScene::GameOver => self.set_scene(2),
            }
        }
    }

    fn draw(&self) {
        match &self.active_scene {
            Some(s) => s.borrow().draw(&self.renderer),
            None => panic!("`draw` called without active scene."),
        }
    }

    fn add_scene(&mut self, scene: Rc<RefCell<dyn Scene>>) {
        self.scenes.push(scene);
    }

    fn set_scene(&mut self, index: usize) {
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
    #[allow(clippy::cast_precision_loss)]
    fn new() -> Self {
        let cell_width = SCREEN_WIDTH / GRID_WIDTH as f32;
        let cell_height = SCREEN_HEIGHT / GRID_HEIGHT as f32;

        let object_gap_width = cell_width * 0.1;
        let object_gap_height = cell_height * 0.1;

        let body_width = cell_width - object_gap_width;
        let body_height = cell_height - object_gap_height;

        Self {
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

    fn draw_head(&self, head: &Position) {
        self.draw_rect_at_point(head, Color { r: 0.8, g: 1., b: 0.8, a: 1.})
    }

    fn draw_fruit(&self, f: &Position) {
        self.draw_rect_at_point(f, RED);
    }

    #[allow(clippy::cast_precision_loss)]
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

    #[allow(clippy::cast_precision_loss)]
    fn draw_button(but: &Button) {
        draw_rectangle(
            but.pos.x as f32,
            but.pos.y as f32,
            but.width as f32,
            but.height as f32,
            WHITE,
        );

        draw_text(
            &but.label,
            but.pos.x as f32,
            (but.pos.y + but.height / 2 + 12) as f32,
            50f32,
            GREEN,
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

    let mainmenu = Rc::new(RefCell::new(Menu::new()));

    let gamescene = Rc::new(RefCell::new(GameScene::new()));

    let game_over = Rc::new(RefCell::new(GameOver::new()));

    game.add_scene(mainmenu);

    game.add_scene(gamescene);

    game.add_scene(game_over);

    game.set_scene(0);

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
