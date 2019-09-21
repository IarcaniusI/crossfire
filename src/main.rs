extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

//use piston::window::WindowSettings;
use piston_window::*;
//use piston::event_loop::*;
//use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use std::collections::BTreeSet;

const WIDTH_CELL_SIZE: f64 = 32.0;
const HEIGHT_CELL_SIZE: f64 = 32.0;

const HEIGHT_HUD_SEGMENT_SIZE: f64 = 8.0;
const WIDTH_HUD_SEGMENT_SIZE: f64 = 8.0;

//state of enemies with different behaviours in each of them
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
enum EnimyState {
    Manual, HideLeft, HideUp,
    ToWaitLeft, ToWaitUp, WaitLeft, WaitUp,
    ToHideLeft, ToHideUp,
    Attack
}

//directions for objects - robots, blocks and bullets
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
enum Direct {
    NONE, UP, DOWN, LEFT, RIGHT
}

//possible arguments of hitTest function
#[derive(Debug)]
#[derive(Clone, Copy)]
enum HitTestType {
    FULL, INNER, UP, DOWN, LEFT, RIGHT
}

//type of blocks on landscape:
//NODE - hero can pass and stand, enimies and bullets can fly over it
//HOLE - hero can't pass, but enimies and bullets can fly over it
//WALL - hero can't pass, enimies and bullets can't fly throw it
//SLIDE - hero can move, but can't stand, enimies and bullets can fly over it
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
enum BlockType {
    NODE, HOLE, WALL, SLIDE
}

//geometrical properties of robots, bullets and blocks
struct GameObject {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

//enemies and hero are robots
struct Robot {
    object: GameObject,
    start_x: f64,
    start_y: f64,
    direct: Direct,
    next_direct: Direct,
    speed: f64,
    lives: i64,
    bullet_speed :f64,
    max_bullets: usize,
    bullets: Vec<Bullet>,
    prepare_fire: Direct,
    action_state: EnimyState
}

struct Bullet {
    object: GameObject,
    direct: Direct
}

struct Block {
    object: GameObject,
    block_type: BlockType
}

struct Game {
    hero :Robot,
    blocks :Vec<Block>,
    enimies :Vec<Robot>,
    free_bullets :Vec<Bullet>,//bullets of died robots
    point_num :i64, //count of killed enemies
    crash_num :i64, //count of crashed enemies
    paused: bool,
    game_over: bool, //any reason, win or fail
    game_win: bool //game over and win
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    game :Game
}

impl GameObject {
    //check collision or touching of two objects
    fn rectangle_hit_test(&self, check_obj: &GameObject,
                            hit_type: HitTestType) -> bool {

        let hit_left = match hit_type {
            HitTestType::FULL | HitTestType::LEFT =>
                                    self.x + self.width >= check_obj.x,
            _ => self.x + self.width > check_obj.x
        };
        let hit_right = match hit_type {
            HitTestType::FULL | HitTestType::RIGHT =>
                                    self.x <= check_obj.x + check_obj.width,
            _ => self.x < check_obj.x + check_obj.width
        };
        let hit_up = match hit_type {
            HitTestType::FULL | HitTestType::UP =>
                                    self.y + self.height >= check_obj.y,
            _ => self.y + self.height > check_obj.y
        };
        let hit_down = match hit_type {
            HitTestType::FULL | HitTestType::DOWN =>
                                    self.y <= check_obj.y + check_obj.height,
            _ => self.y < check_obj.y + check_obj.height
        };

        (hit_left && hit_right && hit_up && hit_down)
    }
}

impl Robot {
    fn new(x: f64, y :f64, width: f64, height :f64,
            lives: i64, action_state: EnimyState) -> Robot {

        let obj = GameObject {
                                x: x, y: y,
                                height: height,
                                width: width
                            };

        Robot {
                object: obj, lives: lives,
                start_x: x, start_y: y, speed: 2.0, bullet_speed: 4.0,
                max_bullets: 1, bullets: vec![], prepare_fire: Direct::NONE,
                direct: Direct::NONE, next_direct: Direct::NONE,
                action_state: action_state
        }
    }
}

impl Block {
    fn new(x: f64, y :f64, width: f64, height :f64,
            block_type: BlockType) -> Block {
        let obj = GameObject {
                                x: x, y: y,
                                width: width, height: height
                            };
        Block { object: obj, block_type: block_type}
    }
}

impl Bullet {
    fn new (x: f64, y :f64, width: f64, height :f64,
            direct :Direct) -> Bullet {
        let obj = GameObject {
                                x: x, y: y,
                                width: width, height: height
                            };
        Bullet { object: obj, direct: direct}
    }
}

impl Game {
    fn new() -> Game {
        let hero = Robot::new(0.0, 0.0, WIDTH_CELL_SIZE, WIDTH_CELL_SIZE, 1,
                                EnimyState::Manual);
        let blocks = vec![];
        let enimies = vec![];
        let free_bullets = vec![];

        Game {
                hero: hero, blocks: blocks, enimies: enimies,
                free_bullets: free_bullets,
                point_num: 0, crash_num: 0,
                paused: false, game_over: false, game_win: false
            }
    }

