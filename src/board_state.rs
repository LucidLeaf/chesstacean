#![allow(unused)]

use std::hint::black_box;

const NOTHING: i32 = 0;
const PAWN: i32 = 1;
const ROOK: i32 = 2;
const KNIGHT: i32 = 3;
const BISHOP: i32 = 4;
const QUEEN: i32 = 5;
const KING: i32 = 6;

const WHITE: i32 = 8;
const BLACK: i32 = 16;

const INVALID_SQUARE: (i32, i32) = (256, 256);
const PIECE_MASK: i32 = 7;
const COLOR_MASK: i32 = 24;

pub struct BoardState {
    // [row][column] (0,0)=a1 (0,1)=b1...
    board: [[i32; 8]; 8],
    turn: i32,
    en_passant: (i32, i32),
    white_castling_rights: bool,
    black_castling_rights: bool,
}

pub fn new() -> BoardState {
    BoardState {
        board: [
            [WHITE + ROOK, WHITE + KNIGHT, WHITE + BISHOP, WHITE + QUEEN, WHITE + KING, WHITE + BISHOP, WHITE + KNIGHT, WHITE + ROOK],
            [WHITE + PAWN; 8],
            [NOTHING; 8],
            [NOTHING; 8],
            [NOTHING; 8],
            [NOTHING; 8],
            [BLACK + PAWN; 8],
            [BLACK + ROOK, BLACK + KNIGHT, BLACK + BISHOP, BLACK + QUEEN, BLACK + KING, BLACK + BISHOP, BLACK + KNIGHT, BLACK + ROOK],
        ],
        turn: WHITE,
        en_passant: INVALID_SQUARE,
        white_castling_rights: true,
        black_castling_rights: true,
    }
}

pub fn print_state(state: &BoardState) {
    fn piece_string_representation(piece: i32) -> String {
        let colorless_piece = piece & PIECE_MASK;
        let char = match colorless_piece {
            PAWN => "p",
            ROOK => "r",
            KNIGHT => "n",
            BISHOP => "b",
            QUEEN => "q",
            KING => "k",
            _ => " ",
        };
        if is_piece_white(piece) {
            return char.to_uppercase();
        }
        return char.to_string();
    }
    for n in (0..64).rev() {
        //start from top right
        let col = 7 - n % 8;
        let row = n / 8;
        let string = piece_string_representation(state.board[row][col]);
        print!("{} ", string);
        if n % 8 == 0 {
            println!()
        }
    }
}

fn is_piece_white(piece: i32) -> bool {
    let color = piece & COLOR_MASK;
    if color == WHITE {
        return true;
    }
    return false;
}

fn is_en_passant_field(state: &BoardState, position: (i32, i32)) -> bool {
    let ep_square = state.en_passant;
    return (position.0 == ep_square.0) && (position.1 == ep_square.1);
}

fn is_opposite_color(piece_1: i32, piece_2: i32) -> bool {
    let col_1 = piece_1 & COLOR_MASK;
    let col_2 = piece_2 & COLOR_MASK;
    return (col_1 == WHITE && col_2 == BLACK) || (col_1 == BLACK && col_2 == WHITE);
}

fn is_same_color(piece_1: i32, piece_2: i32) -> bool {
    let col_1 = piece_1 & COLOR_MASK;
    let col_2 = piece_2 & COLOR_MASK;
    return (col_1 == col_2) && (col_1 != NOTHING);
}

fn is_move_out_of_bounds(position: (i32, i32), relative_move: (i32, i32)) -> bool {
    let new_position = (position.0 + relative_move.0, position.1 + relative_move.1);
    if new_position.0 < 0 || new_position.1 < 0 || new_position.0 > 7 || new_position.1 > 7 {
        return true;
    }
    return false;
}

fn get_square_content(state: &BoardState, position: (i32, i32)) -> i32 {
    return state.board[position.0 as usize][position.1 as usize];
}

pub fn get_piece_moves_disregarding_checks(state: &BoardState, position: (i32, i32)) -> Vec<(i32, i32)> {
    let piece = get_square_content(state, position);
    let color = piece & COLOR_MASK;
    let colorless_piece = piece & PIECE_MASK;
    let moves = match colorless_piece {
        PAWN => get_pawn_moves(state, position, piece),
        ROOK => get_rook_moves(state, position, piece),
        KNIGHT => get_knight_moves(state, position, piece),
        BISHOP => get_bishop_moves(state, position, piece),
        QUEEN => get_queen_moves(state, position, piece),
        KING => get_king_moves(state, position, piece),
        _ => Vec::new(),
    };
    return moves;
}

fn get_controlled_squares(state: &BoardState, color: i32) -> [[bool; 8]; 8] {
    let mut squares = [[false; 8]; 8];
    for row in 0..8 {
        for col in 0..8 {
            let square = get_square_content(state, (row, col));
            if is_same_color(square, color) {
                todo!();
            }
        }
    }
    return squares;
}

