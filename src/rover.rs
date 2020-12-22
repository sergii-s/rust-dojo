#[derive(PartialEq, Debug)]
pub enum Direction {
    North,
    East,
    West,
    South,
}

pub trait RoverProcessor {
    fn parse_command<'x>(&self, command: &'x str) -> Result<Box<dyn Command + 'x>, &'static str>;
}

pub struct RoverProcessorV1;

impl RoverProcessor for RoverProcessorV1 {
    fn parse_command(&self, command: &str) -> Result<Box<dyn Command>, &'static str> {
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

                Ok(Box::new(MoveCommand {
                    distance: move_count,
                    direction,
                }))
            }
            "turn" => match words[1] {
                "left" => Ok(Box::new(TurnCommand {
                    direction: TurnDirection::Left,
                })),
                "right" => Ok(Box::new(TurnCommand {
                    direction: TurnDirection::Right,
                })),
                _ => Err("Command not supported"),
            },
            _ => Err("Command not supported"),
        }
    }
}

pub struct RoverProcessorV2;

impl RoverProcessor for RoverProcessorV2 {
    fn parse_command<'x>(&self, command: &'x str) -> Result<Box<dyn Command + 'x>, &'static str> {
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

                Ok(Box::new(MoveCommand {
                    distance: move_count,
                    direction,
                }))
            }
            "turn" => match words[1] {
                "left" => Ok(Box::new(TurnCommand {
                    direction: TurnDirection::Left,
                })),
                "right" => Ok(Box::new(TurnCommand {
                    direction: TurnDirection::Right,
                })),
                _ => Err("Command not supported"),
            },
            //"print-hohoh-hahahaha"
            "print" => {
                let message = &command[6..];
                Ok(Box::new(PrintCommand::<'x>{ message }))
            }
            _ => Err("Command not supported"),
        }
    }
}

pub struct Rover {
    pub name: String,
    pub pos_x: i32,
    pub pos_y: i32,
    pub direction: Direction,
    pub processor: Box<dyn RoverProcessor>,
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

struct PrintCommand<'x> {
    message: &'x str,
}

trait Command {
    fn execute(&self, rover: &mut Rover);
}

impl Command for MoveCommand {
    fn execute(&self, rover: &mut Rover) {
        let dim: i32 = rover.map.len() as i32;

        let direction = match self.direction {
            MoveDirection::Forward => 1,
            MoveDirection::Backward => -1,
        };

        let mut distance;
        match rover.direction {
            Direction::North => {
                distance = (rover.pos_y - self.distance * direction) % dim;
            }
            Direction::East => {
                distance = (rover.pos_x + self.distance * direction) % dim;
            }
            Direction::West => {
                distance = (rover.pos_x - self.distance * direction) % dim;
            }
            Direction::South => {
                distance = (rover.pos_y + self.distance * direction) % dim;
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
}


impl Command for TurnCommand {
    fn execute(&self, rover: &mut Rover) {
        let new_direction = match self.direction {
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
}

impl Command for PrintCommand<'_> {
    fn execute(&self, rover: &mut Rover) {
        println!("{}", self.message)
    }
}

impl Rover {
    pub fn send_command(&mut self, command_string: String) {
        let command = self.processor.parse_command(&command_string).unwrap();
        command.execute(self);
    }

    pub fn scan(&self) -> [&[char]; 3] {
        let y_start : usize = (self.pos_y - 1) as usize;
        let y_mid : usize = (self.pos_y) as usize;
        let y_end : usize = (self.pos_y + 1) as usize;
        let x_start : usize = (self.pos_x - 1) as usize;
        let x_end : usize = (self.pos_x + 2) as usize;

        let res1 = &self.map[y_start][x_start..x_end];
        let res2 = &self.map[y_mid][x_start..x_end];
        let res3 = &self.map[y_end][x_start..x_end];

        [res1, res2, res3]
    }
}

// OOOO
// OOOO
// OOOO
// OOOO

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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
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
            processor: Box::new(RoverProcessorV1),
            map,
        };

        rover.send_command(String::from("turn-left"));

        assert_eq!(rover.direction, Direction::West);
    }

    #[test]
    fn it_works_print() {
        let map = vec![vec!['O'; 10]; 10];
        let mut rover = Rover {
            name: String::from("Discovery"),
            pos_x: 0,
            pos_y: 0,
            direction: Direction::North,
            processor: Box::new(RoverProcessorV2),
            map,
        };

        rover.send_command(String::from("print-xxxx-aaaa"));

        assert_eq!(rover.direction, Direction::West);
    }

    #[test]
    fn it_works_scan() {
        let res : [&[char]; 3];

        {
            let mut map = vec![vec!['O'; 10]; 10];
            let mut x = 1;
            let mut rover = Rover {
                name: String::from("Discovery"),
                pos_x: x,
                pos_y: 1,
                direction: Direction::North,
                processor: Box::new(RoverProcessorV2),
                map,
            };

            x = 2;

            res =  rover.scan();
            assert_eq!(res, [['O','X','O'], ['O','O','O'], ['O','O','Y']]);
        }
    }
}