    fn create_level(&mut self) {
        self.paused = false;
        self.game_over = false;
        self.game_win = false;
        self.crash_num = 0;
        self.point_num = 0;

        //clear object vectors
        self.blocks.clear();
        self.enimies.clear();
        self.free_bullets.clear();

        //init hero
        let hero_x = 9.0*WIDTH_CELL_SIZE;
        let hero_y = 11.0*HEIGHT_CELL_SIZE;
        let hero = Robot::new(hero_x, hero_y, WIDTH_CELL_SIZE,
                                    WIDTH_CELL_SIZE, 3, EnimyState::Manual);
        self.hero = hero;

        //init blocks
        for x_cell in 0..17 {
            let x :f64 = (x_cell as f64)*WIDTH_CELL_SIZE;

            for y_cell in 0..14 {
                let y :f64 = (y_cell as f64)*HEIGHT_CELL_SIZE;

                let mut block_type = match x_cell%2 {
                    0 => match y_cell%2 {
                        0 => BlockType::WALL,
                        _ => BlockType::HOLE,
                    },
                    _ => BlockType::HOLE
                };

                block_type = if (x_cell >= 3) && (y_cell >= 3) &&
                    (x_cell <= 13) && (y_cell <= 11) {
                    match block_type {
                        BlockType::HOLE =>
                                if (x_cell%2 == 0)||(y_cell%2 == 0) {
                                    BlockType::SLIDE
                                } else {
                                    BlockType::NODE
                                },
                        _ => block_type
                    }
                } else {
                    block_type
                };

                block_type = match x_cell {
                    0 | 16 => BlockType::WALL,
                    _ => block_type
                };

                block_type = match y_cell {
                    0 | 13 => BlockType::WALL,
                    _ => block_type
                };

                let block = Block::new(x, y,
                        WIDTH_CELL_SIZE, HEIGHT_CELL_SIZE,
                        block_type);
                self.blocks.push(block);
            }
        }

        //init enemies over than field
        let y_cell: f64 = 1.0;
        let y :f64 = y_cell*HEIGHT_CELL_SIZE;
        for i in 0..6 {
            let x_cell = (i*2+4) as f64;
            let x :f64 = x_cell*WIDTH_CELL_SIZE;
            let enimy = Robot::new(x, y, WIDTH_CELL_SIZE, HEIGHT_CELL_SIZE,
                                    3, EnimyState::HideLeft);

            self.enimies.push(enimy);
        }

        //init enimies lefter than field
        let x_cell: f64 = 1.0;
        let x :f64 = x_cell*WIDTH_CELL_SIZE;
        for i in 0..3 {
            let y_cell = (i*4+4) as f64;
            let y :f64 = y_cell*HEIGHT_CELL_SIZE;
            let enimy = Robot::new(x, y, WIDTH_CELL_SIZE, HEIGHT_CELL_SIZE,
                                    3, EnimyState::HideUp);
            self.enimies.push(enimy);
        }

        //init enimies righter than field
        let x_cell: f64 = 15.0;
        let x :f64 = x_cell*WIDTH_CELL_SIZE;
        for i in 0..2 {
            let y_cell = (i*4+6) as f64;
            let y :f64 = y_cell*HEIGHT_CELL_SIZE;
            let enimy = Robot::new(x, y, WIDTH_CELL_SIZE, HEIGHT_CELL_SIZE,
                                    3, EnimyState::HideUp);
            self.enimies.push(enimy);
        }
    }

