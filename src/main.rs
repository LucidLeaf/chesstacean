use std::io;
use std::process::exit;

use crate::board_state::{BoardState, Position};

mod board_state;

fn read_line(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut input).expect("No input received");
    input = input.replace("\n", "");
    if input.to_lowercase() == "exit" {
        exit(0);
    }
    return input;
}

fn game_loop() {
    println!("Type \"exit\" at any point to stop the game");
    let mut bs: BoardState = BoardState::new();
    loop {
        println!("{}", bs.full_state_str());
        if bs.is_checkmate() {
            println!("Checkmate");
            break;
        }
        let starting_position = loop {
            let line = read_line("Provide coordinates of piece:");
            if line.len() != 2 {
                continue;
            }
            break line;
        };

        let original_square = board_state::Position::position_from_string(&starting_position);
        let moves = bs.get_piece_moves_respecting_checks(original_square);
        if moves.len() == 0 {
            println!("no possible moves");
            continue;
        }
        for i in 0..moves.len() {
            print!("{}", board_state::coordinates_to_notation(moves[i]));
            if i < moves.len() - 1 {
                print!(" ,");
            } else {
                println!()
            }
        }
        let new_position: Position = loop {
            let written_move: String = read_line("select move:");
            let position = board_state::Position::position_from_string(&written_move);
            if moves.contains(&position) {
                break position;
            }
        };
        bs = bs.perform_move(original_square, new_position);
    }
}

fn main() {
    game_loop();
}
