#[derive(PartialEq, Debug)]
pub enum Direction {
    North,
    East,
    West,
    South,
}

pub struct Rover {
    pub name: String,
    pub pos_x: i32,
    pub pos_y: i32,
    pub direction: Direction,
    pub map: Vec<Vec<char>>,
}

enum TurnDirection {
    Left,
    Right,
}

enum MoveDirection {
    Forward,
    Backward,
}

struct MoveCommand {
    distance: i32,
    direction: MoveDirection,
}

struct TurnCommand {
    direction: TurnDirection,
}

enum Command {
    Move(MoveCommand),
    Turn(TurnCommand),
}


fn parse_command(command: String) -> Result<Command, &'static str> {
    let words: Vec<&str> = command.split("-").collect();

    match words[0] {
        "move" => {
            let direction = match words[1] {
                "forward" => Ok(MoveDirection::Forward),
                "backward" => Ok(MoveDirection::Backward),
                _ => Err("Unknown direction"),
            }?;

            let move_count = command.split("-").last().ok_or("xxx")?;
            let move_count: i32 = move_count.parse().map_err(|e| "not valid number")?;

            Ok(Command::Move(MoveCommand {
                distance: move_count,
                direction,
            }))
        }
        "turn" => match words[1] {
            "left" => Ok(Command::Turn(TurnCommand {
                direction: TurnDirection::Left,
            })),
            "right" => Ok(Command::Turn(TurnCommand {
                direction: TurnDirection::Right,
            })),
            _ => Err("Command not supported"),
        },
        _ => Err("Command not supported"),
    }
}

fn execute_turn(rover: &mut Rover, command: TurnCommand) {
    let new_direction = match command.direction {
        TurnDirection::Left =>
            match rover.direction {
                Direction::North => Direction::West,
                Direction::East => Direction::North,
                Direction::West => Direction::South,
                Direction::South => Direction::East
            }
        TurnDirection::Right =>
            match rover.direction {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::West => Direction::North,
                Direction::South => Direction::West
            }
    };

    rover.direction = new_direction;
}

fn execute_move(rover: &mut Rover, command: MoveCommand) {
    let dim: i32 = rover.map.len() as i32;

    let direction = match command.direction {
        MoveDirection::Forward => 1,
        MoveDirection::Backward => -1,
    };

    let mut distance;
    match rover.direction {
        Direction::North => {
            distance = (rover.pos_y - command.distance * direction) % dim;
        }
        Direction::East => {
            distance = (rover.pos_x + command.distance * direction) % dim;
        }
        Direction::West => {
            distance = (rover.pos_x - command.distance * direction) % dim;
        }
        Direction::South => {
            distance = (rover.pos_y + command.distance * direction) % dim;
        }
    }

    if distance < 0 {
        distance = dim + distance
    }

    match rover.direction {
        Direction::North | Direction::South => {
            rover.pos_y = distance;
        }
        Direction::East | Direction::West => {
            rover.pos_x = distance;
        }
    }
}

impl Rover {
    pub fn send_command(&mut self, command: String) {
        let command = parse_command(command).unwrap();

        match command {
            Command::Move(move_command) => execute_move(self, move_command),
            Command::Turn(turn_command) => execute_turn(self, turn_command),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rover::*;
    #[test]
    fn it_works() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::East,
            map,
        };

        rover.send_command(String::from("move-forward-1"));

        assert_eq!(rover.pos_x, 1);
        assert_eq!(rover.pos_y, 0);
    }

    #[test]
    fn it_works_move_2() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::East,
            map,
        };

        rover.send_command(String::from("move-forward-2"));

        assert_eq!(rover.pos_x, 2);
        assert_eq!(rover.pos_y, 0);
    }

    #[test]
    fn it_works_move_3() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::South,
            map,
        };

        rover.send_command(String::from("move-forward-1"));

        assert_eq!(rover.pos_x, 0);
        assert_eq!(rover.pos_y, 1);
    }

    #[test]
    fn it_works_move_4() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::South,
            map,
        };

        rover.send_command(String::from("move-forward-21"));

        assert_eq!(rover.pos_x, 0);
        assert_eq!(rover.pos_y, 1);
    }

    #[test]
    fn it_works_move_5() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 9,
            pos_y: 9,
            direction: Direction::South,
            map,
        };

        rover.send_command(String::from("move-forward-2"));

        assert_eq!(rover.pos_x, 9);
        assert_eq!(rover.pos_y, 1);
    }

    #[test]
    fn it_works_move_backward_5() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 9,
            pos_y: 9,
            direction: Direction::South,
            map,
        };

        rover.send_command(String::from("move-backward-2"));

        assert_eq!(rover.pos_x, 9);
        assert_eq!(rover.pos_y, 7);
    }

    #[test]
    fn it_works_move_backward_6() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::South,
            map,
        };

        rover.send_command(String::from("move-backward-2"));

        assert_eq!(rover.pos_x, 0);
        assert_eq!(rover.pos_y, 8);
    }

    #[test]
    fn it_works_move_backward_7() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::South,
            map,
        };

        rover.send_command(String::from("move-backward-12"));

        assert_eq!(rover.pos_x, 0);
        assert_eq!(rover.pos_y, 8);
    }

    #[test]
    fn it_works_move_backward_8() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::East,
            map,
        };

        rover.send_command(String::from("move-backward-12"));

        assert_eq!(rover.pos_x, 8);
        assert_eq!(rover.pos_y, 0);
    }

    #[test]
    fn it_works_turn_left() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::East,
            map,
        };

        rover.send_command(String::from("turn-left"));

        assert_eq!(rover.direction, Direction::North);
    }


    #[test]
    fn it_works_turn_left_2() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::North,
            map,
        };

        rover.send_command(String::from("turn-left"));

        assert_eq!(rover.direction, Direction::West);
    }
}