    fn move_robots(&mut self) {

        let robot_move = |robot:&mut Robot, blocks: &Vec<Block>, hero: bool| {

            //------------------------move logic-----------
            let next_direct_hit_side = match robot.next_direct {
                Direct::LEFT => HitTestType::LEFT,
                Direct::RIGHT => HitTestType::RIGHT,
                Direct::UP => HitTestType::UP,
                Direct::DOWN => HitTestType::DOWN,
                Direct::NONE => HitTestType::INNER,
                //_ => panic!("Incorrect direction for robot")
            };

            let direct_hit_side = match robot.direct {
                Direct::LEFT => HitTestType::LEFT,
                Direct::RIGHT => HitTestType::RIGHT,
                Direct::UP => HitTestType::UP,
                Direct::DOWN => HitTestType::DOWN,
                Direct::NONE => HitTestType::INNER,
                //_ => panic!("Incorrect direction for robot")
            };

            //check if cell slide
            let mut slided = false;
            for block in blocks {
                let intersect = block.object.rectangle_hit_test(&robot.object,
                                                            HitTestType::INNER);
                let slide_block = match block.block_type {
                    BlockType::SLIDE => true,
                    _ => false
                };

                if slide_block && intersect {
                    slided = true;
                    break;
                }
            }
            if !hero {
                slided = false;
            }

            let opposite_direct = match robot.direct {
                Direct::LEFT => Direct::RIGHT,
                Direct::RIGHT => Direct::LEFT,
                Direct::UP => Direct::DOWN,
                Direct::DOWN => Direct::UP,
                _ => Direct::NONE
            };

            //if can change direction, check if direct passable
            //let mut stoped = false;
            let mut blocked_next_direct = false;
            let mut blocked_direct = false;
            for block in blocks {

                let next_intersect =
                    block.object.rectangle_hit_test(&robot.object,
                                                        next_direct_hit_side);
                let current_intersect = block.object.rectangle_hit_test(
                                        &robot.object, direct_hit_side);

                let passable = match block.block_type {
                    BlockType::NODE => true,
                    BlockType::SLIDE => true,
                    BlockType::HOLE => !hero,
                    _ => false
                };

                if !passable {
                    if next_intersect {
                        blocked_next_direct = true;
                    }
                    if current_intersect {
                        blocked_direct = true;
                    }
                }

            }

            let mut new_direct :Direct = robot.direct;
            let mut new_next_direct :Direct = robot.next_direct;
            if !slided || (robot.next_direct == opposite_direct) {
                new_direct = new_next_direct;
                if blocked_next_direct {
                    new_direct = Direct::NONE;
                    new_next_direct = Direct::NONE;
                }
            }

            if slided && blocked_direct {
                new_direct = opposite_direct;
                new_next_direct = Direct::NONE;
            }

            //access new changing
            let speed = robot.speed;
            robot.direct = new_direct;
            robot.next_direct = new_next_direct;
            match new_direct {
                Direct::LEFT => robot.object.x -= speed,
                Direct::RIGHT => robot.object.x += speed,
                Direct::UP => robot.object.y -= speed,
                Direct::DOWN => robot.object.y += speed,
                Direct::NONE => {},
                //_ => panic!("Invalid direction for hero", )
            }
        };

        let blocks = &self.blocks;
        robot_move(&mut self.hero, blocks, true);
        for enimy in &mut self.enimies {
            robot_move(enimy, blocks, false);
        }

    }

    fn create_bullets(&mut self) {

        let create_bullet = |robot:&mut Robot, blocks: &Vec<Block>| {

            let check_rate = robot.bullets.len() < robot.max_bullets;
            let check_command = match robot.prepare_fire {
                Direct::NONE => false,
                _ => true
            };

            let hit_side = match robot.prepare_fire {
                Direct::LEFT => HitTestType::LEFT,
                Direct::RIGHT => HitTestType::RIGHT,
                Direct::UP => HitTestType::UP,
                Direct::DOWN => HitTestType::DOWN,
                Direct::NONE => HitTestType::INNER,
                //_ => panic!("Incorrect direction for robot")
            };

            let mut check_shootable = true;
            for block in blocks {
                let intersect = block.object.rectangle_hit_test(&robot.object,
                                                            hit_side);
                let shootable = match block.block_type {
                    BlockType::WALL => false,
                    _ => true
                };
                if !shootable && intersect {
                    check_shootable = false;
                }
            }

            if check_rate && check_command && check_shootable {

                //let bullet = Bullet::new();
                let bullet_width :f64 = 8.0;
                let bullet_height :f64 = 8.0;

                let bullet_x = match robot.prepare_fire {
                    Direct::LEFT => robot.object.x - bullet_width,
                    Direct::RIGHT =>
                        robot.object.x + robot.object.width + bullet_width,
                    Direct::UP =>
                        robot.object.x + robot.object.width/2.0
                                                        - bullet_width/2.0,
                    Direct::DOWN =>
                        robot.object.x + robot.object.width/2.0
                                                        - bullet_width/2.0,
                    _ => panic!("Invalid value for preparing shoot")
                };

                let bullet_y = match robot.prepare_fire {
                    Direct::LEFT =>
                        robot.object.y + robot.object.height/2.0
                                                        - bullet_height/2.0,
                    Direct::RIGHT =>
                        robot.object.y + robot.object.height/2.0
                                                        - bullet_height/2.0,
                    Direct::UP => robot.object.y - bullet_height,
                    Direct::DOWN =>
                        robot.object.y + robot.object.height + bullet_height,
                    _ => panic!("Invalid value for preparing shoot")
                };

                let bullet = Bullet::new(bullet_x, bullet_y,
                                        bullet_width, bullet_height,
                                        robot.prepare_fire);
                robot.bullets.push(bullet);

                robot.prepare_fire = Direct::NONE;
            }

        };

        let blocks = &self.blocks;
        create_bullet(&mut self.hero, blocks);

        for enimy in &mut self.enimies {
            create_bullet(enimy, blocks);
        }

    }

