# README

## Description

This game is remake of the [CrossFire](https://en.wikipedia.org/wiki/Crossfire_(1981_video_game))(1981) in the style of abstractionism.

## Content of repository

- ```README.md``` - this file with general description;
- ```.gitignore``` - wildcard of files, that will not save in repository;
- ```Cargo.toml``` - ```Cargo``` project file;
- ```main.rs``` - source code of program.

## Building

1. Install ```Rust``` according the instructions:
    ```
    https://www.rust-lang.org/tools/install
    ```
1. Build and run program:
    ```
    cargo run --release
    ```

## Rules

The player controls robot that moves among wall-blocks and can shoot in all 4 directions.
He should destroy all enemy robots avoiding collisions with them and their incoming fire.

The player can stand only on the crossroads, in the passage he will always slide.
The player and enemies can launch only one bullet at the time.

The player has three lives, and when he loses life, he spawns again at the starting point.
If player loses all lives, game will be over and player will fail.
Enemies have only one lives.

## Blocks

There are 5 type of blocks on the field:

- Green - player robot;
- Red - enemy robots;
- Blue - walls;
- Yellow - passages;
- Cyan - enemy zone.

Small red squares are bullets.

## Control

List of movement keys:

- ```W``` - start move up;
- ```A``` - start move left;
- ```S``` - start move down;
- ```D``` - start move right;
- ```Space``` - stop on the first oncoming crossroad;

If the player presses motion button while robot is standing, the robot will begin to move in the specified direction.
If the player presses movement button that matches direction of the robot movement, the robot will stop on the oncoming crossroad.
If the player presses movement button that not matches direction of the robot movement, the robot will turn in the specified direction on the oncoming crossroad.
If the player presses ``` Space```  while robot is moving, the robot will stop on the oncoming crossroad.

List of shoot keys:

- ```I``` - shoot up;
- ```J``` - shoot left;
- ```K``` - shoot down;
- ```L``` - shoot right;

List of other keys:

- ```Esc``` - quit game;
- ```P``` - pause game;
- ```Enter``` - restart if game is over (win or fail).

## Indicators

There are 3 indicators in the upper right corner display the following parameters:

- Green squares - count of lives;
- Red squares - count of killed enemies (points);
- Yellow squares - count of crashed enemies (by player or each other).

## Banners

The game pauses when banner covers playing field.

There are 3 types of banners:

- Yellow banner - game paused, press ```P``` for unpause;
- Red banner - game over and player fail, press ```Enter``` for restart;
- Green banner - game over and player win, press ```Enter``` for restart.
