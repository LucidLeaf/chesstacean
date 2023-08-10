const NOTHING: i32 = 0;
const PAWN: i32 = 1;
const ROOK: i32 = 2;
const KNIGHT: i32 = 3;
const BISHOP: i32 = 4;
const QUEEN: i32 = 5;
const KING: i32 = 6;

pub const WHITE: i32 = 8;
pub const BLACK: i32 = 16;

pub(crate) const INVALID_POSITION: Position = Position {
    row: 1234567890,
    col: 1234567890,
};
const PIECE_MASK: i32 = 7;
const COLOR_MASK: i32 = 24;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const E4_FEN: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
pub const CASTLING_TEST: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq e3 0 1";

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub(crate) row: i32,
    pub(crate) col: i32,
}

impl Position {
    pub fn index_from_position(pos: Position) -> usize {
        return (pos.row * 8 + pos.col) as usize;
    }

    pub fn position_from_indices(index: usize) -> Position {
        let col = (index % 8) as i32;
        let row = ((index - index % 8) / 8) as i32;
        return Position { row, col };
    }

    pub fn position_from_string(input: &str) -> Position {
        if input.len() != 2 {
            return INVALID_POSITION;
        }
        let mut formatted_input = input.to_lowercase();
        let column_string = formatted_input.remove(0);
        let row_string = formatted_input.remove(0);
        let col: i32 = (column_string.to_digit(20).expect("Not a row") - 10) as i32;
        let row: i32 = (row_string.to_digit(10).expect("Not a number") - 1) as i32;
        return Position { row, col };
    }

    pub fn str(&self) -> String {
        if self == &INVALID_POSITION {
            return String::from("");
        }
        let row = char::from_digit((self.row + 1) as u32, 10).expect("Invalid position");
        let col: char = char::from(97u8 + self.col as u8);
        let mut result = col.to_string();
        result.push(row);
        return result;
    }
}

impl PartialEq<Self> for Position {
    fn eq(&self, other: &Self) -> bool {
        self.col == other.col && self.row == other.row
    }
}

pub struct BoardState {
    // [0]=h1 [1]=g1... [63]=a8
    board: [i32; 64],
    color_to_move: i32,
    castling_rights: String,
    en_passant_square: Position,
    half_move_clock: u32,
    full_move_clock: u32,
}

impl BoardState {
    pub fn from_fen(fen: &str) -> BoardState {
        let split_string: Vec<&str> = fen.split(' ').collect();
        let mut string_array: [&str; 6] = [""; 6];
        for i in 0..split_string.len() {
            string_array[i] = split_string.get(i).expect("FEN invalid");
        }
        let board = fen_to_board(&string_array[0]);
        let color_to_move = if string_array[1] == "w" { WHITE } else { BLACK };
        let castling_rights = String::from(string_array[2]);
        let en_passant_square = Position::position_from_string(string_array[3]);
        let half_move_clock: u32 = string_array[4].parse().expect("Not a valid clock");
        let full_move_clock: u32 = string_array[5].parse().expect("Not a valid clock");
        return BoardState {
            board,
            color_to_move,
            castling_rights,
            en_passant_square,
            half_move_clock,
            full_move_clock,
        };
    }

    pub fn new() -> BoardState {
        return BoardState::from_fen(STARTING_FEN);
    }

    pub fn str(&self) -> String {
        let mut result = String::new();
        for n in (0..64).rev() {
            // start from top left
            let col = 7 - n % 8;
            let row = n / 8;
            let piece_string = char_from_piece(self.get_piece_at_position(Position { row, col }));
            result.push(piece_string);
            if n % 8 == 0 && n != 63 && n != 0 {
                result.push('\n');
            } else {
                result.push(' ');
            }
        }
        return result;
    }

