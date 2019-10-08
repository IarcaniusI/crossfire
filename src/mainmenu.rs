
use opengl_graphics::GlGraphics;
use piston::input::{RenderArgs, Button, Key};
use graphics::{rectangle, clear, Rectangle, draw_state};

use crate::scene::SceneAction;

const WIDTH_CELL_SIZE: f64 = 32.0;
const HEIGHT_CELL_SIZE: f64 = 32.0;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GR: [f32; 4] = [0.0, 1.0, 1.0, 1.0];

#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
enum ButtonType {
    NEWGAME, SETTINGS, ABOUT, EXIT
}

//coordinates and sizes in cells, not in pixels
struct MenuButton {
    x :f64,
    y :f64,
    width :f64,
    height :f64,
    button_type :ButtonType,
    title :String,
    color :[f32; 4]
}

pub struct MainMenu {
    about :bool,
    active_button :usize,
    buttons :Vec<MenuButton>
}

impl MenuButton {
    fn new(x: f64, y: f64, width: f64, height: f64,
            button_type: ButtonType, title: String, color: [f32; 4]) -> MenuButton {
        
        MenuButton {
            x: x, y: y, width: width, height: height,
            button_type: button_type, title: title, color: color
        }
        
    }

    fn get_dims(&self) -> [f64; 4] {
        let mut coordinates :[f64; 4] = [
            self.x * WIDTH_CELL_SIZE,
            self.y * HEIGHT_CELL_SIZE,
            self.width * WIDTH_CELL_SIZE,
            self.height * HEIGHT_CELL_SIZE,
        ];
        coordinates
    }

}

impl MainMenu {
    pub fn new() -> MainMenu {
        let new_game = MenuButton::new(2.0, 2.0, 10.0, 1.0, 
                                        ButtonType::NEWGAME, "New game".to_string(), GREEN);
        let settings = MenuButton::new(2.0, 4.0, 10.0, 1.0, 
                                        ButtonType::NEWGAME, "Settings".to_string(), YELLOW);
        let about = MenuButton::new(2.0, 6.0, 10.0, 1.0, 
                                        ButtonType::NEWGAME, "About".to_string(), BLUE);
        let quit = MenuButton::new(2.0, 8.0, 10.0, 1.0, 
                                        ButtonType::NEWGAME, "Quit".to_string(), RED);

        let mut buttons = vec![new_game, settings, about, quit];
        MainMenu {
            about: false,
            active_button: 0,
            buttons: buttons
        }
    }

    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        let buttons = &self.buttons;
        let active_button = self.active_button;
        let about = self.about;

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            let draw_state = &c.draw_state;
            let draw_state = &draw_state::DrawState::default();

            //let draw_state_default = &draw_state::Default::default();
            clear(BLACK, gl);

            for (button_num, button) in buttons.iter().enumerate() {
                let border_color = if button_num == active_button {
                    GR
                } else {
                    WHITE
                };

                let rect = Rectangle::new_border(border_color, 3.0)
                            .color(button.color);
                rect.draw(button.get_dims(), draw_state, transform, gl)
//                rectangle(button.color, button.get_dims(), transform, gl);
            }
            

        });
    }

    pub fn input(&mut self, button: &Button) -> SceneAction {
        SceneAction::NONE
    }
}
