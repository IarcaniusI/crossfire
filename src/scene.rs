
//actions of scene
//NONE - resume executing of current scene
//EXIT - exit from program
//GAME - start the game
//MAINMENU - remove all scenes before main menu
//PAUSEMENU - add new scene - pause menu
//SETTINGS - add new scene - settings menu
//BACK - back from settings or pause menu to previous scene
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
pub enum SceneAction {
    NONE, EXIT, GAME, MAINMENU, PAUSEMENU, SETTINGS, BACK
}