    fn move_bullets(&mut self) {

        let move_bullet = |bullets: &mut Vec<Bullet>, bullet_speed: f64| {
            for bullet in bullets {
                let add_x = match bullet.direct {
                    Direct::LEFT => -bullet_speed,
                    Direct::RIGHT => bullet_speed,
                    _ => 0.0
                };

                let add_y = match bullet.direct {
                    Direct::DOWN => bullet_speed,
                    Direct::UP => -bullet_speed,
                    _ => 0.0
                };

                bullet.object.x += add_x;
                bullet.object.y += add_y;
            }
        };

        move_bullet(&mut self.hero.bullets, self.hero.bullet_speed);
        for enimy in &mut self.enimies {
            move_bullet(&mut enimy.bullets, enimy.bullet_speed);
        }
        move_bullet(&mut self.free_bullets, self.hero.bullet_speed);

    }

    fn collision_bullets(&mut self) {

        let field_out = |bullets :&Vec<Bullet>| -> Vec<usize> {
            //check bullets that out of range
            let mut fire_bullets = vec![];
            for (bullet_num, bullet) in bullets.iter().enumerate() {

                let check_left = bullet.object.x < 0.0;
                let check_right = bullet.object.x > WIDTH_CELL_SIZE*17.0;
                let check_top = bullet.object.y < 0.0;
                let check_bottom = bullet.object.y > HEIGHT_CELL_SIZE*14.0;
                if check_left || check_right || check_top || check_bottom {
                    fire_bullets.push(bullet_num)
                }
            }
            fire_bullets
        };
        let robot_collision =
                    |bullets :&Vec<Bullet>, goal_robot: &Robot| -> Vec<usize> {

            let mut fire_bullets = vec![];
            for (bullet_num, bullet) in bullets.iter().enumerate() {
                if goal_robot.object.rectangle_hit_test(&bullet.object,
                                                    HitTestType::INNER) {
                    fire_bullets.push(bullet_num);
                    break;
                }
            }
            fire_bullets
        };
        let block_collision =
                    |bullets :&Vec<Bullet>, blocks: &Vec<Block>| -> Vec<usize> {

            let mut fire_bullets = vec![];
            for (bullet_num, bullet) in bullets.iter().enumerate() {
                for block in blocks {

                    let passable = match block.block_type {
                        BlockType::WALL => false,
                        _ => true
                    };

                    if !passable &&
                        block.object.rectangle_hit_test(&bullet.object,
                                                            HitTestType::INNER) {
                        fire_bullets.push(bullet_num);
                        break;
                    }

                }
            }
            fire_bullets
        };

        //check hero bullets
        let mut fire_hero_bullets :BTreeSet<usize> = BTreeSet::new();
        let mut fire_enemies :BTreeSet<usize> = BTreeSet::new();
        let mut hero_die = false;

        for bullet_num in field_out(&self.hero.bullets) {
            fire_hero_bullets.insert(bullet_num);
        }
        for bullet_num in block_collision(&self.hero.bullets, &self.blocks) {
            fire_hero_bullets.insert(bullet_num);
        }

        for (enimy_num, enimy) in self.enimies.iter().enumerate() {
            let enimy_fire_bullets = robot_collision(&self.hero.bullets, enimy);
            if enimy_fire_bullets.len() > 0 {
                for bullet_num in enimy_fire_bullets {
                    fire_hero_bullets.insert(bullet_num);
                    self.point_num += 1;
                }
                fire_enemies.insert(enimy_num);
            }
        }
        //self fired
        let self_fire_bullets = robot_collision(&self.hero.bullets,
                                                                &self.hero);
        if self_fire_bullets.len() > 0 {
            for bullet_num in self_fire_bullets {
                fire_hero_bullets.insert(bullet_num);
            }
            hero_die = true;
        }

        //remove hero bullets
        let hero_bullets = &mut self.hero.bullets;
        let mut counter = 0;
        for bullet_num in fire_hero_bullets {
             //hero_bullets.remove(bullet_num);
             hero_bullets.remove(bullet_num-counter);
             counter += 1;
        }

        //check free bullets
        let mut fire_free_bullets :BTreeSet<usize> = BTreeSet::new();
        for bullet_num in field_out(&self.free_bullets) {
            fire_free_bullets.insert(bullet_num);
        }
        for bullet_num in block_collision(&self.free_bullets, &self.blocks) {
            fire_free_bullets.insert(bullet_num);
        }

        //hero fired
        let hero_fire_bullets = robot_collision(&self.free_bullets,
                                                                &self.hero);
        if hero_fire_bullets.len() > 0 {
            for bullet_num in hero_fire_bullets {
                fire_free_bullets.insert(bullet_num);
            }
            hero_die = true;
        }

        //remove free bullets
        let free_bullets = &mut self.free_bullets;
        let mut counter = 0;
        for bullet_num in fire_free_bullets {
             //hero_bullets.remove(bullet_num);
             free_bullets.remove(bullet_num-counter);
             counter += 1;
        }

        let mut fire_enimy_bullets :Vec<BTreeSet<usize>> = vec![];
        //check every enimy bullets
        for (shooter_num, shooter_enemy) in self.enimies.iter().enumerate() {

            fire_enimy_bullets.push(BTreeSet::new());
            let enimy_bullet_set = &mut fire_enimy_bullets[shooter_num];

            for bullet_num in field_out(&shooter_enemy.bullets) {
                enimy_bullet_set.insert(bullet_num);
            }

            for bullet_num in block_collision(&shooter_enemy.bullets,
                                                            &self.blocks) {
                enimy_bullet_set.insert(bullet_num);
            }

            // for (enimy_num, enimy) in self.enimies.iter().enumerate() {
            //     let local_enimy_fire_bullets =
            //                         robot_collision(shooter_enemy, enimy);
            //     if local_enimy_fire_bullets.len() > 0 {
            //         for bullet_num in local_enimy_fire_bullets {
            //             enimy_bullet_set.insert(bullet_num);
            //         }
            //         fire_enemies.insert(enimy_num);
            //     }
            // }

            //hero fired
            let local_enimy_fire_bullets =
                        robot_collision(&shooter_enemy.bullets, &self.hero);
            if local_enimy_fire_bullets.len() > 0 {
                for bullet_num in local_enimy_fire_bullets {
                    enimy_bullet_set.insert(bullet_num);
                }
                hero_die = true;
            }

        }

        //remove enimy bullets
        let mut shooter_num = 0;
        //println!("START REMOVE bullets IN collision_bullets");
        for shooter_fired_bullets in &fire_enimy_bullets {
            let enimy_bullets = &mut self.enimies[shooter_num].bullets;
            let mut counter = 0;
            for bullet_num in shooter_fired_bullets {
                 //hero_bullets.remove(bullet_num);
                 enimy_bullets.remove(bullet_num-counter);
                 counter += 1;
            }
            shooter_num += 1;
        }
        //println!("END REMOVE bullets IN collision_bullets");

        //remove enemies
        //let enimies = &mut ;
        let mut counter = 0;
        //println!("START REMOVE enimies IN collision_bullets");
        for enimy_num in fire_enemies {
            let index = enimy_num-counter;
            self.free_bullets.append(&mut self.enimies[index].bullets);
            self.enimies.remove(index);
            counter += 1;
        }
        //println!("END REMOVE enimies IN collision_bullets");

        //remove hero
        if hero_die {
            if self.hero.lives > 0 {
                self.hero.lives -= 1;
                self.hero.object.x = self.hero.start_x;
                self.hero.object.y = self.hero.start_y;
            }
        }

    }