fn get_pawn_moves(state: &BoardState, position: (i32, i32), piece: i32) -> Vec<(i32, i32)> {
    let is_on_home_row = || -> bool {
        let color = piece & COLOR_MASK;
        let row: i32 = if color == WHITE { 1 } else { 6 };
        return position.0 == row;
    };
    let move_direction = if is_piece_white(piece) { 1 } else { -1 };
    let mut moves = Vec::new();
    //if no piece is blocking the way, we can move forwards one square
    if get_square_content(state, (position.0 + move_direction * 1, position.1)) == NOTHING {
        moves.push((move_direction * 1, 0));
        //if the pawn is on its home row and the square two in front is also free, it can move two squares
        if is_on_home_row() && get_square_content(state, (position.0 + move_direction * 2, position.1)) == NOTHING {
            moves.push((move_direction * 2, 0));
        }
    }
    //check diagonal squares for taking a piece
    for i in [-1, 1] {
        let diagonal_square = (move_direction * 1, i);
        let row = position.0 + diagonal_square.0;
        let col = position.1 + diagonal_square.1;
        //prevent checks outside of the board
        if is_move_out_of_bounds(position, diagonal_square) {
            continue;
        }
        let diagonal_square_piece = get_square_content(state, (row, col));
        if is_opposite_color(piece, diagonal_square_piece) || is_en_passant_field(state, (row, col)) {
            moves.push(diagonal_square);
        }
    }
    return moves;
}

fn get_sliding_moves(state: &BoardState, position: (i32, i32), piece: i32, straight: bool) -> Vec<(i32, i32)> {
    let mut moves = Vec::new();
    for direction in [-1, 1] {
        for toggle in [true, false] {
            let mut length = 1;
            loop {
                let new_move = if straight {
                    if toggle {
                        (direction * length, 0)
                    } else {
                        (0, direction * length)
                    }
                } else {
                    if toggle {
                        (direction * length, length)
                    } else {
                        (direction * length, -length)
                    }
                };
                if is_move_out_of_bounds(position, new_move) {
                    break;
                }
                let new_field = get_square_content(state, (position.0 + new_move.0, position.1 + new_move.1));
                if is_same_color(piece, new_field) {
                    break;
                }
                moves.push(new_move);
                if is_opposite_color(piece, new_field) {
                    break;
                }
                length += 1;
            }
        }
    }
    return moves;
}

fn get_rook_moves(state: &BoardState, position: (i32, i32), piece: i32) -> Vec<(i32, i32)> {
    return get_sliding_moves(state, position, piece, true);
}

fn get_knight_moves(state: &BoardState, position: (i32, i32), piece: i32) -> Vec<(i32, i32)> {
    let mut moves = Vec::new();
    for long_side in [-2, 2] {
        for short_side in [-1, 1] {
            for new_move in [(long_side, short_side), (short_side, long_side)] {
                if is_move_out_of_bounds(position, new_move) {
                    continue;
                }
                let other_square = get_square_content(state, (position.0 + new_move.0, position.1 + new_move.1));
                if is_same_color(piece, other_square) {
                    continue;
                }
                moves.push(new_move);
            }
        }
    }
    return moves;
}

fn get_bishop_moves(state: &BoardState, position: (i32, i32), piece: i32) -> Vec<(i32, i32)> {
    return get_sliding_moves(state, position, piece, false);
}

fn get_queen_moves(state: &BoardState, position: (i32, i32), piece: i32) -> Vec<(i32, i32)> {
    let mut moves = get_sliding_moves(state, position, piece, true);
    moves.extend(get_sliding_moves(state, position, piece, false));
    return moves;
}

fn get_king_moves(state: &BoardState, position: (i32, i32), piece: i32) -> Vec<(i32, i32)> {
    let mut moves = Vec::new();
    for row in [-1, 0, 1] {
        for col in [-1, 0, 1] {
            let new_move = (row, col);
            if is_move_out_of_bounds(position, new_move) {
                continue;
            }
            let new_square = get_square_content(state, (position.0 + new_move.0, position.1 + new_move.1));
            if is_same_color(piece, new_square) {
                continue;
            }
            moves.push(new_move);
        }
    }
    //castling
    if (is_piece_white(piece) && state.white_castling_rights) || (!is_piece_white(piece) && state.black_castling_rights) {
        //short castle
        let one_square_left = get_square_content(state, (position.0, position.1 - 1));
        let two_squares_left = get_square_content(state, (position.0, position.1 - 2));
        if one_square_left == NOTHING && two_squares_left == NOTHING {
            moves.push((0, -2));
        }
        //long castle
        let one_square_right = get_square_content(state, (position.0, position.1 + 1));
        let two_squares_right = get_square_content(state, (position.0, position.1 + 2));
        let three_squares_right = get_square_content(state, (position.0, position.1 + 3));
        if one_square_right == NOTHING && two_squares_right == NOTHING && three_squares_right == NOTHING {
            moves.push((0, 2));
        }
    }
    return moves;
}
