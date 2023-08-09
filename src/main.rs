use std::io;
use std::process::exit;

use crate::board_state::{BoardState, Position};

mod board_state;

fn coordinates_to_notation(input: Position) -> String {
    if input == board_state::INVALID_POSITION {
        return String::from("");
    }
    let row = char::from_digit((input.row + 1) as u32, 10).expect("Invalid coordinate");
    let col: char = char::from(97u8 + input.col as u8);
    let mut result = col.to_string();
    result.push(row);
    return result;
}

fn read_line(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut input).expect("No input received");
    input = input.replace("\n", "");
    if input == "exit" {
        exit(0);
    }
    return input;
}

fn game_loop() {
    let mut bs: BoardState = BoardState::new();
    loop {
        println!("{}", bs.str());
        println!("white in check: {}", bs.is_color_in_check(board_state::WHITE));
        println!("black in check: {}", bs.is_color_in_check(board_state::BLACK));
        println!("en-passant square: {}", coordinates_to_notation(bs.get_en_passant_square()));
        let input = read_line("Provide coordinates of piece:");
        let original_square = board_state::notation_to_coordinates(&input);
        let moves = bs.get_piece_moves_respecting_checks(original_square);
        if moves.len() == 0 {
            println!("no possible moves");
            continue;
        }
        for i in 0..moves.len() {
            print!("{}", coordinates_to_notation(moves[i]));
            if i < moves.len() - 1 {
                print!(" ,");
            } else {
                println!()
            }
        }
        let new_position: Position = loop {
            let written_move: String = read_line("select move:");
            let position = board_state::notation_to_coordinates(&written_move);
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