    fn collision_robots(&mut self) {

        let mut hero_die = false;
        let mut enimies_die :BTreeSet<usize> = BTreeSet::new();

        for (enimy_goal_num, enimy_goal) in self.enimies.iter().enumerate() {
            for (enimy_num, enimy) in self.enimies.iter().enumerate() {

                if enimy_goal_num == enimy_num {
                    continue;
                }

                if enimy_goal.object.rectangle_hit_test(&enimy.object,
                                                    HitTestType::INNER) {
                    enimies_die.insert(enimy_goal_num);
                    self.crash_num += 1;
                }
            }

            if enimy_goal.object.rectangle_hit_test(&self.hero.object,
                                                        HitTestType::INNER) {
                enimies_die.insert(enimy_goal_num);
                self.crash_num += 1;
            }

        }

        for enimy in &self.enimies {
            if self.hero.object.rectangle_hit_test(&enimy.object,
                                                HitTestType::INNER) {
                hero_die = true;
                break;
            }
        }

        //remove enemies
        let mut counter = 0;
        for enimy_num in enimies_die {
            let index = enimy_num-counter;
            self.free_bullets.append(&mut self.enimies[index].bullets);
            self.enimies.remove(enimy_num-counter);
            counter += 1;
        }

        //remove hero
        if hero_die {
            if self.hero.lives > 0 {
                self.hero.lives -= 1;
            }
        }

    }

