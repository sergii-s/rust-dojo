use crate::rover::{Direction, Rover};

mod rover;

// Implement Mars rover logic.

// Mars rover has it position and can execute commands.

// MAP 10x10
// O -> terre
// X -> mountain

// * Commands are sent as text.
// * move-forward-x
// * move-backward-x
// * turn-left
// * turn-right

// 0000
// 0X00
// 0000
// 0000

// * Obstacles detections (stop and report)
// * Map is sphere
// * Bonuses
// * map can be edited

fn main() {
    let map = vec![vec!['O'; 10]; 10];

    let mut x = Rover {
        name: String::from("Discovery"),
        pos_x: 0,
        pos_y: 0,
        direction: Direction::East,
        map,
    };
    x.send_command(String::from("xxxx"));

    println!("Hello, rover!");
}
