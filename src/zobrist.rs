use crate::chess::Game;
use lazy_static::lazy_static;
use rand_xorshift::XorShiftRng;
use rand::{RngCore,  SeedableRng};
use bitintr::{ Tzcnt };

const SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
lazy_static! {
    pub static ref PIECE_SQUARE : [[u64; 64]; 12] = {
        let mut piece_square = [[0u64; 64]; 12];
        let mut rng = XorShiftRng::from_seed(SEED);
        for i in 0..12 {
            for k in 0..64 {
                piece_square[i][k] = rng.next_u64();
            }
        }
        piece_square
    };

    pub static ref CASTLING_RIGHT : [u64;4] = {
        let mut piece_square = [[0u64; 64]; 12];
        let mut castling_right = [0u64;4];
        let mut rng = XorShiftRng::from_seed(SEED);
        for i in 0..12 {
            for k in 0..64 {
                piece_square[i][k] = rng.next_u64();
            }
        }
        for i in 0..4 {
            castling_right[i] = rng.next_u64();
        }
        castling_right
    };
    
    pub static ref VALID_ENPASSANT : [u64; 8] = {
        let mut rng = XorShiftRng::from_seed(SEED);
        let mut castling_right = [0u64;4];
        let mut piece_square = [[0u64; 64]; 12];
        let mut valid_enpassant = [0u64; 8];
        for i in 0..12 {
            for k in 0..64 {
                piece_square[i][k] = rng.next_u64();
            }
        }
        for i in 0..4 {
            castling_right[i] = rng.next_u64();
        }
        for i in 0..8 {
            valid_enpassant[i] = rng.next_u64();
        }
        valid_enpassant
    };
    pub static ref  SIDETOMOVE : u64 = {
        let mut rng = XorShiftRng::from_seed(SEED);
        let mut castling_right = [0u64;4];
        let mut piece_square = [[0u64; 64]; 12];
        let mut valid_enpassant = [0u64; 8];
        
        for i in 0..12 {
            for k in 0..64 {
                piece_square[i][k] = rng.next_u64();
            }
        }
        for i in 0..4 {
            castling_right[i] = rng.next_u64();
        }
        for i in 0..8 {
            valid_enpassant[i] = rng.next_u64();
        }
        rng.next_u64()
    };
}


impl Game {
    pub fn as_array(&self) -> [u64; 12] {
        [self.wp, self.wn, self.wb, self.wr, self.wq, self.wk, self.bp, self.bn, self.bb, self.br, self.bq, self.bk]
    }
    pub fn castling_as_array(&self) -> [bool; 6] {
        [self.wking_never_move, self.wqueen_rook_never_move, self.wking_rook_never_move,
         self.bking_never_move, self.bqueen_rook_never_move, self.bking_rook_never_move]
    }
}
pub fn init_zobrist_key(game : &Game
    /*,piece_square : &mut [[u64;64]; 12], sidetomoveb: &mut u64, castling_right:&mut [u64;4],
    valid_enpassant : &mut [u64; 8],*/
) -> u64 {
    let mut zobrist_key = 0u64;
    let mut i = 0;
    for mut piece in game.as_array() {
        while piece != 0 {
            zobrist_key ^= PIECE_SQUARE[i][piece.tzcnt() as usize];
            piece &= piece-1;
        }
        i+=1;
    }
    i = 0;
    for right in game.castling_as_array() {
        if right {
            zobrist_key ^= CASTLING_RIGHT[i];
        }
    }
    zobrist_key ^= *SIDETOMOVE;
    zobrist_key
}