    pub fn full_state_str(&self) -> String {
        let mut strings_to_insert: Vec<&str> = Vec::new();
        // color to move
        let color_string: &str = if self.color_to_move == WHITE { "white" } else { "black" };
        let color_notice = format!("\t\tTo move: {}\n", color_string);
        strings_to_insert.push(color_notice.as_str());
        // castling rights
        let castling_notice = format!("\t\tCastling rights: {}\n", self.castling_rights);
        strings_to_insert.push(&castling_notice);
        // en_passant_square
        let en_passant_notice = format!("\t\tEn-passant-square: {}\n", self.en_passant_square.str());
        strings_to_insert.push(&en_passant_notice);
        // half move clock
        let half_move_notice = format!("\t\tHalf move clock: {}\n", self.half_move_clock);
        strings_to_insert.push(&half_move_notice);
        // full move clock
        let full_move_notice = format!("\t\tFull move clock: {}\n", self.full_move_clock);
        strings_to_insert.push(&full_move_notice);

        let original_string = self.str();
        let mut lines: Vec<&str> = original_string.lines().collect();
        let mut result_string = String::new();
        for insert in strings_to_insert {
            let mut new_line = String::from(lines.remove(0));
            new_line.push_str(insert);
            result_string.push_str(new_line.as_str());
        }
        for line in lines {
            result_string.push_str(line);
            result_string.push('\n')
        }
        let final_string = result_string;
        return final_string;
    }

    pub fn to_fen(&self) -> String {
        todo!()
    }

