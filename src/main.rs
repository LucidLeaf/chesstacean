use crate::board_state::{get_piece_moves_disregarding_checks, BoardState};
use std::io;
use std::time::Instant;

mod board_state;

fn notation_to_coordinates(input: String) -> Result<(i32, i32), &'static str> {
    if input.len() != 2 {
        return Err("Invalid input length");
    }
    let formatted_input = input.to_lowercase();
    let column_string = formatted_input.get(0..1).expect("Error converting column");
    let row_string = formatted_input.get(1..2).expect("Error converting row");
    if !["a", "b", "c", "d", "e", "f", "g", "h"].contains(&column_string) {
        return Err("Invalid column");
    }
    if !["1", "2", "3", "4", "5", "6", "7", "8"].contains(&row_string) {
        return Err("Invalid row");
    }
    let column = match column_string {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        "e" => 4,
        "f" => 5,
        "g" => 6,
        "h" => 7,
        _ => 0,
    };
    let row: i32 = row_string.parse().expect("Not a number");
    return Ok((row - 1, column));
}

fn read_line(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut input).expect("No input received");
    input = input.replace("\n", "");
    return input;
}

fn main() {
    let bs: BoardState = board_state::new();
    board_state::print_state(&bs);
    loop {
        let mut input = read_line("Provide coordinates of piece to move:");
        let coordinates = match notation_to_coordinates(input) {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let now = Instant::now();
        let moves = get_piece_moves_disregarding_checks(&bs, coordinates);
        let elapsed = now.elapsed();
        println!("calculation took {:.2?}", elapsed);
        if moves.len() == 0 {
            println!("no possible moves");
        }
        for i in 0..moves.len() {
            println!("{0}: ({1}, {2})", i, moves[i].0, moves[i].1);
        }
        let mut move_number: i32 = loop {
            let number: i32 = read_line("select move").parse().expect("Not a number");
            if number < moves.len() as i32 || number >= 0 {
                break number;
            }
            println!("Invalid number");
        };
    }
}
