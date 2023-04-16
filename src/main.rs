use crate::board_state::{BoardState, PositionVector, BLACK, WHITE};
use std::io;

mod board_state;

fn notation_to_coordinates(input: String) -> Result<PositionVector, &'static str> {
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
    let col = match column_string {
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
    return Ok(PositionVector { row: row - 1, col });
}

fn coordinates_to_notation(input: PositionVector) -> String {
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
    return input;
}

fn main() {
    let bs: BoardState = BoardState::new();
    loop {
        println!("{}", bs.str());
        println!("white in check: {}\nblack in check: {}", bs.is_color_in_check(WHITE), bs.is_color_in_check(BLACK));
        let input = read_line("Provide coordinates of piece to move:");
        let coordinates = match notation_to_coordinates(input) {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let moves = bs.get_piece_moves_respecting_checks(coordinates);
        if moves.len() == 0 {
            println!("no possible moves");
        }
        for i in 0..moves.len() {
            println!("[{0}] {1}", i, coordinates_to_notation(coordinates + moves[i]));
        }
        let _move_number: i32 = loop {
            let number: i32 = read_line("select move (0, 1...)").parse().expect("Not a number");
            if number < moves.len() as i32 || number >= 0 {
                break number;
            }
            println!("Invalid number");
        };
    }
}
