use std::io;
use std::process::exit;

use crate::board_state::{BoardState, Position};

mod board_state;

fn notation_to_coordinates(input: String) -> Result<Position, &'static str> {
    if input.len() != 2 {
        return Err("Invalid input length");
    }
    let formatted_input = input.to_lowercase();
    let column_string = formatted_input.get(0..1).expect("Error converting column");
    let row_string = formatted_input.get(1..2).expect("Error converting row");
    let col = match column_string {
        "a" => 1,
        "b" => 2,
        "c" => 3,
        "d" => 4,
        "e" => 5,
        "f" => 6,
        "g" => 7,
        "h" => 8,
        _ => 0,
    };
    let row: i32 = row_string.parse().expect("Not a number");
    return Ok(Position { row: row - 1, col: col - 1 });
}

fn coordinates_to_notation(input: Position) -> String {
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
        let original_square = match notation_to_coordinates(input) {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let moves = bs.get_piece_moves_respecting_checks(original_square);
        if moves.len() == 0 {
            println!("no possible moves");
            continue;
        }
        for i in 0..moves.len() {
            print!("{}", coordinates_to_notation(original_square + moves[i]));
            if i < moves.len() - 1 {
                print!(" ,");
            } else {
                println!()
            }
        }
        let square_to_move_to: Position = loop {
            let written_move: String = read_line("select move:");
            let position_option = notation_to_coordinates(written_move);
            if position_option.is_err() {
                println!("Not a move from above");
            } else {
                let position = position_option.unwrap();
                let relative_position = position - original_square;
                if moves.contains(&relative_position) {
                    break position;
                }
            }
        };
        let relative_move = Position::absolute_to_relative(original_square, square_to_move_to);
        bs = bs.perform_move(original_square, relative_move);
    }
}

fn main() {
    game_loop();
}
