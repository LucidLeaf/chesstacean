const NOTHING: i32 = 0;
const PAWN: i32 = 1;
const ROOK: i32 = 2;
const KNIGHT: i32 = 3;
const BISHOP: i32 = 4;
const QUEEN: i32 = 5;
const KING: i32 = 6;

pub const WHITE: i32 = 8;
pub const BLACK: i32 = 16;

pub(crate) const INVALID_POSITION: Position = Position { row: 1234567890, col: 1234567890 };
const PIECE_MASK: i32 = 7;
const COLOR_MASK: i32 = 24;

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const E4_FEN: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

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
    white_short_castle: bool,
    white_long_castle: bool,
    black_short_castle: bool,
    black_long_castle: bool,
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
        let castling_string = &string_array[2];
        let white_short_castle = castling_string.contains("K");
        let white_long_castle = castling_string.contains("Q");
        let black_short_castle = castling_string.contains("k");
        let black_long_castle = castling_string.contains("q");
        let en_passant_square = notation_to_coordinates(string_array[3]);
        let half_move_clock: u32 = string_array[4].parse().expect("Not a valid clock");
        let full_move_clock: u32 = string_array[5].parse().expect("Not a valid clock");
        return BoardState {
            board,
            color_to_move,
            en_passant_square,
            white_short_castle,
            white_long_castle,
            black_short_castle,
            black_long_castle,
            half_move_clock,
            full_move_clock,
        };
    }

    pub fn new() -> BoardState {
        return BoardState::from_fen(STARTING_FEN);
    }

    pub fn str(&self) -> String {
        fn piece_char_representation(piece: i32) -> char {
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
        let mut result = String::new();
        for n in (0..64).rev() {
            // start from top left
            let col = 7 - n % 8;
            let row = n / 8;
            let piece_string = piece_char_representation(self.get_piece_at_position(Position { row, col }));
            result.push(piece_string);
            if n % 8 == 0 && n != 63 && n != 0 {
                result.push('\n');
            } else {
                result.push(' ');
            }
        }
        return result;
    }

    pub fn get_en_passant_square(&self) -> Position {
        return self.en_passant_square;
    }

    pub fn is_color_in_check(&self, color: i32) -> bool {
        for position_index in 0..64 {
            if self.board[position_index] == KING + color {
                let king_position = Position::position_from_indices(position_index);
                return self.is_position_attacked(king_position);
            }
        }
        panic!("King not found {}", self.str());
    }

    pub fn get_legal_moves(&self) {
        todo!();
    }

    pub fn get_piece_moves_respecting_checks(&self, position: Position) -> Vec<Position> {
        let piece = self.get_piece_at_position(position);
        //player turn
        if piece & COLOR_MASK != self.color_to_move {
            return Vec::new();
        }
        let moves_disregarding_checks = self.get_piece_moves_disregarding_checks(position);
        let mut moves_respecting_checks = Vec::new();
        for move_to_check_for_check in moves_disregarding_checks {
            if piece & PIECE_MASK == KING {
                //todo implement castling checks checker
            }
            let new_state = self.perform_move(position, move_to_check_for_check);
            if new_state.is_color_in_check(WHITE) || new_state.is_color_in_check(BLACK) {
                continue;
            }
            moves_respecting_checks.push(move_to_check_for_check);
        }
        return moves_respecting_checks;
    }

    /**
    move to perform from first position to second position
     */
    pub fn perform_move(&self, position: Position, new_position: Position) -> BoardState {
        let piece = self.get_piece_at_position(position);
        let mut piece_moved_state = self.set_piece_at_position(new_position, piece).set_piece_at_position(position, NOTHING);
        if new_position == self.en_passant_square {
            let enemy_pawn_position = Position { row: position.row, col: new_position.col };
            piece_moved_state = piece_moved_state.set_piece_at_position(enemy_pawn_position, NOTHING);
        }
        //todo castle both king and rook
        //todo change castling rights
        if piece & PAWN > 0 && (position.row - new_position.row).abs() == 2 {
            let en_passant_position = Position { row: (position.row + new_position.row) / 2, col: position.col };
            piece_moved_state.en_passant_square = en_passant_position;
        } else {
            piece_moved_state.en_passant_square = INVALID_POSITION;
        }
        piece_moved_state.color_to_move = if piece_moved_state.color_to_move == WHITE { BLACK } else { WHITE };
        return piece_moved_state;
    }

    /// returns piece integer at the given position, position must be in bounds
    fn get_piece_at_position(&self, position: Position) -> i32 {
        if position.row < 0 || position.col < 0 || position.row > 7 || position.col > 7 {
            panic!("Indices out of bounds getting piece at position {},{}", position.row, position.col);
        }
        return self.board[Position::index_from_position(position)];
    }

    fn set_piece_at_position(&self, position: Position, new_piece: i32) -> BoardState {
        if position.row < 0 || position.col < 0 {
            panic!("Negative Indices setting piece at position {},{}", position.row, position.col);
        }
        let mut new_board = self.board;
        new_board[Position::index_from_position(position)] = new_piece;
        let new_state = BoardState {
            board: new_board,
            color_to_move: self.color_to_move,
            en_passant_square: self.en_passant_square,
            white_short_castle: self.white_short_castle,
            white_long_castle: self.white_long_castle,
            black_short_castle: self.black_short_castle,
            black_long_castle: self.black_long_castle,
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
        let one_square_forward = Position { row: position.row + move_direction, col: position.col };
        if self.get_piece_at_position(one_square_forward) == NOTHING {
            moves.push(one_square_forward);
            //if the pawn is on its home row and the square two in front is also free, it can move two squares
            let two_squares_forward = Position { row: position.row + move_direction * 2, col: position.col };
            if is_on_home_row() && self.get_piece_at_position(two_squares_forward) == NOTHING {
                moves.push(two_squares_forward);
            }
        }
        //check diagonal squares for taking a piece
        for i in [-1, 1] {
            let diagonal_square = Position { row: position.row + move_direction, col: position.col + i };
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
                    Position { row: position.row + long_side, col: position.col + short_side },
                    Position { row: position.row + short_side, col: position.col + long_side },
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
                let new_position = Position { row: position.row + row_step, col: position.col + col_step };
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
        //short castle
        if (is_piece_white(king) && self.white_short_castle) || (!is_piece_white(king) && self.black_short_castle) {
            for i in [1, 2] {
                let position_i_squares_to_the_side = self.get_piece_at_position(Position { row: position.row, col: position.col + i });
                if position_i_squares_to_the_side != NOTHING {
                    continue;
                }
            }
            moves.push(Position { row: position.row, col: position.col + 2 });
        }
        //long castle
        if (is_piece_white(king) && self.white_long_castle) || (!is_piece_white(king) && self.black_long_castle) {
            for i in [1, 2, 3] {
                let position_i_squares_to_the_side = self.get_piece_at_position(Position { row: position.row, col: position.col - i });
                if position_i_squares_to_the_side != NOTHING {
                    continue;
                }
            }
            moves.push(Position { row: position.row, col: position.col - 2 });
        }
        return moves;
    }
}

fn fen_to_board(board_string: &&str) -> [i32; 64] {
    fn char_to_piece(piece: char) -> i32 {
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
    let mut rank = 7;
    let mut col = 0;
    let mut board = [0; 64];
    for ch in board_string.chars() {
        match ch {
            '1'..='8' => col = col + (ch.to_digit(10).expect("Invalid number parsing FEN") as i32),
            'r' | 'n' | 'b' | 'q' | 'k' | 'p' | 'R' | 'N' | 'B' | 'Q' | 'K' | 'P' => {
                let piece = char_to_piece(ch);
                let index = (rank * 8 + col) as usize;
                board[index] = piece;
            }
            '/' => {
                rank = rank - 1;
                col = 0;
                continue;
            }
            _ => panic!("Unknown fen char: {}", ch),
        }
        col = col + 1;
    }
    let final_board = board;
    return final_board;
}

pub fn notation_to_coordinates(input: &str) -> Position {
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

fn is_position_out_of_bounds(position: Position) -> bool {
    if position.row < 0 || position.col < 0 || position.row > 7 || position.col > 7 {
        return true;
    }
    return false;
}

fn is_index_out_of_bounds(index: i32) -> bool {
    return index < 0 || index > 63;
}
