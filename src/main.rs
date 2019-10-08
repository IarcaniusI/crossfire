extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use opengl_graphics::{ GlGraphics, OpenGL };
use glutin_window::GlutinWindow as Window;

use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings};
use piston::input::{RenderEvent, RenderArgs, PressEvent, Button};

mod mainmenu;
mod game;
mod scene;
use crate::mainmenu::MainMenu;
use crate::game::Game;
use crate::scene::SceneAction;

//it is possible implement Scene stack using Traits, but number of scenes is small and it is easier
pub enum Scene {
    MAINMENU(MainMenu), SETTINGS, GAME(Game), PAUSEMENU
}

impl Scene {
    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        //render according scene
        match self {
            Scene::GAME(game) => game.render(gl, args),
            Scene::MAINMENU(menu) => menu.render(gl, args),
            _ => ()
        }
    }

    pub fn input(&mut self, button: &Button) -> SceneAction {
        //process input button and obtain code for changing scene
        match self {
            Scene::GAME(game) => game.input(button),
            Scene::MAINMENU(menu) => menu.input(button),
            _ => SceneAction::NONE
        }
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    scenes: Vec<Scene>
}

impl App {
    fn new(gl: GlGraphics) -> App {
        let mut scenes = vec![];//
        let init_scene = Scene::MAINMENU(MainMenu::new());
        scenes.push(init_scene);
        App {
            gl: gl,
            scenes: scenes
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        let work_scene_number = self.scenes.len() - 1;
        self.scenes[work_scene_number].render(&mut self.gl, args);
    }

    fn input(&mut self, button: &Button) {
        let work_scene_number = self.scenes.len() - 1;
        let action = self.scenes[work_scene_number].input(button);

        //add or remove scene to stack according the action
        match action {
            SceneAction::NONE => (),
            SceneAction::EXIT => std::process::exit(0),
            SceneAction::GAME => (),
            SceneAction::MAINMENU => (),
            SceneAction::PAUSEMENU => (),
            SceneAction::SETTINGS => (),
            SceneAction::BACK => { self.scenes.pop(); }
        };

        //if accidentally stack of scenes is empty, then exit
        if self.scenes.len() == 0 {
            std::process::exit(0);
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "CrossFire",
            [640, 480]
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new application and init its
    let mut app = App::new( GlGraphics::new(opengl) );

    //processing of events
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        //rerender window
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        //processing of keyboard events
        if let Some(button) = e.press_args() {
            app.input(&button);
        }

        //unusable
        // if let Some(u) = e.update_args() {
        //     app.update(&u);
        // }
    }
}