    pub fn get_piece_moves_respecting_checks(&self, position: Position) -> Vec<Position> {
        let piece = self.get_piece_at_position(position);
        //player turn
        if piece & COLOR_MASK != self.color_to_move {
            return Vec::new();
        }
        let moves_disregarding_checks = self.get_piece_moves_disregarding_checks(position);
        let mut moves_respecting_checks = Vec::new();
        'next_move: for move_to_check_for_check in moves_disregarding_checks {
            if piece & PIECE_MASK == KING && (move_to_check_for_check.col - position.col).abs() > 1 {
                for col_in_between in position.col..move_to_check_for_check.col {
                    let position_king_moves_through = Position {
                        row: position.row,
                        col: col_in_between,
                    };
                    let new_state = self.perform_move(position_king_moves_through, move_to_check_for_check);
                    if new_state.is_color_in_check(self.color_to_move) {
                        continue 'next_move;
                    }
                }
            }
            let new_state = self.perform_move(position, move_to_check_for_check);
            if new_state.is_color_in_check(self.color_to_move) {
                continue;
            }
            moves_respecting_checks.push(move_to_check_for_check);
        }
        return moves_respecting_checks;
    }

    /**
    move piece to new position, allows illegal moves
     */
    pub fn perform_move(&self, position: Position, new_position: Position) -> BoardState {
        let piece = self.get_piece_at_position(position);
        let mut new_state = self.set_piece_at_position(new_position, piece).set_piece_at_position(position, NOTHING);
        if new_position == self.en_passant_square {
            let enemy_pawn_position = Position {
                row: position.row,
                col: new_position.col,
            };
            new_state = new_state.set_piece_at_position(enemy_pawn_position, NOTHING);
        }
        // increment move counter after blacks turn
        if self.color_to_move == BLACK {
            new_state.full_move_clock = self.full_move_clock + 1
        }
        //todo change half move counter
        if piece & PIECE_MASK == KING {
            //castle both king and rook
            if new_position.col - position.col == 2 {
                //king side
                new_state = new_state.set_piece_at_position(Position { row: position.row, col: 7 }, NOTHING);
                new_state = new_state.set_piece_at_position(Position { row: position.row, col: 5 }, ROOK | (piece & COLOR_MASK));
            }
            if new_position.col - position.col == -2 {
                //queen side
                new_state = new_state.set_piece_at_position(Position { row: position.row, col: 0 }, NOTHING);
                new_state = new_state.set_piece_at_position(Position { row: position.row, col: 3 }, ROOK | (piece & COLOR_MASK));
            }
            // remove castling rights
            let castling_chars = if self.color_to_move == WHITE { ["K", "Q"] } else { ["k", "q"] };
            for char in castling_chars {
                new_state.castling_rights = new_state.castling_rights.replace(char, "");
            }
        }
        //remove castling rights for respective side
        if piece & PIECE_MASK == ROOK {
            if position.row == 0 && position.col == 0 {
                new_state.castling_rights = self.castling_rights.replace("Q", "");
            }
            if position.row == 0 && position.col == 7 {
                new_state.castling_rights = self.castling_rights.replace("K", "");
            }
            if position.row == 7 && position.col == 0 {
                new_state.castling_rights = self.castling_rights.replace("q", "");
            }
            if position.row == 7 && position.col == 7 {
                new_state.castling_rights = new_state.castling_rights.replace("k", "");
            }
        }
        // set en-passant square
        if piece & PAWN > 0 && (position.row - new_position.row).abs() == 2 {
            let en_passant_position = Position {
                row: (position.row + new_position.row) / 2,
                col: position.col,
            };
            new_state.en_passant_square = en_passant_position;
        } else {
            new_state.en_passant_square = INVALID_POSITION;
        }
        //change turn
        new_state.color_to_move = if self.color_to_move == WHITE { BLACK } else { WHITE };
        return new_state;
    }

    pub fn get_legal_moves(&self) -> Vec<(Position, Position)> {
        let mut moves: Vec<(Position, Position)> = Vec::new();
        for square in 0..64 {
            if self.board[square] & COLOR_MASK != self.color_to_move {
                continue;
            }
            let piece_position = Position::position_from_indices(square);
            let target_squares = self.get_piece_moves_respecting_checks(piece_position);
            for target_square in target_squares {
                let move_tuple = (piece_position, target_square);
                moves.push(move_tuple);
            }
        }
        return moves;
    }

    pub fn is_checkmate(&self) -> bool {
        return self.get_legal_moves().len() == 0 && self.is_color_in_check(self.color_to_move);
    }

    fn is_color_in_check(&self, color: i32) -> bool {
        for position_index in 0..64 {
            if self.board[position_index] == KING + color {
                let king_position = Position::position_from_indices(position_index);
                return self.is_position_attacked(king_position);
            }
        }
        panic!("King not found \n{}", self.str());
    }

    /// returns piece integer at the given position, position must be in bounds
    fn get_piece_at_position(&self, position: Position) -> i32 {
        return self.board[Position::index_from_position(position)];
    }

    fn set_piece_at_position(&self, position: Position, new_piece: i32) -> BoardState {
        let mut new_board = self.board;
        new_board[Position::index_from_position(position)] = new_piece;
        let new_state = BoardState {
            board: new_board,
            color_to_move: self.color_to_move,
            en_passant_square: self.en_passant_square,
            castling_rights: String::from(&self.castling_rights),
            half_move_clock: self.half_move_clock,
            full_move_clock: self.full_move_clock,
        };
        return new_state;
    }

    fn is_position_attacked(&self, position_to_be_checked: Position) -> bool {
        let color = COLOR_MASK & self.get_piece_at_position(position_to_be_checked);
        for row in 0..8 {
            for col in 0..8 {
                let iterator_position = Position { row, col };
                let square = self.get_piece_at_position(iterator_position);
                if is_same_color(square, color) {
                    continue;
                }
                for attacked_square in self.get_piece_moves_disregarding_checks(iterator_position) {
                    if attacked_square == position_to_be_checked {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    fn get_piece_moves_disregarding_checks(&self, position: Position) -> Vec<Position> {
        let piece = self.get_piece_at_position(position);
        let colorless_piece = piece & PIECE_MASK;
        let moves = match colorless_piece {
            PAWN => self.get_pawn_moves(position),
            ROOK => self.get_rook_moves(position),
            KNIGHT => self.get_knight_moves(position),
            BISHOP => self.get_bishop_moves(position),
            QUEEN => self.get_queen_moves(position),
            KING => self.get_king_moves(position),
            _ => Vec::new(),
        };
        return moves;
    }

    fn get_pawn_moves(&self, position: Position) -> Vec<Position> {
        let piece = self.get_piece_at_position(position);
        let is_en_passant_square = |pos: Position| -> bool {
            return self.en_passant_square == pos;
        };
        let is_on_home_row = || -> bool {
            let color = piece & COLOR_MASK;
            let row = if color == WHITE { 1 } else { 6 };
            return position.row == row;
        };
        let move_direction: i32 = if is_piece_white(piece) { 1 } else { -1 };
        let mut moves = Vec::new();
        //if no piece is blocking the way, we can move forwards one square
        let one_square_forward = Position {
            row: position.row + move_direction,
            col: position.col,
        };
        if self.get_piece_at_position(one_square_forward) == NOTHING {
            moves.push(one_square_forward);
            //if the pawn is on its home row and the square two in front is also free, it can move two squares
            let two_squares_forward = Position {
                row: position.row + move_direction * 2,
                col: position.col,
            };
            if is_on_home_row() && self.get_piece_at_position(two_squares_forward) == NOTHING {
                moves.push(two_squares_forward);
            }
        }
        //check diagonal squares for taking a piece
        for i in [-1, 1] {
            let diagonal_square = Position {
                row: position.row + move_direction,
                col: position.col + i,
            };
            //prevent checks outside of the board
            if is_position_out_of_bounds(diagonal_square) {
                continue;
            }
            let diagonal_square_piece = self.get_piece_at_position(diagonal_square);
            if is_opposite_color(piece, diagonal_square_piece) || is_en_passant_square(diagonal_square) {
                moves.push(diagonal_square);
            }
        }
        return moves;
    }

    fn get_knight_moves(&self, position: Position) -> Vec<Position> {
        let piece = self.get_piece_at_position(position);
        let mut moves = Vec::new();
        for long_side in [-2, 2] {
            for short_side in [-1, 1] {
                for new_position in [
                    Position {
                        row: position.row + long_side,
                        col: position.col + short_side,
                    },
                    Position {
                        row: position.row + short_side,
                        col: position.col + long_side,
                    },
                ] {
                    if is_position_out_of_bounds(new_position) {
                        continue;
                    }
                    let other_square = self.get_piece_at_position(new_position);
                    if is_same_color(piece, other_square) {
                        continue;
                    }
                    moves.push(new_position);
                }
            }
        }
        return moves;
    }

    fn get_sliding_moves(&self, position: Position, diagonal: bool) -> Vec<Position> {
        let piece: i32 = self.get_piece_at_position(position);
        let mut moves: Vec<Position> = Vec::new();
        let current_position_index: i32 = Position::index_from_position(position) as i32;
        //array directions lead to more readable code (as of my abilities)
        let directions: [i32; 4] = if diagonal { [7, 9, -9, -7] } else { [1, 8, -1, -8] };
        for direction in directions {
            let mut previous_position_checked = position;
            let mut length = 1;
            loop {
                let new_index: i32 = current_position_index + direction * length;
                if is_index_out_of_bounds(new_index) {
                    break;
                }
                let new_position: Position = Position::position_from_indices(new_index as usize);
                //check whether the move ran over border
                if (new_position.row - previous_position_checked.row).abs() > 1 || (new_position.col - previous_position_checked.col).abs() > 1 {
                    break;
                }
                let piece_at_new_position: i32 = self.get_piece_at_position(new_position);
                if is_same_color(piece, piece_at_new_position) {
                    break;
                }
                moves.push(new_position);
                if is_opposite_color(piece, piece_at_new_position) {
                    break;
                }
                length = length + 1;
                previous_position_checked = new_position;
            }
        }
        return moves;
    }

    fn get_rook_moves(&self, position: Position) -> Vec<Position> {
        return self.get_sliding_moves(position, false);
    }

    fn get_bishop_moves(&self, position: Position) -> Vec<Position> {
        return self.get_sliding_moves(position, true);
    }

    fn get_queen_moves(&self, position: Position) -> Vec<Position> {
        let mut moves = self.get_sliding_moves(position, true);
        moves.extend(self.get_sliding_moves(position, false));
        return moves;
    }

    fn get_king_moves(&self, position: Position) -> Vec<Position> {
        let king = self.get_piece_at_position(position);
        let mut moves = Vec::new();
        //1 step in all 8 directions
        for row_step in [-1, 0, 1] {
            for col_step in [-1, 0, 1] {
                //this is the square on which the king is standing
                if row_step == 0 && col_step == 0 {
                    continue;
                }
                let new_position = Position {
                    row: position.row + row_step,
                    col: position.col + col_step,
                };
                if is_position_out_of_bounds(new_position) {
                    continue;
                }
                let new_square = self.get_piece_at_position(new_position);
                if is_same_color(king, new_square) {
                    continue;
                }
                moves.push(new_position);
            }
        }
        //castling
        let mut directions_to_check: Vec<Vec<i32>> = Vec::new();
        if (is_piece_white(king) && self.castling_rights.contains("K")) || (is_piece_black(king) && self.castling_rights.contains("k")) {
            let short_directions = vec![1, 2];
            directions_to_check.push(short_directions);
        }
        if (is_piece_white(king) && self.castling_rights.contains("Q")) || (is_piece_black(king) && self.castling_rights.contains("q")) {
            let long_directions = vec![-1, -2, -3];
            directions_to_check.push(long_directions);
        }
        'castling_direction: for castling_direction in directions_to_check {
            let mut direction_sign = 0;
            for column in castling_direction {
                direction_sign = column.signum();
                let position_i_squares_to_the_side = self.get_piece_at_position(Position {
                    row: position.row,
                    col: position.col + column,
                });
                if position_i_squares_to_the_side != NOTHING {
                    break 'castling_direction;
                }
            }
            moves.push(Position {
                row: position.row,
                col: position.col + direction_sign * 2,
            });
        }

        return moves;
    }
}

fn char_from_piece(piece: i32) -> char {
    let colorless_piece = piece & PIECE_MASK;
    let mut char: char = match colorless_piece {
        PAWN => 'p',
        ROOK => 'r',
        KNIGHT => 'n',
        BISHOP => 'b',
        QUEEN => 'q',
        KING => 'k',
        _ => ' ',
    };
    if is_piece_white(piece) {
        char = char.to_uppercase().to_string().remove(0);
    }
    return char;
}

fn piece_from_char(piece: char) -> i32 {
    let color_mask = if piece.is_uppercase() { WHITE } else { BLACK };
    return color_mask
        | match piece.to_ascii_lowercase() {
            'r' => ROOK,
            'n' => KNIGHT,
            'b' => BISHOP,
            'q' => QUEEN,
            'k' => KING,
            'p' => PAWN,
            _ => 0,
        };
}

fn fen_to_board(board_string: &&str) -> [i32; 64] {
    let mut rank = 7;
    let mut col = 0;
    let mut board = [0; 64];
    for ch in board_string.chars() {
        match ch {
            '1'..='8' => col = col + (ch.to_digit(10).expect("Invalid number parsing FEN") as i32),
            'r' | 'n' | 'b' | 'q' | 'k' | 'p' | 'R' | 'N' | 'B' | 'Q' | 'K' | 'P' => {
                let piece = piece_from_char(ch);
                let index = (rank * 8 + col) as usize;
                board[index] = piece;
                col = col + 1;
            }
            '/' => {
                rank = rank - 1;
                col = 0;
            }
            _ => println!("Unknown fen char: {}", ch),
        }
    }
    let final_board = board;
    return final_board;
}

fn is_piece_white(piece: i32) -> bool {
    let color = piece & COLOR_MASK;
    return color == WHITE;
}

fn is_piece_black(piece: i32) -> bool {
    let color = piece & COLOR_MASK;
    return color == BLACK;
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

fn is_position_out_of_bounds(position: Position) -> bool {
    if position.row < 0 || position.col < 0 || position.row > 7 || position.col > 7 {
        return true;
    }
    return false;
}

fn is_index_out_of_bounds(index: i32) -> bool {
    return index < 0 || index > 63;
}