    fn logic(&mut self) {
        self.move_robots();
        self.create_bullets();
        self.move_bullets();
        self.collision_bullets();
        self.collision_robots();
    }

    fn enimies_decision(&mut self) {
        use	rand::Rng;

        for enimy in &mut self.enimies {
            //enimy.next_direct = Direct::NONE;

            let mut new_next_direct = enimy.next_direct;
            let mut new_state = enimy.action_state;

            match enimy.action_state {
                EnimyState::HideLeft => {
                    let decision = rand::thread_rng().gen_range(0, 500) <= 0;
                    if decision {
                        new_next_direct = Direct::LEFT;
                        new_state = EnimyState::ToWaitLeft;
                    } else {
                        new_next_direct = Direct::NONE;
                    }
                },
                EnimyState::HideUp => {
                    let decision = rand::thread_rng().gen_range(0, 500) <= 0;
                    if decision {
                        new_next_direct = Direct::UP;
                        new_state = EnimyState::ToWaitUp;
                    } else {
                        new_next_direct = Direct::NONE;
                    }
                },
                EnimyState::ToWaitLeft => {
                    if enimy.object.x == enimy.start_x - WIDTH_CELL_SIZE {
                        new_next_direct = Direct::NONE;
                        new_state = EnimyState::WaitLeft;
                    }
                },
                EnimyState::ToWaitUp => {
                    if enimy.object.y == enimy.start_y - HEIGHT_CELL_SIZE {
                        new_next_direct = Direct::NONE;
                        new_state = EnimyState::WaitUp;
                    }
                },
                EnimyState::WaitLeft => {
                    let hide_decision
                                = rand::thread_rng().gen_range(0, 1000) <= 0;
                    let attack_decision
                                = rand::thread_rng().gen_range(0, 200) <= 0;
                    if hide_decision {
                        new_next_direct = Direct::RIGHT;
                        new_state = EnimyState::ToHideLeft;
                    } else if attack_decision {
                        new_next_direct = Direct::DOWN;
                        new_state = EnimyState::Attack;
                    }
                },
                EnimyState::WaitUp => {
                    let hide_decision
                                = rand::thread_rng().gen_range(0, 1000) <= 0;
                    let attack_decision
                                = rand::thread_rng().gen_range(0, 200) <= 0;
                    if hide_decision {
                        new_next_direct = Direct::DOWN;
                        new_state = EnimyState::ToHideUp;
                    } else if attack_decision {
                        new_next_direct = if self.hero.object.x >
                                                        enimy.object.x {
                            Direct::RIGHT
                        } else {
                            Direct::LEFT
                        };
                        new_state = EnimyState::Attack;
                    }
                },
                EnimyState::ToHideLeft => {
                    if (enimy.object.x == enimy.start_x) &&
                                        (enimy.object.y == enimy.start_y) {
                        new_next_direct = Direct::NONE;
                        new_state = EnimyState::HideLeft;
                    }
                },
                EnimyState::ToHideUp => {
                    if (enimy.object.x == enimy.start_x) &&
                                        (enimy.object.y == enimy.start_y) {
                        new_next_direct = Direct::NONE;
                        new_state = EnimyState::HideUp;
                    }
                },
                EnimyState::Attack => {
                    let mut node_touch = false;
                    let mut slide_touch = false;
                    for block in &self.blocks {
                        let intersect = block.object.rectangle_hit_test(
                                            &enimy.object, HitTestType::INNER);

                        if (block.block_type == BlockType::NODE) && intersect {
                            node_touch = true;
                        } else if (block.block_type == BlockType::SLIDE) &&
                                                                    intersect {
                            slide_touch = true;
                        }

                    }

                    if node_touch && !slide_touch {
                        //new_next_direct
                        let hero_object = &self.hero.object;

                        let mut horizontal_should = false;
                        let mut vertical_should = false;
                        let mut horizontal_direct = Direct::NONE;
                        let mut vertical_direct = Direct::NONE;

                        if enimy.object.x >
                                        hero_object.x + hero_object.width {
                            horizontal_should = true;
                            horizontal_direct = Direct::LEFT;
                        }
                        if enimy.object.x + enimy.object.width <
                                                            hero_object.x {
                            horizontal_should = true;
                            horizontal_direct = Direct::RIGHT;
                        }
                        if enimy.object.y >
                                        hero_object.y + hero_object.height {
                            vertical_should = true;
                            vertical_direct = Direct::UP;
                        }
                        if enimy.object.y + enimy.object.height <
                                                            hero_object.y {
                            vertical_should = true;
                            vertical_direct = Direct::DOWN;
                        }

                        if horizontal_should && vertical_should {
                            //true - horizontal, false - vertical
                            let vec = rand::thread_rng().gen_range(0, 2) <= 0;
                            if vec {
                                new_next_direct = horizontal_direct;
                            } else {
                                new_next_direct = vertical_direct;
                            }
                        } else if horizontal_should {
                            new_next_direct = horizontal_direct;
                        } else if vertical_should {
                            new_next_direct = vertical_direct;
                        }
                    }

                },
                _ => {}
            }
            enimy.next_direct = new_next_direct;
            enimy.action_state = new_state;

            //fire control
            let hero = &self.hero;
            let horizontal_see = (enimy.object.x < hero.object.x + hero.object.width) && (enimy.object.x + enimy.object.width > hero.object.x);

            let vertical_see = (enimy.object.y < hero.object.y + hero.object.height) && (enimy.object.y + enimy.object.height > hero.object.y);

            let fire_direct = if horizontal_see {
                if enimy.object.y > hero.object.y {
                    Direct::UP
                } else {
                    Direct::DOWN
                }
            } else if vertical_see {
                if enimy.object.x > hero.object.x {
                    Direct::LEFT
                } else {
                    Direct::RIGHT
                }
            } else {
                Direct::NONE
            };

            enimy.prepare_fire = fire_direct;
        }
    }
}

impl App {

