use std::ops::Add;

const NOTHING: i32 = 0;
const PAWN: i32 = 1;
const ROOK: i32 = 2;
const KNIGHT: i32 = 3;
const BISHOP: i32 = 4;
const QUEEN: i32 = 5;
const KING: i32 = 6;

pub const WHITE: i32 = 8;
pub const BLACK: i32 = 16;

const INVALID_POSITION: PositionVector = PositionVector { row: 1234567890, col: 1234567890 };
const PIECE_MASK: i32 = 7;
const COLOR_MASK: i32 = 24;

#[derive(Clone, Copy, Debug)]
pub struct PositionVector {
    pub(crate) row: i32,
    pub(crate) col: i32,
}

impl Add for PositionVector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { row: self.row + other.row, col: self.col + other.col }
    }
}

impl PartialEq<Self> for PositionVector {
    fn eq(&self, other: &Self) -> bool {
        self.col == other.col && self.row == other.row
    }
}

pub struct BoardState {
    // [row][column] (0,0)=a1 (0,1)=b1...
    board: [[i32; 8]; 8],
    color_to_move: i32,
    en_passant: PositionVector,
    white_castling_rights: bool,
    black_castling_rights: bool,
}

impl BoardState {
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
            color_to_move: WHITE,
            en_passant: INVALID_POSITION,
            white_castling_rights: true,
            black_castling_rights: true,
        }
    }

    pub fn str(&self) -> String {
        fn piece_string_representation(piece: i32) -> String {
            let colorless_piece = piece & PIECE_MASK;
            let char = match colorless_piece {
                PAWN => "p ",
                ROOK => "r ",
                KNIGHT => "n ",
                BISHOP => "b ",
                QUEEN => "q ",
                KING => "k ",
                _ => "  ",
            };
            if is_piece_white(piece) {
                return char.to_uppercase();
            }
            return char.to_string();
        }
        let mut result = String::new();
        for n in (0..64).rev() {
            //start from top right
            let col = 7 - n % 8;
            let row = n / 8;
            let string = piece_string_representation(self.board[row][col]);
            result = result + &*string;
            if n % 8 == 0 {
                result.push('\n');
            }
        }
        return result;
    }

    fn get_piece_at_position(&self, position: PositionVector) -> i32 {
        if position.row < 0 || position.col < 0 {
            panic!("Negative Indices getting piece at position {},{}", position.row, position.col);
        }
        return self.board[position.row as usize][position.col as usize];
    }

    fn set_piece_at_position(&self, position: PositionVector, new_value: i32) -> BoardState {
        if position.row < 0 || position.col < 0 {
            panic!("Negative Indices setting piece at position {},{}", position.row, position.col);
        }
        let mut new_board = self.board;
        new_board[position.row as usize][position.col as usize] = new_value;
        let new_state = BoardState {
            board: new_board,
            color_to_move: self.color_to_move,
            en_passant: self.en_passant,
            white_castling_rights: self.white_castling_rights,
            black_castling_rights: self.black_castling_rights,
        };
        return new_state;
    }

    fn get_piece_moves_disregarding_checks(&self, position: PositionVector) -> Vec<PositionVector> {
        let piece = self.get_piece_at_position(position);
        let color = piece & COLOR_MASK;
        let colorless_piece = piece & PIECE_MASK;
        let moves = match colorless_piece {
            PAWN => self.get_pawn_moves(position, piece),
            ROOK => self.get_rook_moves(position, piece),
            KNIGHT => self.get_knight_moves(position, piece),
            BISHOP => self.get_bishop_moves(position, piece),
            QUEEN => self.get_queen_moves(position, piece),
            KING => self.get_king_moves(position, piece),
            _ => Vec::new(),
        };
        return moves;
    }

    fn get_pawn_moves(&self, position: PositionVector, piece: i32) -> Vec<PositionVector> {
        let is_on_home_row = || -> bool {
            let color = piece & COLOR_MASK;
            let row = if color == WHITE { 1 } else { 6 };
            return position.row == row;
        };
        let move_direction: i32 = if is_piece_white(piece) { 1 } else { -1 };
        let mut moves = Vec::new();
        //if no piece is blocking the way, we can move forwards one square
        let new_move = PositionVector { row: move_direction * 1, col: 0 };
        if self.get_piece_at_position(position + new_move) == NOTHING {
            moves.push(PositionVector { row: move_direction * 1, col: 0 });
            //if the pawn is on its home row and the square two in front is also free, it can move two squares
            if is_on_home_row() && self.get_piece_at_position(PositionVector { row: position.row + move_direction * 2, col: position.col }) == NOTHING
            {
                moves.push(PositionVector { row: move_direction * 2, col: 0 });
            }
        }
        //check diagonal squares for taking a piece
        for i in [-1, 1] {
            let diagonal_square = PositionVector { row: move_direction * 1, col: i };
            let row = position.row + diagonal_square.row;
            let col = position.col + diagonal_square.col;
            //prevent checks outside of the board
            if is_move_out_of_bounds(position, diagonal_square) {
                continue;
            }
            let diagonal_square_piece = self.get_piece_at_position(PositionVector { row, col });
            if is_opposite_color(piece, diagonal_square_piece) || self.is_en_passant_field(PositionVector { row, col }) {
                moves.push(diagonal_square);
            }
        }
        return moves;
    }

    fn get_sliding_moves(&self, position: PositionVector, piece: i32, straight: bool) -> Vec<PositionVector> {
        let mut moves = Vec::new();
        for main_direction in [-1, 1] {
            for second_direction_toggle in [true, false] {
                let mut length = 1;
                loop {
                    let new_move = if straight {
                        if second_direction_toggle {
                            PositionVector { row: main_direction * length, col: 0 }
                        } else {
                            PositionVector { row: 0, col: main_direction * length }
                        }
                    } else {
                        if second_direction_toggle {
                            PositionVector { row: main_direction * length, col: length }
                        } else {
                            PositionVector { row: main_direction * length, col: -length }
                        }
                    };
                    if is_move_out_of_bounds(position, new_move) {
                        break;
                    }
                    let new_field = self.get_piece_at_position(position + new_move);
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

    fn get_rook_moves(&self, position: PositionVector, piece: i32) -> Vec<PositionVector> {
        return self.get_sliding_moves(position, piece, true);
    }

    fn get_knight_moves(&self, position: PositionVector, piece: i32) -> Vec<PositionVector> {
        let mut moves = Vec::new();
        for long_side in [-2, 2] {
            for short_side in [-1, 1] {
                for new_move in [PositionVector { row: long_side, col: short_side }, PositionVector { row: short_side, col: long_side }] {
                    if is_move_out_of_bounds(position, new_move) {
                        continue;
                    }
                    let other_square = self.get_piece_at_position(position + new_move);
                    if is_same_color(piece, other_square) {
                        continue;
                    }
                    moves.push(new_move);
                }
            }
        }
        return moves;
    }

    fn get_bishop_moves(&self, position: PositionVector, piece: i32) -> Vec<PositionVector> {
        return self.get_sliding_moves(position, piece, false);
    }

    fn get_queen_moves(&self, position: PositionVector, piece: i32) -> Vec<PositionVector> {
        let mut moves = self.get_sliding_moves(position, piece, true);
        moves.extend(self.get_sliding_moves(position, piece, false));
        return moves;
    }

    fn get_king_moves(&self, position: PositionVector, piece: i32) -> Vec<PositionVector> {
        let mut moves = Vec::new();
        //1 step in all 8 directions
        for row_step in [-1, 0, 1] {
            for col_step in [-1, 0, 1] {
                //this is the square on which the king is standing
                if row_step == 0 && col_step == 0 {
                    continue;
                }
                let new_move = PositionVector { row: row_step, col: col_step };
                if is_move_out_of_bounds(position, new_move) {
                    continue;
                }
                let new_square = self.get_piece_at_position(position + new_move);
                if is_same_color(piece, new_square) {
                    continue;
                }
                moves.push(new_move);
            }
        }
        //castling
        if (is_piece_white(piece) && self.white_castling_rights) || (!is_piece_white(piece) && self.black_castling_rights) {
            //short castle
            let one_square_left = self.get_piece_at_position(PositionVector { row: position.row, col: position.col - 1 });
            let two_squares_left = self.get_piece_at_position(PositionVector { row: position.row, col: position.col - 2 });
            if one_square_left == NOTHING && two_squares_left == NOTHING {
                moves.push(PositionVector { row: 0, col: -2 });
            }
            //long castle
            let one_square_right = self.get_piece_at_position(PositionVector { row: position.row, col: position.col + 1 });
            let two_squares_right = self.get_piece_at_position(PositionVector { row: position.row, col: position.col + 2 });
            let three_squares_right = self.get_piece_at_position(PositionVector { row: position.row, col: position.col + 3 });
            if one_square_right == NOTHING && two_squares_right == NOTHING && three_squares_right == NOTHING {
                moves.push(PositionVector { row: 0, col: 2 });
            }
        }
        return moves;
    }

    fn is_en_passant_field(&self, position: PositionVector) -> bool {
        return self.en_passant == position;
    }

    pub fn get_piece_moves_respecting_checks(&self, position: PositionVector) -> Vec<PositionVector> {
        let moves_disregarding_checks = self.get_piece_moves_disregarding_checks(position);
        let mut moves_respecting_checks = Vec::new();
        for move_to_check_for_check in moves_disregarding_checks {
            let new_state = self.perform_move(position, move_to_check_for_check);
            //todo implement castling checks checker
            if new_state.is_color_in_check(WHITE) || new_state.is_color_in_check(BLACK) {
                continue;
            }
            moves_respecting_checks.push(move_to_check_for_check);
        }
        todo!();
        return moves_respecting_checks;
    }

    pub fn is_color_in_check(&self, color: i32) -> bool {
        //find the king
        let mut king_position = INVALID_POSITION;
        'outer: for row in 0..8 {
            for col in 0..8 {
                let position = PositionVector { row, col };
                if self.get_piece_at_position(position) == KING + color {
                    king_position = position;
                    break 'outer;
                }
            }
        }
        assert_ne!(king_position, INVALID_POSITION);
        for row in 0..8 {
            for col in 0..8 {
                let position = PositionVector { row, col };
                let square = self.get_piece_at_position(position);
                if !is_same_color(square, color) {
                    for m in self.get_piece_moves_disregarding_checks(position) {
                        let piece_attack_square = m + position;
                        if piece_attack_square == king_position {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }

    fn perform_move(&self, position: PositionVector, move_to_perform: PositionVector) -> BoardState {
        if is_move_out_of_bounds(position, move_to_perform) {
            panic!("move not legal");
        }
        let piece = self.get_piece_at_position(position);
        let new_position = position + move_to_perform;

        todo!();
    }
}

fn is_piece_white(piece: i32) -> bool {
    let color = piece & COLOR_MASK;
    if color == WHITE {
        return true;
    }
    return false;
}

fn is_opposite_color(piece_1: i32, piece_2: i32) -> bool {
    let col_1 = piece_1 & COLOR_MASK;
    let col_2 = piece_2 & COLOR_MASK;
    return (col_1 == WHITE && col_2 == BLACK) || (col_1 == BLACK && col_2 == WHITE);
}

fn is_same_color(piece_1: i32, piece_2: i32) -> bool {
    if piece_1 == NOTHING || piece_2 == NOTHING {
        return false;
    }
    let col_1 = piece_1 & COLOR_MASK;
    let col_2 = piece_2 & COLOR_MASK;
    return col_1 == col_2;
}

fn is_move_out_of_bounds(position: PositionVector, relative_move: PositionVector) -> bool {
    let new_position = position + relative_move;
    if new_position.row < 0 || new_position.col < 0 || new_position.row > 7 || new_position.col > 7 {
        return true;
    }
    return false;
}