    fn render(&mut self, args: &RenderArgs) {

        if !self.game.paused && !self.game.game_over {
            self.game.enimies_decision();
            self.game.logic();

            if self.game.hero.lives <= 0 {
                self.game.game_over = true;
                self.game.game_win = false;
            } else if self.game.enimies.len() <= 0 {
                self.game.game_over = true;
                self.game.game_win = true;
            }
        }

        //const SLIDE_COLOR: [f32; 4] = [1.0, 0.9, 0.3, 1.0];
        //const HOLE_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
        //const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
        const GR: [f32; 4] = [0.0, 1.0, 1.0, 1.0];

        const PAUSE_BANNER_COLOR: [f32; 4] = [0.9, 0.9, 0.1, 0.96];
        const WIN_BANNER_COLOR: [f32; 4] = [0.1, 0.9, 0.1, 0.97];
        const FAIL_BANNER_COLOR: [f32; 4] = [0.8, 0.1, 0.2, 0.99];
        const TRANSPARENT: [f32; 4] = [0.0, 0.0, 0.0, 0.0];


        use graphics::*;
//        println!("FRAME {}", self.counter);
//        self.counter += 1;

        let lives = self.game.hero.lives;
        let point_num = self.game.point_num;
        let crash_num = self.game.crash_num;

        let game_over = self.game.game_over;
        let game_win = self.game.game_win;
        let paused = self.game.paused;

        let blocks = &self.game.blocks;
        let enimies = &self.game.enimies;
        let free_bullets = &self.game.free_bullets;
        let hero = &self.game.hero;

        //drawing
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            let transform = c.transform;

            for block in blocks {
                let obj = &block.object;
                let square = rectangle::square(obj.x, obj.y,
                                                obj.width);
                // Draw a box rotating around the middle of the screen.

                let color = match block.block_type {
                    BlockType::WALL => BLUE,
                    BlockType::NODE => YELLOW,
                    BlockType::HOLE => GR,
                    BlockType::SLIDE => YELLOW,
                };

                rectangle(color, square, transform, gl);
            }

            for enimy in enimies {
                let obj = &enimy.object;
                let square = rectangle::square(obj.x, obj.y,
                                                obj.width);
                rectangle(RED, square, transform, gl);
            }

            let obj = &hero.object;
            let square = rectangle::square(obj.x, obj.y,
                                            obj.width);
            if hero.lives > 0 {
                rectangle(GREEN, square, transform, gl);
            }

            for bullet in &hero.bullets {
                let obj = &bullet.object;
                let square = rectangle::square(obj.x, obj.y,
                                                obj.width);

                rectangle(RED, square, transform, gl);
            }

            for bullet in free_bullets {
                let obj = &bullet.object;
                let square = rectangle::square(obj.x, obj.y,
                                                obj.width);
                rectangle(RED, square, transform, gl);
            }

            for enimy in enimies {
                for bullet in &enimy.bullets {
                    let obj = &bullet.object;
                    let square = rectangle::square(obj.x, obj.y,
                                                    obj.width);

                    rectangle(RED, square, transform, gl);
                }
            }

            let left_hud_border = WIDTH_CELL_SIZE*17.0 +
                                    WIDTH_HUD_SEGMENT_SIZE/2.0;

            //print lives of hero as green squares
            for i in 0..lives {
                let offense = left_hud_border +
                                        (i as f64)*WIDTH_HUD_SEGMENT_SIZE*1.5;
                let square = rectangle::square(offense, 0.0,
                                                    WIDTH_HUD_SEGMENT_SIZE);
                rectangle(GREEN, square, transform, gl);
            }

            //print count of killed enemies as red squares
            for i in 0..point_num {
                let offense = HEIGHT_HUD_SEGMENT_SIZE*1.5 +
                                        (i as f64)*HEIGHT_HUD_SEGMENT_SIZE*1.5;
                let square = rectangle::square(left_hud_border, offense,
                                                    WIDTH_HUD_SEGMENT_SIZE);
                rectangle(RED, square, transform, gl);
            }

            //print count of crashed enemies as yellow squares
            for i in 0..crash_num {
                let offense = HEIGHT_HUD_SEGMENT_SIZE*1.5 +
                                        (i as f64)*HEIGHT_HUD_SEGMENT_SIZE*1.5;
                let square = rectangle::square(left_hud_border +
                                            WIDTH_HUD_SEGMENT_SIZE*1.5,
                                            offense, WIDTH_HUD_SEGMENT_SIZE);
                rectangle(YELLOW, square, transform, gl);
            }

            //draw banners - none(transparent), pause, win or fail of game
            let banner_color = if paused {
                PAUSE_BANNER_COLOR
            } else if game_over && game_win {
                WIN_BANNER_COLOR
            } else if game_over && !game_win {
                FAIL_BANNER_COLOR
            } else {
                TRANSPARENT
            };

            if banner_color != TRANSPARENT {
                let square = [
                                                WIDTH_CELL_SIZE*5.0,
                                                HEIGHT_CELL_SIZE*5.5,
                                                WIDTH_CELL_SIZE*7.0,
                                                HEIGHT_CELL_SIZE*3.0];

                rectangle(banner_color, square, transform, gl);
            }
        });
    }

    fn input(&mut self, button: &Button) {

        if self.game.paused {
            match button {
                Button::Keyboard(key) =>
                    match key {
                        Key::P => self.game.paused = false,
                        _ => println!("Previously unpause game by pressing 'P'")
                    },
                _ => println!("Non keyboard button")
            }
        } else if self.game.game_over {
            match button {
                Button::Keyboard(key) =>
                    match key {
                        Key::Return => self.game.create_level(),
                        _ => println!("Previously restart game by pressing 'Enter'")
                    },
                _ => println!("Non keyboard button")
            }
        } else {

            let hero = &self.game.hero;
            let mut new_direct = Direct::NONE;
            let mut new_fire_direct = hero.prepare_fire;
            match button {
                Button::Keyboard(key) =>
                    match key {
                        Key::P => self.game.paused = true,
                        Key::Space => new_direct = hero.next_direct,
                        Key::W => new_direct = Direct::UP,
                        Key::S => new_direct = Direct::DOWN,
                        Key::A => new_direct = Direct::LEFT,
                        Key::D => new_direct = Direct::RIGHT,
                        Key::I => new_fire_direct = Direct::UP,
                        Key::K => new_fire_direct = Direct::DOWN,
                        Key::J => new_fire_direct = Direct::LEFT,
                        Key::L => new_fire_direct = Direct::RIGHT,
                        _ => println!("Another keyboard button")
                    },
                _ => println!("Non keyboard button")
            }

           //access move changing
            let hero = &mut self.game.hero;
            if hero.next_direct == new_direct {
               hero.next_direct = Direct::NONE;
           } else if new_direct != Direct::NONE {
               hero.next_direct = new_direct;
            }

            //access creating bullet
            hero.prepare_fire = new_fire_direct;

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
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and init it
    let mut game = Game::new();
    game.create_level();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        //rotation: 0.0,
        //counter: 0,
        game: game
    };

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
