use lazy_static::lazy_static;
use bitintr::{ Tzcnt, Blsr };//, Lzcnt, Andn};
use std::collections::VecDeque;
use crate::zobrist::*;
mod zobrist;


static BASICSTART_CHESS_BOARD:[[char;8];8] = [
    ['r','n','b','q','k','b','n','r'],
    ['p','p','p','p','p','p','p','p'],
    [' ',' ',' ',' ',' ',' ',' ',' '],
    [' ',' ',' ',' ',' ',' ',' ',' '],
    [' ',' ',' ',' ',' ',' ',' ',' '],
    [' ',' ',' ',' ',' ',' ',' ',' '],
    ['P','P','P','P','P','P','P','P'],
    ['R','N','B','Q','K','B','N','R'],
];

#[derive(Debug, Copy, Clone)]
pub enum Piece {
    NONE,
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
    KING
}
#[derive(Clone, Copy)]
pub struct Game {
    pub wp : u64, pub wn : u64, pub wb : u64, pub wr : u64, pub wq : u64, pub wk : u64,
    pub bp : u64, pub bn : u64, pub bb : u64, pub br : u64, pub bq : u64, pub bk : u64,
    pub white_to_play : bool,
    pub wking_rook_never_move : bool,
    pub wqueen_rook_never_move : bool,
    pub wking_never_move : bool,
    pub bking_rook_never_move : bool,
    pub bqueen_rook_never_move : bool,
    pub bking_never_move : bool,
    pub nb_coups : u16,
    pub hash : u64,
}
impl Game {
    pub fn occupied(&self) -> u64 {
        self.wp | self.wn | self.wb | self.wr | self.wq | self.wk | self.bp | self.bn | self.bb | self.br | self.bq | self.bk
    }
    pub fn white(&self) -> u64 {
        self.wp | self.wn | self.wb | self.wr | self.wq | self.wk
    }
    pub fn black(&self) -> u64 {
        self.bp | self.bn | self.bb | self.br | self.bq | self.bk
    }
    
}
impl Default for Game {
    fn default() -> Self { 
        get_game_from_basicpos()
    }
}

pub fn convert_square_to_move(a_move : u64) -> String{
    let b = (a_move / 8) as u8;
    let a:u8 = (a_move % 8) as u8;
    let f = (b'a' + a ) as char;
    let mut a = String::from(f);
    a.push((48 + b+1 ) as char );
    a
}

pub static RANK_MASK : [u64;8] = [
    0x00000000000000FF,
    0x000000000000FF00,
    0x0000000000FF0000, 
    0x00000000FF000000,
    0x000000FF00000000,
    0x0000FF0000000000,
    0x00FF000000000000,
    0xFF00000000000000
];
static FILE_MASKS : [u64;8] = [
    0x101010101010101, 0x202020202020202, 0x404040404040404, 0x808080808080808,
    0x1010101010101010, 0x2020202020202020, 0x4040404040404040, 0x8080808080808080
];

static DIAG_MASKS : [u64;15] = [
    0x1, 0x102, 0x10204, 0x1020408, 0x102040810, 0x10204081020, 0x1020408102040,
	0x102040810204080, 0x204081020408000, 0x408102040800000, 0x810204080000000,
	0x1020408000000000, 0x2040800000000000, 0x4080000000000000, 0x8000000000000000
];
static ANTIDIAG_MASKS : [u64;15] = [
    0x80, 0x8040, 0x804020, 0x80402010, 0x8040201008, 0x804020100804, 0x80402010080402,
	0x8040201008040201, 0x4020100804020100, 0x2010080402010000, 0x1008040201000000,
	0x804020100000000, 0x402010000000000, 0x201000000000000, 0x100000000000000
];


pub static mg_pawn_table : [i32 ; 64] = [
      0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
];

pub static eg_pawn_table : [i32;64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];
pub static  mg_knight_table :[i32;64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

pub static eg_knight_table : [i32;64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

pub static KNIGHT_POS_SCORE : [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

pub static KING_POS_MIDDLE : [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    20, 20,  0,  0,  0,  0, 20, 20,
    20, 30, 10,  0,  0, 10, 30, 20
];
pub static KING_POS_END : [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50
];

//pub static SQUARE_CENTER : u64 = 103481868288;
pub static SQUARE_CENTER : u64 = 0x1818000000;

lazy_static! {
    static ref FIRST_RANK_ATTACKS: [[u64; 8]; 64] = {
        let mut first_rank_attacks = [[0; 8]; 64];
        for o in 0..64 {
            for f in 0..8 {
                first_rank_attacks[o][f] = 0;

                for i in (f + 1)..8 {
                    first_rank_attacks[o][f] |= 1 << i;
                    if (o << 1) & (1 << i) > 0 {
                        break;
                    }
                }
                for i in (0..f).rev() {
                    first_rank_attacks[o][f] |= 1 << i;
                    if (o << 1) & (1 << i) > 0 {
                        break;
                    }
                }
            }
        }
        first_rank_attacks
    };

    static ref KING_MOVE : [u64;64] = {
        let mut king_moves = [0u64;64];
        for x in 0u64..64 {
            king_moves[x as usize] = possibility_k(1u64<<x);
        }
        king_moves
    };
    static ref KNIGHT_MOVE : [u64;64] = {
        let mut knight_moves = [0u64;64];
        for x in 0u64..64 {
            knight_moves[x as usize] = possibility_n(1u64<<x);
        }
        knight_moves
    };
    
}

#[allow(clippy::too_many_arguments)]
pub fn array_to_bitboard(chessboard : [[char;8]; 8], wp:&mut u64, wn:&mut u64, wb:&mut u64, wr:&mut u64, wq:&mut u64, wk:&mut u64, bp:&mut u64, bn:&mut u64, bb:&mut u64, br:&mut u64, bq:&mut u64, bk:&mut u64) {
    let mut i = 0;
    for v in chessboard {
        for c in v {
            match c {
                'p' => { *wp += convert_string_to_bitboard(i); },
                'n' => { *wn += convert_string_to_bitboard(i); },
                'b' => { *wb += convert_string_to_bitboard(i); },
                'r' => { *wr += convert_string_to_bitboard(i); },
                'q' => { *wq += convert_string_to_bitboard(i); },
                'k' => { *wk += convert_string_to_bitboard(i); },
                'P' => { *bp += convert_string_to_bitboard(i); },
                'N' => { *bn += convert_string_to_bitboard(i); },
                'B' => { *bb += convert_string_to_bitboard(i); },
                'R' => { *br += convert_string_to_bitboard(i); },
                'Q' => { *bq += convert_string_to_bitboard(i); },
                'K' => { *bk += convert_string_to_bitboard(i); },
                _ => {}
            }
            i+=1;
        }
    }
}
pub fn draw_the_game_state(game : &Game) {
    eprintln!("GAME STATE");
    eprintln!("WPAWN");
    _draw_bitboard(game.wp);
    eprintln!("WKNIGHT");
    _draw_bitboard(game.wn);
    eprintln!("WBISHOP");
    _draw_bitboard(game.wb);
    eprintln!("WROOK");
    _draw_bitboard(game.wr);
    eprintln!("WQUEEN");
    _draw_bitboard(game.wq);
    eprintln!("WKING");
    _draw_bitboard(game.wk);
    eprintln!("BPAWN");
    _draw_bitboard(game.bp);
    eprintln!("BKNIGHT");
    _draw_bitboard(game.bn);
    eprintln!("BBISHOP");
    _draw_bitboard(game.bb);
    eprintln!("BROOK");
    _draw_bitboard(game.br);
    eprintln!("BQUEEN");
    _draw_bitboard(game.bq);
    eprintln!("BKING");
    _draw_bitboard(game.bk);
}
pub fn get_game_from_basicpos() -> Game {
    let mut wp : u64 = 0;
    let mut wn : u64 = 0;
    let mut wb : u64 = 0;
    let mut wr : u64 = 0;
    let mut wq : u64 = 0;
    let mut wk : u64 = 0;
    let mut bp : u64 = 0;
    let mut bn : u64 = 0;
    let mut bb : u64 = 0;
    let mut br : u64 = 0;
    let mut bq : u64 = 0;
    let mut bk : u64 = 0;

    array_to_bitboard(BASICSTART_CHESS_BOARD, &mut wp, &mut wn, &mut wb, &mut wr, &mut wq, &mut wk, &mut bp, &mut bn, &mut bb, &mut br, &mut bq, &mut bk);
    
    let mut game = Game {
        wp, wn, wb, wr, wq, wk,
        bp, bn, bb, br, bq, bk,
        white_to_play : true,
        wking_never_move : true, wqueen_rook_never_move : true, wking_rook_never_move : true,
        bking_never_move : true, bqueen_rook_never_move : true, bking_rook_never_move : true,
        nb_coups : 0,
        hash : 0
    };
    game.hash = init_zobrist_key(&game);
    game
}
pub fn _draw_bitboard(bitboard : u64) {
    println!();
    let mut i = 0;
    for _k in 0..8 {
        println!();
        for _p in 0..8 {
            print!("{}", bitboard>>i & 1);
            i+=1;
        }
    }
    println!();
}
pub fn _count_bit(mut bit : u64) -> i8 {
    let mut count = 0;
    while bit != 0 {
        bit &= bit-1;
        count+=1;
    }
    count
}
pub fn _draw_board(game : &Game) {
    let mut chess_board:[[char;8];8] = [[' ';8];8];
    let mut i = 0;
    for x in &mut chess_board {
        for c in x {
            if ((game.wp >> i) & 1) == 1  { *c = 'P'; }
            if ((game.wn >> i) & 1) == 1  { *c = 'N'; }
            if ((game.wb >> i) & 1) == 1  { *c = 'B'; }
            if ((game.wr >> i) & 1) == 1  { *c = 'R'; }
            if ((game.wq >> i) & 1) == 1  { *c = 'Q'; }
            if ((game.wk >> i) & 1) == 1  { *c = 'K'; }
            if ((game.bp >> i) & 1) == 1  { *c = 'p'; }
            if ((game.bn >> i) & 1) == 1  { *c = 'n'; }
            if ((game.bb >> i) & 1) == 1  { *c = 'b'; }
            if ((game.br >> i) & 1) == 1  { *c = 'r'; }
            if ((game.bq >> i) & 1) == 1  { *c = 'q'; }
            if ((game.bk >> i) & 1) == 1  { *c = 'k'; }
            i+=1;
        }
    }
    let letter = 'a';
    print!("     ");
    for i in 0..8 {
        print!("  {} ", (letter as u8+i) as char);
    }
    println!();
    
    for (i, x) in chess_board.iter().enumerate() {
        println!("     ---------------------------------");
        print!("   {} ", i+1);
        for c in x {
            print!("| {c} ");
        }
        println!("|");
    }
    println!("     ---------------------------------");
}
pub fn convert_string_to_bitboard(binary:usize) -> u64 {
    //u64::pow(2, (binary) as u32)
    1<<binary
}
pub fn possibility_wp(wpawn : u64, empty : u64, black : u64) -> u64 {
    let pmoves1 = (wpawn & !FILE_MASKS[0])<<7 & black;// & !RANK_MASK[7] ;
    let pmoves2 = (wpawn & !FILE_MASKS[7])<<9 & black;// & !RANK_MASK[7] ;
    let pmoves3 = wpawn<<8 & empty;// & !RANK_MASK[7];
    let pmoves4 = wpawn<<16 & empty & (empty<<8) & RANK_MASK[3];
    pmoves1 | pmoves2 | pmoves3 | pmoves4
}
pub fn possibility_bp2( bpawn: u64, empty : u64, white : u64) -> u64 {

    let pmoves1 = (bpawn & !FILE_MASKS[7])>>7 & white;// & !RANK_MASK[0] ;
    let pmoves2 = (bpawn & !FILE_MASKS[0])>>9 & white;// & !RANK_MASK[0] ;
    let pmoves3 = bpawn>>8 & empty;// & !RANK_MASK[0];
    let pmoves4 = bpawn>>16 & empty & (empty>>8) & RANK_MASK[4];
    pmoves1 | pmoves2 | pmoves3 | pmoves4
}
pub fn attack_wp(wpawn : u64, white : u64) -> u64 {
    let pmoves1 = wpawn<<7 & white;// & !FILE_MASKS[0];
    let pmoves2 = wpawn<<9 & white;// & !FILE_MASKS[7];
    pmoves1 | pmoves2
}
pub fn attack_bp(bpawn : u64, black : u64) -> u64 {
    let pmoves1 = bpawn>>7 & black;// & !FILE_MASKS[0];
    let pmoves2 = bpawn>>9 & black;// & !FILE_MASKS[7];
    pmoves1 | pmoves2
}
pub fn possibility_n(knight : u64) -> u64 {
    let nonoea:u64 =  (knight << 17) & !FILE_MASKS[0];
    let noeaea:u64 =  (knight << 10) & !(FILE_MASKS[0] |  FILE_MASKS[1]);
    let soeaea:u64 =  (knight >>  6) & !(FILE_MASKS[0] | FILE_MASKS[1]);
    let sosoea:u64 =  (knight >> 15) & !FILE_MASKS[0];
    let nonowe:u64 =  (knight << 15) & !FILE_MASKS[7];
    let nowewe:u64 =  (knight <<  6) & !(FILE_MASKS[6] | FILE_MASKS[7]);
    let sowewe:u64 =  (knight >> 10) & !(FILE_MASKS[6] | FILE_MASKS[7]);
    let sosowe:u64 =  (knight >> 17) & !FILE_MASKS[7];
    nonoea | noeaea | soeaea | sosoea | nonowe | nowewe | sowewe | sosowe
}
/*
pub fn possibility_k(mut wk : u64) -> u64 {
    let mut attack = wk<<1 | wk>>1;
    wk |= attack;
    attack |= wk<<8 | wk>>8;
    attack
}*/
pub fn possibility_k(wk : u64) -> u64 {
    let mut attack = (wk & !FILE_MASKS[7])<<1 | (wk & !FILE_MASKS[0])>>1;
    attack |= (wk & !FILE_MASKS[7])<<9 | (wk & !FILE_MASKS[7])>>7;
    attack |= (wk & !FILE_MASKS[0])>>9 | (wk & !FILE_MASKS[0])<<7;
    attack |= wk<<8 | wk>>8;
    attack
}
pub fn hyperbola_quintessence(occupied : u64, mask: u64, mut number : u64) -> u64 {
    number = 1<<number;
    let mut forward = occupied & mask ;
    let mut reverse = forward.swap_bytes();

    forward = forward.wrapping_sub(number.wrapping_mul(2));
    reverse = reverse.wrapping_sub(number.swap_bytes().wrapping_mul(2));
    forward ^= reverse.swap_bytes();
    forward & mask
    //( - 2 * number) ^ ((occupied & mask).swap_bytes() - 2 * number.swap_bytes()).swap_bytes()
    //(occupied - 2 * number) ^ (occupied.reverse_bits() - 2 * number.reverse_bits()).reverse_bits()
}
pub fn rank_attacks(occupied: u64, sq: u64) -> u64 {
    let f = sq & 7; // sq.file() as Bitboard;
    let r = sq & !7; // (sq.rank() * 8) as Bitboard;
    let o = (occupied >> (r + 1)) & 63;
    FIRST_RANK_ATTACKS[o as usize][f as usize] << r
}
pub fn convert_move_to_bitboard(moves : &str) -> (u64, u64, Piece) {
    let mut length = 4;

    if moves.len() == 5 {
        length = 5;
    }
    let mut iter1 = moves[0..length].chars();
    let un = iter1.next().unwrap() as u64-96;
    let deux = iter1.next().unwrap() as u64-48;
    let trois = iter1.next().unwrap() as u64-96;
    let quatre = iter1.next().unwrap() as u64-48;
    let a = (deux-1) *8 +  un-1 ;
    let b = (quatre-1) *8 +  trois-1;
    let mut promotion_piece = Piece::NONE;
    if moves.len() == 5 {
        let promote = iter1.next().unwrap();
        promotion_piece = match promote {
            'q' => Piece::QUEEN ,
            'r' => Piece::ROOK,
            'b' => Piece::BISHOP,
            'n' => Piece::KNIGHT,
             _  => Piece::NONE
        }
    }
    (a,b, promotion_piece)
}
pub fn diag_antid_moves(square : u64, occupied : u64) -> u64 {
    hyperbola_quintessence(occupied, DIAG_MASKS[((square/8) + (square%8)) as usize], square) | hyperbola_quintessence(occupied, ANTIDIAG_MASKS[((square/8)+7 - (square%8)) as usize], square)
}
pub fn hv_moves(square : u64, occupied : u64) -> u64 {
    let b = hyperbola_quintessence(occupied, FILE_MASKS[(square % 8) as usize], square);
    rank_attacks(occupied, square) | b
}
pub fn compute_move_w(chessmove : (u64, u64, Piece), game : &mut Game) -> i8 {
    let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    let occupied = black | white;
    let mut a = chessmove.0;
    let mut b = chessmove.1;
    let square_a = a;
    let square_b = b;
    a = 1u64<<a;
    b = 1u64<<b;
    game.nb_coups+=1;
    let mut moves= 0;
    let mut from: &mut u64 = &mut 0;
    if (game.wp & a) != 0 {
        moves = possibility_wp(a, !occupied, black);
        if moves & b != 0 && b & RANK_MASK[7] != 0 {
            game.wp &= !a;
            match chessmove.2 {
                Piece::QUEEN  => game.wq |= b,
                Piece::ROOK   => game.wr |= b,
                Piece::BISHOP => game.wb |= b,
                Piece::KNIGHT => game.wn |= b,
                _ => { game.wp |= b; }
            }
            if black & b != 0 {
                if game.bp & b != 0 {  game.bp &= !b; return 1;}
                else if game.bn & b != 0 { game.bn &= !b; return 3;}
                else if game.bb & b != 0 { game.bb &= !b; return 3;}
                else if game.br & b != 0 { game.br &= !b; return 5;}
                else if game.bq & b != 0 { game.bq &= !b; return 11;}
            }
            return 1;
        }
        from = &mut game.wp;
    }
    else if game.wn & a != 0 {
        //moves = possibility_n(game.wn & a) & !white;
        moves = KNIGHT_MOVE[square_a as usize] & !white;
        from = &mut game.wn;
    }
    else if game.wb & a != 0 {
        let occupied = black | white;
        moves = diag_antid_moves(square_a, occupied) & !white;
        from = &mut game.wb;
    }
    else if game.wr & a != 0 {
        let occupied = black | white;
        moves = hv_moves(square_a, occupied) & !white;
        from = &mut game.wr;
        if moves & b != 0 {
            game.wking_rook_never_move = false;
        }
    }
    else if game.wq & a != 0 {
        let occupied = black | white;
        moves = (hv_moves(square_a, occupied) | diag_antid_moves(square_a, occupied)) & !white;
        
        from = &mut game.wq;
    }
    else if game.wk & a != 0 {
        //println!("{square_b} {} {} {}", game.wking_never_move, game.wking_rook_never_move, game.wqueen_rook_never_move);
        if square_b == 2 && square_a == 4 { // Grand roque
            //check if the king and the rook has never move
            if game.wking_never_move && game.wking_rook_never_move && (black | white) & (2u64.pow(1) + 2u64.pow(2)) == 0 && possibility_b(game) & (2u64.pow(1) + 2u64.pow(2)) == 0 {
                game.wking_never_move = false;
                game.wking_rook_never_move = false;
                //Do grand roque
                game.wk &= !a;
                game.wk |= b;
                game.wr &= !(2u64.pow(0));
                game.wr |= 2u64.pow(3);
                return 0;
            }
            return -1;
            //check if no piece is between
            //check if square between isn't attacked
        }
        else if square_b == 6  && square_a == 4 { //Petit Roque
            if game.wking_never_move && game.wqueen_rook_never_move && (black | white) & (2u64.pow(6) + 2u64.pow(5)) == 0 && possibility_b(game) & (2u64.pow(6) + 2u64.pow(5)) == 0 {
                game.wking_never_move = false;
                game.wqueen_rook_never_move = false;
                game.wk &= !a;
                game.wk |= b;
                game.wr &= !(2u64.pow(7));
                game.wr |= 2u64.pow(5);
                return 0;
            }
            return -1;
        }
        moves = KING_MOVE[square_a as usize] & !white;
        //moves = possibility_k(game.wk) & !white;
        from = &mut game.wk;
        if moves & b != 0 {
            game.wking_never_move = false;
        }
    }
    if moves & b != 0 {
        (*from) &= !a;
        (*from) |= b;
        if black & b != 0 {
            if game.bp & b != 0 {  game.bp &= !b; return 1;}
            else if game.bn & b != 0 { game.bn &= !b; return 3;}
            else if game.bb & b != 0 { game.bb &= !b; return 3;}
            else if game.br & b != 0 { game.br &= !b; return 5;}
            else if game.bq & b != 0 { game.bq &= !b; return 11;}
        }
        0
    }
    else {
        -1
    }
}
pub fn compute_move_w_thrust(chessmove : (u64, u64, Piece), game : &mut Game) -> i8 {
    //let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    //let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    //let occupied = black | white;
    let mut a = chessmove.0;
    let mut b = chessmove.1;
    let square_a = a;
    let square_b = b;
    a = 1u64<<a;
    b = 1u64<<b;
    game.nb_coups+=1;
    //let mut moves= 0;
    let mut from: &mut u64 = &mut 0;
    if (game.wp & a) != 0 {
        //moves = possibility_wp(a, !occupied, black);
        if /*moves & b != 0 &&*/ b & RANK_MASK[7] != 0 {
            game.wp &= !a;
            match chessmove.2 {
                Piece::QUEEN  => game.wq |= b,
                Piece::ROOK   => game.wr |= b,
                Piece::BISHOP => game.wb |= b,
                Piece::KNIGHT => game.wn |= b,
                _ => { game.wp |= b; }
            }
            //if black & b != 0 {
                if game.bp & b != 0 {  game.bp &= !b; return 1;}
                else if game.bn & b != 0 { game.bn &= !b; return 3;}
                else if game.bb & b != 0 { game.bb &= !b; return 3;}
                else if game.br & b != 0 { game.br &= !b; return 5;}
                else if game.bq & b != 0 { game.bq &= !b; return 11;}
            //}
            return 1;
        }
        from = &mut game.wp;
    }
    else if game.wn & a != 0 {
        //moves = possibility_n(game.wn & a) & !white;
        //moves = KNIGHT_MOVE[square_a as usize] & !white;
        from = &mut game.wn;
    }
    else if game.wb & a != 0 {
        //let occupied = black | white;
        //moves = diag_antid_moves(square_a, occupied) & !white;
        from = &mut game.wb;
    }
    else if game.wr & a != 0 {
        //let occupied = black | white;
        //moves = hv_moves(square_a, occupied) & !white;
        from = &mut game.wr;
        //if moves & b != 0 {
            game.wking_rook_never_move = false;
        //}
    }
    else if game.wq & a != 0 {
        //let occupied = black | white;
        //moves = (hv_moves(square_a, occupied) | diag_antid_moves(square_a, occupied)) & !white;
        
        from = &mut game.wq;
    }
    else if game.wk & a != 0 {
        //println!("{square_b} {} {} {}", game.wking_never_move, game.wking_rook_never_move, game.wqueen_rook_never_move);
        if square_b == 2 && square_a == 4 { // Grand roque
            //check if the king and the rook has never move
            //if game.wking_never_move && game.wking_rook_never_move && (black | white) & (2u64.pow(1) + 2u64.pow(2)) == 0 && possibility_b(game) & (2u64.pow(1) + 2u64.pow(2)) == 0 {
                game.wking_never_move = false;
                game.wking_rook_never_move = false;
                //Do grand roque
                game.wk &= !a;
                game.wk |= b;
                game.wr &= !(2u64.pow(0));
                game.wr |= 2u64.pow(3);
                return 0;
            //}
            //return -1;
            //check if no piece is between
            //check if square between isn't attacked
        }
        else if square_b == 6  && square_a == 4 { //Petit Roque
            //if game.wking_never_move && game.wqueen_rook_never_move && (black | white) & (2u64.pow(6) + 2u64.pow(5)) == 0 && possibility_b(game) & (2u64.pow(6) + 2u64.pow(5)) == 0 {
                game.wking_never_move = false;
                game.wqueen_rook_never_move = false;
                game.wk &= !a;
                game.wk |= b;
                game.wr &= !(2u64.pow(7));
                game.wr |= 2u64.pow(5);
                return 0;
            //}
            //return -1;
        }
        //moves = KING_MOVE[square_a as usize] & !white;
        //moves = possibility_k(game.wk) & !white;
        from = &mut game.wk;
        //if moves & b != 0 {
            game.wking_never_move = false;
        //}
    }
    //if moves & b != 0 {
        (*from) &= !a;
        (*from) |= b;
        //if black & b != 0 {
            if game.bp & b != 0 {  game.bp &= !b; return 1;}
            else if game.bn & b != 0 { game.bn &= !b; return 3;}
            else if game.bb & b != 0 { game.bb &= !b; return 3;}
            else if game.br & b != 0 { game.br &= !b; return 5;}
            else if game.bq & b != 0 { game.bq &= !b; return 11;}
        //}
        0
    /*}
    else {
        -1
    }*/
}
pub fn compute_move_w_hash (chessmove : (u64, u64, Piece), game : &mut Game) -> i8 {
    /*let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;*/
    //let occupied = black | white;
    let mut a = chessmove.0;
    let mut b = chessmove.1;
    let square_a = a;
    let square_b = b;
    a = 1u64<<a;
    b = 1u64<<b;
    game.nb_coups+=1;
    //let mut moves= 0;
    let mut from: &mut u64 = &mut 0;
    game.hash ^= *SIDETOMOVE;
    if (game.wp & a) != 0 {
        //println!("old : {} {}", game.hash, PIECE_SQUARE[0][square_a as usize]);
        game.hash ^= PIECE_SQUARE[0][square_a as usize];
        game.hash ^= PIECE_SQUARE[0][square_b as usize];
        //println!("new : {} {}", game.hash, PIECE_SQUARE[0][square_b as usize]);
        //moves = possibility_wp(a, !occupied, black);
        if /*moves & b != 0 &&*/ b & RANK_MASK[7] != 0 {
            game.wp &= !a;
            match chessmove.2 {
                Piece::QUEEN  => game.wq |= b,
                Piece::ROOK   => game.wr |= b,
                Piece::BISHOP => game.wb |= b,
                Piece::KNIGHT => game.wn |= b,
                _ => { game.wp |= b; }
            }
            //if black & b != 0 {
                if game.bp & b != 0 {  game.bp &= !b; return 1;}
                else if game.bn & b != 0 { game.bn &= !b; return 3;}
                else if game.bb & b != 0 { game.bb &= !b; return 3;}
                else if game.br & b != 0 { game.br &= !b; return 5;}
                else if game.bq & b != 0 { game.bq &= !b; return 11;}
            //}
            return 1;
        }
        from = &mut game.wp;
    }
    else if game.wn & a != 0 {
        game.hash ^= PIECE_SQUARE[1][square_a as usize];
        game.hash ^= PIECE_SQUARE[1][square_b as usize];
        //moves = possibility_n(game.wn & a) & !white;
        //moves = KNIGHT_MOVE[square_a as usize] & !white;
        from = &mut game.wn;
    }
    else if game.wb & a != 0 {
        game.hash ^= PIECE_SQUARE[2][square_a as usize];
        game.hash ^= PIECE_SQUARE[2][square_b as usize];
        //let occupied = black | white;
        //moves = diag_antid_moves(square_a, occupied) & !white;
        from = &mut game.wb;
    }
    else if game.wr & a != 0 {
        game.hash ^= PIECE_SQUARE[3][square_a as usize];
        game.hash ^= PIECE_SQUARE[3][square_b as usize];
        //let occupied = black | white;
        //moves = hv_moves(square_a, occupied) & !white;
        from = &mut game.wr;
        //if moves & b != 0 {
            game.wking_rook_never_move = false;
        //}
    }
    else if game.wq & a != 0 {
        game.hash ^= PIECE_SQUARE[4][square_a as usize];
        game.hash ^= PIECE_SQUARE[4][square_b as usize];
        //let occupied = black | white;
        //moves = (hv_moves(square_a, occupied) | diag_antid_moves(square_a, occupied)) & !white;
        
        from = &mut game.wq;
    }
    else if game.wk & a != 0 {
        game.hash ^= PIECE_SQUARE[5][square_a as usize];
        game.hash ^= PIECE_SQUARE[5][square_b as usize];
        //println!("{square_b} {} {} {}", game.wking_never_move, game.wking_rook_never_move, game.wqueen_rook_never_move);
        if square_b == 2 && square_a == 4 { // Grand roque
            //check if the king and the rook has never move
            //if game.wking_never_move && game.wking_rook_never_move && (black | white) & (2u64.pow(1) + 2u64.pow(2)) == 0 && possibility_b(game) & (2u64.pow(1) + 2u64.pow(2)) == 0 {
                game.wking_never_move = false;
                game.wking_rook_never_move = false;
                //Do grand roque
                game.wk &= !a;
                game.wk |= b;
                game.wr &= !(2u64.pow(0));
                game.wr |= 2u64.pow(3);
                return 0;
            //}
            //return -1;
            //check if no piece is between
            //check if square between isn't attacked
        }
        else if square_b == 6  && square_a == 4 { //Petit Roque
            //if game.wking_never_move && game.wqueen_rook_never_move && (black | white) & (2u64.pow(6) + 2u64.pow(5)) == 0 && possibility_b(game) & (2u64.pow(6) + 2u64.pow(5)) == 0 {
                game.wking_never_move = false;
                game.wqueen_rook_never_move = false;
                game.wk &= !a;
                game.wk |= b;
                game.wr &= !(2u64.pow(7));
                game.wr |= 2u64.pow(5);
                return 0;
            //}
            //return -1;
        }
        //moves = KING_MOVE[square_a as usize] & !white;
        //moves = possibility_k(game.wk) & !white;
        from = &mut game.wk;
        //if moves & b != 0 {
            game.wking_never_move = false;
        //}
    }
    //if moves & b != 0 {
        (*from) &= !a;
        (*from) |= b;
        //if black & b != 0 {
            if game.bp & b != 0 {
                game.hash ^= PIECE_SQUARE[6][square_b as usize]; 
                game.bp &= !b; return 1;
            }
            else if game.bn & b != 0 { 
                game.hash ^= PIECE_SQUARE[7][square_b as usize];
                game.bn &= !b; return 3;}
            else if game.bb & b != 0 { 
                game.hash ^= PIECE_SQUARE[8][square_b as usize];
                game.bb &= !b; return 3;}
            else if game.br & b != 0 { 
                game.hash ^= PIECE_SQUARE[9][square_b as usize];
                game.br &= !b; return 5;}
            else if game.bq & b != 0 { 
                game.hash ^= PIECE_SQUARE[10][square_b as usize];
                game.bq &= !b; return 11;}
        //}
        0
    /*}
    else {
        -1
    }*/
}

pub fn compute_move_b_hash(chessmove : (u64,u64,Piece), game :&mut Game) -> i8 {
    let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    let mut a = chessmove.0;
    let mut b = chessmove.1;
    let square_a = a;
    let square_b = b;
    a = 1<<a;
    b = 1<<b;
    game.nb_coups+=1;
    game.hash ^= *SIDETOMOVE;
    //let mut moves = 0;
    let mut from = &mut (0);
    if (game.bp & a) != 0 {
        game.hash ^= PIECE_SQUARE[6][square_a as usize];
        game.hash ^= PIECE_SQUARE[6][square_b as usize];
        //moves = possibility_bp2(a, !(black | white), white);
        if /*moves & b != 0 &&*/ b & RANK_MASK[0] != 0 {
            game.bp &= !(1u64<<square_a);
            match chessmove.2 {
                Piece::QUEEN  => game.bq |= 1u64<<square_b,
                Piece::ROOK   => game.br |= 1u64<<square_b,
                Piece::BISHOP => game.bb |= 1u64<<square_b,
                Piece::KNIGHT => game.bn |= 1u64<<square_b,
                _ => { game.bp |= 1u64<<square_b; }
            }
            if white & b != 0 {
                if game.wp & b != 0 { game.wp &= !b; return 1;}
                else if game.wn & b != 0 { game.wn &= !b; return 3;}
                else if game.wb & b != 0 { game.wb &= !b; return 3;}
                else if game.wr & b != 0 { game.wr &= !b; return 5;}
                else if game.wq & b != 0 { game.wq &= !b; return 11;}
            }
            return 1;
        }
        from = &mut game.bp;
    }
    else if game.bn & a != 0 {
        game.hash ^= PIECE_SQUARE[7][square_a as usize];
        game.hash ^= PIECE_SQUARE[7][square_b as usize];
        //moves = possibility_n( a) & !black;
        //moves = KNIGHT_MOVE[square_a as usize] & !black;
        from = &mut game.bn;
    }
    else if game.bb & a != 0 {
        game.hash ^= PIECE_SQUARE[8][square_a as usize];
        game.hash ^= PIECE_SQUARE[8][square_b as usize];
        let occupied = black | white;
        //moves = diag_antid_moves(square_a, occupied) & !black;
        from = &mut game.bb;
    }
    else if game.br & a != 0 {
        game.hash ^= PIECE_SQUARE[9][square_a as usize];
        game.hash ^= PIECE_SQUARE[9][square_b as usize];
        let occupied = black | white;
        //moves = hv_moves(square_a, occupied) & !black;
        from = &mut game.br;
    }
    else if game.bq & a != 0 {
        game.hash ^= PIECE_SQUARE[10][square_a as usize];
        game.hash ^= PIECE_SQUARE[10][square_b as usize];
        let occupied = black | white;
        //moves = (hv_moves(square_a, occupied) | diag_antid_moves(square_a, occupied)) & !black;
        from = &mut game.bq;
    }
    else if game.bk & a != 0 {
        game.hash ^= PIECE_SQUARE[11][square_a as usize];
        game.hash ^= PIECE_SQUARE[11][square_b as usize];
        if square_a == 60 && square_b == 58 && game.bking_never_move && game.bking_rook_never_move && (black | white) & (2u64.pow(58) + 2u64.pow(57)) == 0 && possibility_w(game) & (2u64.pow(58) + 2u64.pow(57)) == 0 {
                //println!("Grand roque");
                game.bking_never_move = false;
                game.bking_rook_never_move = false;
                //Do grand roque
                game.bk &= !a;
                game.bk |= b;
                game.br &= !(2u64.pow(56));
                game.br |= 2u64.pow(59);
                return 0;
        }
            //check if no piece is between
            //check if square between isn't attacked
        
        else if square_a == 60 && square_b == 62  && game.bking_never_move && game.bqueen_rook_never_move && (black | white) & (2u64.pow(61) + 2u64.pow(62)) == 0 && possibility_w(game) & (2u64.pow(61) + 2u64.pow(62)) == 0 {
                game.bking_never_move = false;
                game.bqueen_rook_never_move = false;
                game.bk &= !a;
                game.bk |= b;
                game.br &= !(2u64.pow(63));
                game.br |= 2u64.pow(61);
                return 0;
            
        }
        //moves = KING_MOVE[square_a as usize] & !black;
        //moves = possibility_k(game.bk) & !black;
        from = &mut game.bk;
        //if moves & b != 0 {
            game.bking_never_move = false;
        //}
    }
    //if moves & b != 0 {
        (*from) &= !a;
        (*from) |=  b;
        if white & b != 0 {
            if game.wp & b != 0 {
                game.hash ^= PIECE_SQUARE[0][square_b as usize];
                game.wp &= !b; return 1;}
            else if game.wn & b != 0 { 
                game.hash ^= PIECE_SQUARE[1][square_b as usize];
                game.wn &= !b; return 3;}
            else if game.wb & b != 0 { 
                game.hash ^= PIECE_SQUARE[2][square_b as usize];
                game.wb &= !b; return 3;}
            else if game.wr & b != 0 { 
                game.hash ^= PIECE_SQUARE[3][square_b as usize];
                game.wr &= !b; return 5;}
            else if game.wq & b != 0 {
                game.hash ^= PIECE_SQUARE[4][square_b as usize];
                game.wq &= !b; return 11;}
        }
        0
    /*}
    else { -1 }*/
}
pub fn compute_move_b(chessmove : (u64,u64,Piece), game :&mut Game) -> i8 {
    let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    let mut a = chessmove.0;
    let mut b = chessmove.1;
    let square_a = a;
    let square_b = b;
    a = 1<<a;
    b = 1<<b;
    game.nb_coups+=1;
    let mut moves = 0;
    let mut from = &mut (0);
    if (game.bp & a) != 0 {
        moves = possibility_bp2(a, !(black | white), white);
        if moves & b != 0 && b & RANK_MASK[0] != 0 {
            game.bp &= !(1u64<<square_a);
            match chessmove.2 {
                Piece::QUEEN  => game.bq |= 1u64<<square_b,
                Piece::ROOK   => game.br |= 1u64<<square_b,
                Piece::BISHOP => game.bb |= 1u64<<square_b,
                Piece::KNIGHT => game.bn |= 1u64<<square_b,
                _ => { game.bp |= 1u64<<square_b; }
            }
            if white & b != 0 {
                if game.wp & b != 0 { game.wp &= !b; return 1;}
                else if game.wn & b != 0 { game.wn &= !b; return 3;}
                else if game.wb & b != 0 { game.wb &= !b; return 3;}
                else if game.wr & b != 0 { game.wr &= !b; return 5;}
                else if game.wq & b != 0 { game.wq &= !b; return 11;}
            }
            return 1;
        }
        from = &mut game.bp;
    }
    else if game.bn & a != 0 {
        //moves = possibility_n( a) & !black;
        moves = KNIGHT_MOVE[square_a as usize] & !black;
        from = &mut game.bn;
    }
    else if game.bb & a != 0 {
        let occupied = black | white;
        moves = diag_antid_moves(square_a, occupied) & !black;
        from = &mut game.bb;
    }
    else if game.br & a != 0 {
        let occupied = black | white;
        moves = hv_moves(square_a, occupied) & !black;
        from = &mut game.br;
    }
    else if game.bq & a != 0 {
        let occupied = black | white;
        moves = (hv_moves(square_a, occupied) | diag_antid_moves(square_a, occupied)) & !black;
        from = &mut game.bq;
    }
    else if game.bk & a != 0 {
        //println!("{square_b} {} {} {}", game.bking_never_move, (black | white) & (2u64.pow(61) + 2u64.pow(62)) == 0, possibility_w(game) & (2u64.pow(61) + 2u64.pow(62)) == 0);
        
        if square_a == 60 && square_b == 58 && game.bking_never_move && game.bking_rook_never_move && (black | white) & (2u64.pow(58) + 2u64.pow(57)) == 0 && possibility_w(game) & (2u64.pow(58) + 2u64.pow(57)) == 0 {
                //println!("Grand roque");
                game.bking_never_move = false;
                game.bking_rook_never_move = false;
                //Do grand roque
                game.bk &= !a;
                game.bk |= b;
                game.br &= !(2u64.pow(56));
                game.br |= 2u64.pow(59);
                return 0;
        }
            //check if no piece is between
            //check if square between isn't attacked
        
        else if square_a == 60 && square_b == 62  && game.bking_never_move && game.bqueen_rook_never_move && (black | white) & (2u64.pow(61) + 2u64.pow(62)) == 0 && possibility_w(game) & (2u64.pow(61) + 2u64.pow(62)) == 0 {
                game.bking_never_move = false;
                game.bqueen_rook_never_move = false;
                game.bk &= !a;
                game.bk |= b;
                game.br &= !(2u64.pow(63));
                game.br |= 2u64.pow(61);
                return 0;
            
        }
        moves = KING_MOVE[square_a as usize] & !black;
        //moves = possibility_k(game.bk) & !black;
        from = &mut game.bk;
        if moves & b != 0 {
            game.bking_never_move = false;
        }
    }
    if moves & b != 0 {
        (*from) &= !a;
        (*from) |=  b;
        if white & b != 0 {
            if game.wp & b != 0 { game.wp &= !b; return 1;}
            else if game.wn & b != 0 { game.wn &= !b; return 3;}
            else if game.wb & b != 0 { game.wb &= !b; return 3;}
            else if game.wr & b != 0 { game.wr &= !b; return 5;}
            else if game.wq & b != 0 { game.wq &= !b; return 11;}
        }
        0
    }
    else { -1 }
}
pub fn compute_move_b_thrust(chessmove : (u64,u64,Piece), game :&mut Game) -> i8 {
    //let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    //let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    let mut a = chessmove.0;
    let mut b = chessmove.1;
    let square_a = a;
    let square_b = b;
    a = 1<<a;
    b = 1<<b;
    game.nb_coups+=1;
    //let mut moves = 0;
    let mut from = &mut (0);
    if (game.bp & a) != 0 {
        //moves = possibility_bp2(a, !(black | white), white);
        if /*moves & b != 0 &&*/ b & RANK_MASK[0] != 0 {
            game.bp &= !(1u64<<square_a);
            match chessmove.2 {
                Piece::QUEEN  => game.bq |= 1u64<<square_b,
                Piece::ROOK   => game.br |= 1u64<<square_b,
                Piece::BISHOP => game.bb |= 1u64<<square_b,
                Piece::KNIGHT => game.bn |= 1u64<<square_b,
                _ => { game.bp |= 1u64<<square_b; }
            }
            //if white & b != 0 {
                if game.wp & b != 0 { game.wp &= !b; return 1;}
                else if game.wn & b != 0 { game.wn &= !b; return 3;}
                else if game.wb & b != 0 { game.wb &= !b; return 3;}
                else if game.wr & b != 0 { game.wr &= !b; return 5;}
                else if game.wq & b != 0 { game.wq &= !b; return 11;}
            //}
            //return 1;
        }
        from = &mut game.bp;
    }
    else if game.bn & a != 0 {
        //moves = possibility_n( a) & !black;
        //moves = KNIGHT_MOVE[square_a as usize] & !black;
        from = &mut game.bn;
    }
    else if game.bb & a != 0 {
        //let occupied = black | white;
        //moves = diag_antid_moves(square_a, occupied) & !black;
        from = &mut game.bb;
    }
    else if game.br & a != 0 {
        //let occupied = black | white;
        //moves = hv_moves(square_a, occupied) & !black;
        from = &mut game.br;
    }
    else if game.bq & a != 0 {
        //let occupied = black | white;
        //moves = (hv_moves(square_a, occupied) | diag_antid_moves(square_a, occupied)) & !black;
        from = &mut game.bq;
    }
    else if game.bk & a != 0 {
        //println!("{square_b} {} {} {}", game.bking_never_move, (black | white) & (2u64.pow(61) + 2u64.pow(62)) == 0, possibility_w(game) & (2u64.pow(61) + 2u64.pow(62)) == 0);
        
        if square_a == 60 && square_b == 58 /*&& game.bking_never_move && game.bking_rook_never_move && (black | white) & (2u64.pow(58) + 2u64.pow(57)) == 0 && possibility_w(game) & (2u64.pow(58) + 2u64.pow(57)) == 0*/ {
                //println!("Grand roque");
                game.bking_never_move = false;
                game.bking_rook_never_move = false;
                //Do grand roque
                game.bk &= !a;
                game.bk |= b;
                game.br &= !(2u64.pow(56));
                game.br |= 2u64.pow(59);
                return 0;
        }
            //check if no piece is between
            //check if square between isn't attacked
        
        else if square_a == 60 && square_b == 62  /*&& game.bking_never_move && game.bqueen_rook_never_move && (black | white) & (2u64.pow(61) + 2u64.pow(62)) == 0 && possibility_w(game) & (2u64.pow(61) + 2u64.pow(62)) == 0*/ {
                game.bking_never_move = false;
                game.bqueen_rook_never_move = false;
                game.bk &= !a;
                game.bk |= b;
                game.br &= !(2u64.pow(63));
                game.br |= 2u64.pow(61);
                return 0;
            
        }
        //moves = KING_MOVE[square_a as usize] & !black;
        //moves = possibility_k(game.bk) & !black;
        from = &mut game.bk;
        //if moves & b != 0 {
            game.bking_never_move = false;
        //}
    }
    //if moves & b != 0 {
        (*from) &= !a;
        (*from) |=  b;
        //if white & b != 0 {
            if game.wp & b != 0 { game.wp &= !b; return 1;}
            else if game.wn & b != 0 { game.wn &= !b; return 3;}
            else if game.wb & b != 0 { game.wb &= !b; return 3;}
            else if game.wr & b != 0 { game.wr &= !b; return 5;}
            else if game.wq & b != 0 { game.wq &= !b; return 11;}
        //}
        0
    //} else { -1 }
}
/*
pub fn is_white_attack_at(game : &Game) -> bool {

}*/
pub fn possibility_w( game : &Game) -> u64 {
    let black = game.black();
    let white = game.white();
    let occupied = black | white;
    let mut attack = 0;
    attack |= attack_wp(game.wp, black);
    let mut copy_wn = game.wn;
    while copy_wn != 0 {
        attack |= KNIGHT_MOVE[copy_wn.tzcnt() as usize] & !white;
        copy_wn &= copy_wn-1;
    }
    /*f game.wn != 0 {
        attack |= possibility_n(game.wn) & !white;
    }*/
    let mut copy_wb = game.wb;
    while copy_wb != 0 {
        attack |= diag_antid_moves(copy_wb.tzcnt() , occupied) & !white;
        copy_wb &= copy_wb-1;
    }
    let mut copy_wr = game.wr;
    while copy_wr != 0 {
        attack |= hv_moves(copy_wr.tzcnt() , occupied) & !white;
        copy_wr &= copy_wr-1;
    }
    let mut copy_wq = game.wq;
    while copy_wq != 0 {
        attack |= (hv_moves(copy_wq.tzcnt(), occupied) | diag_antid_moves(copy_wq.tzcnt(), occupied)) & !white;
        copy_wq &= copy_wq-1;
    }
    //attack |= possibility_k(game.wk) & !white;
    attack |= KING_MOVE[game.wk.tzcnt() as usize];
    
    attack
}
pub fn possibility_b( game : &Game) -> u64 {
    let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    let occupied = black | white;
    let mut attack = 0;

    attack |= attack_bp(game.bp, white);

    if game.bn != 0 {
        attack |= KNIGHT_MOVE[game.bn.tzcnt() as usize] & !black;
        //attack |= possibility_n(game.bn) & !black;
    }
    let mut copy_bb = game.bb;
    while copy_bb != 0 {
        attack |= diag_antid_moves(copy_bb.tzcnt() , occupied) & !black;
        copy_bb &= copy_bb-1;
    }
    let mut copy_br = game.br;
    while copy_br != 0 {
        attack |= hv_moves(copy_br.tzcnt(), occupied) & !black;
        copy_br &= copy_br-1;
    }
    let mut copy_bq = game.bq;
    while copy_bq != 0 {
        attack |= (hv_moves(copy_bq.tzcnt(), occupied) | diag_antid_moves(copy_bq.tzcnt(), occupied) ) & !black;
        copy_bq &= copy_bq-1;
    }
    attack |= KING_MOVE[game.bk.tzcnt() as usize];
    //attack |= possibility_k(game.bk) & !black;
    attack
}
fn is_attacked_by_slider_b (game : &Game, square : u64) -> bool {
    let occupied = game.occupied();
    let mut copy_bb = game.bb;
    while copy_bb != 0 {
        if diag_antid_moves(copy_bb.tzcnt() , occupied) & square != 0{
            return true;
        };
        copy_bb &= copy_bb-1;
    }
    let mut copy_br = game.br;
    while copy_br != 0 {
        if hv_moves(copy_br.tzcnt(), occupied) & square != 0 {
            return true;
        };
        copy_br &= copy_br-1;
    }
    let mut copy_bq = game.bq;
    while copy_bq != 0 {
        if hv_moves(copy_bq.tzcnt(), occupied) & square != 0 || diag_antid_moves(copy_bq.tzcnt(), occupied)  & square != 0 {
            return true;
        };
        copy_bq &= copy_bq-1;
    }
    false

}
fn is_attacked_by_slider_w (game : &Game, square : u64) -> bool {
    let occupied = game.occupied();
    let mut copy_wb = game.wb;
    while copy_wb != 0 {
        if diag_antid_moves(copy_wb.tzcnt() , occupied) & square != 0 {
            return true;
        };
        copy_wb &= copy_wb-1;
    }
    let mut copy_wr = game.wr;
    while copy_wr != 0 {
        if hv_moves(copy_wr.tzcnt(), occupied) & square != 0 {
            return true;
        };
        copy_wr &= copy_wr-1;
    }
    let mut copy_wq = game.wq;
    while copy_wq != 0 {
        if hv_moves(copy_wq.tzcnt(), occupied) & square != 0 || diag_antid_moves(copy_wq.tzcnt(), occupied)  & square != 0 {
            return true;
        };
        copy_wq &= copy_wq-1;
    }
    false

}
fn attack_normal_piece_b(game : &Game, black : u64) -> u64 {
    let black = game.black();
    let k = game.bk.tzcnt();
    let mut attack = KING_MOVE[k as usize];
    attack |= KNIGHT_MOVE[game.bn.tzcnt() as usize];
    attack |= attack_bp(game.bp, black);
    attack & black
}
fn attack_normal_piece_w(game : &Game, white : u64) -> u64 {
    let white = game.white();
    let k = game.wk.tzcnt();
    let mut attack = KING_MOVE[k as usize];
    attack |= KNIGHT_MOVE[game.wn.tzcnt() as usize];
    attack |= attack_wp(game.wp, white);
    attack & white
}
pub fn is_attacked(target_is_wking : bool, game : &Game) -> bool {
    if target_is_wking {
        possibility_b(game) & game.wk != 0
    }
    else {
        possibility_w(game) & game.bk != 0
    }
}

pub fn get_legal_move(side_w : bool, game : &Game) -> VecDeque<(u64, Piece)> {
    //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
    let black = game.bp | game.bn | game.bb | game.br | game.bq | game.bk;
    let white = game.wp | game.wn | game.wb | game.wr | game.wq | game.wk;
    let occupied = black | white;
    let mut legal_moves = VecDeque::<(u64, Piece)>::new();
    
    if side_w { //White Possibility
        //Pions Possibility
        //let black_normal = attack_normal_piece_b(&game, black);
        //let k = game.wk.tzcnt();
        let mut wp_test = game.wp;
        while  wp_test != 0 {
            let piece = wp_test.tzcnt();
            let wp_extract = 1u64 << piece;
            wp_test = wp_test & (wp_test-1);
            let mut possi_wp = possibility_wp(wp_extract, !(occupied), black);
            while possi_wp != 0 {
                let mut promote = 0;
                let mut promote_piece = Piece::NONE;
                let b = possi_wp.tzcnt();
                //println!("hello white {} {} {}", b, RANK_MASK[0] ,b & RANK_MASK[0]);
                if 1<<b & RANK_MASK[7] != 0 {
                    promote = 1;
                    promote_piece = Piece::QUEEN;
                }
                let mut game1 = *game;
                let b = possi_wp.tzcnt();
                let capture = compute_move_w_thrust((piece, b, promote_piece), &mut game1);
                //let is_check = is_attacked_by_slider_w(&game1, k) &&  black_normal & k != 0;
                let is_check = is_attacked(true, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece<<9) + (b<<1) + promote, Piece::PAWN));
                    }
                    else {
                        legal_moves.push_back(((piece<<9) + (b<<1) + promote, Piece::PAWN));
                    }
                }
                possi_wp = possi_wp.blsr();
                //possi_wp = possi_wp & (possi_wp - 1);
            }
        }
        //Knight
        //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
        let mut wn_test = game.wn;
        while wn_test != 0 {
            let piece = wn_test.tzcnt();
            let wn_extract = 1u64 << piece;
            wn_test = wn_test & (wn_test - 1);
            let mut wn_possi = possibility_n( wn_extract) & !white;
            while wn_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = wn_possi.tzcnt();
                let capture = compute_move_w_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(true, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece<<9) + (b<<1), Piece::KNIGHT));
                    }
                    else {
                        legal_moves.push_back(((piece<<9) + (b<<1), Piece::KNIGHT));
                    }
                }
                wn_possi = wn_possi.blsr();
                //wn_possi = wn_possi & (wn_possi - 1);
            }
        }
        
        //Bishop
        let mut wb_test = game.wb;
        while wb_test != 0 {
            let piece = wb_test.tzcnt();
            wb_test = wb_test & (wb_test - 1);
            let mut wb_possi = diag_antid_moves(piece, occupied) & !white;
            while wb_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = wb_possi.tzcnt();
                let capture = compute_move_w_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(true, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece<<9) + (b<<1), Piece::BISHOP));
                    }
                    else {
                        legal_moves.push_back(((piece<<9) + (b<<1), Piece::BISHOP));
                    }
                }
                wb_possi = wb_possi.blsr();
                //wb_possi = wb_possi & (wb_possi - 1);
            }
        }
        //Rook
        let mut wr_test = game.wr;
        while wr_test != 0 {
            let piece = wr_test.tzcnt();
            wr_test = wr_test & (wr_test - 1);
            let mut wr_possi = hv_moves(piece, occupied) & !white;
            while wr_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = wr_possi.tzcnt();
                let capture = compute_move_w_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(true, &game1);
                wr_possi = wr_possi.blsr();
                //wr_possi = wr_possi & (wr_possi - 1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece<<9) + (b<<1), Piece::ROOK));
                    }
                    else {
                        legal_moves.push_back(((piece<<9) + (b<<1), Piece::ROOK));
                    }
                }
            }
        }

        //Queen
        if game.wq != 0 {
            let piece = game.wq.tzcnt();
            let mut wq_possi = (hv_moves(piece, occupied) | diag_antid_moves(piece, occupied)) & !white;
            while wq_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = wq_possi.tzcnt();
                let capture = compute_move_w_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(true, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece<<9) + (b<<1), Piece::QUEEN));
                    }
                    else {
                        legal_moves.push_back(((piece<<9) + (b<<1), Piece::QUEEN));
                    }
                }
                wq_possi = wq_possi.blsr();
                //wq_possi = wq_possi & (wq_possi - 1);
            }
        }
        //King
        
        let mut possi_wk = possibility_k(game.wk) & !white;
        while possi_wk != 0 {
            let mut game1 = *game;
            let b = possi_wk.tzcnt();
            let capture = compute_move_w_thrust((game.wk.tzcnt(), b, Piece::NONE), &mut game1);
            let is_check = is_attacked(true, &game1);
            if !is_check {
                if capture > 0 {
                    legal_moves.push_front(((game.wk.tzcnt() <<9) + (b<<1), Piece::KING));
                }
                else {
                    legal_moves.push_back(((game.wk.tzcnt() <<9) + (b<<1), Piece::KING));
                }
            }
            possi_wk = possi_wk.blsr();
            //possi_wk = possi_wk & (possi_wk - 1);
        }
    }
    else { //Black Possiblity
        //Pions Possibility
        //let white_normal = attack_normal_piece_w(&game, white);
        let mut bp_test = game.bp;
        while  bp_test != 0 {
            let piece = bp_test.tzcnt();
            let bp_extract = 1u64 << piece;
            
            bp_test = bp_test & (bp_test-1);
            let mut possi_bp = possibility_bp2(bp_extract, !(occupied), white);
            while possi_bp != 0 {
                let mut game1 = *game;
                let mut promote = 0;
                let mut promote_piece = Piece::NONE;
                let b = possi_bp.tzcnt();
                if 1<<b & RANK_MASK[0] != 0 {
                    //println!("hello black {} {} {}", b, RANK_MASK[0] ,b & RANK_MASK[0]);
                    promote = 1;
                    promote_piece = Piece::QUEEN;
                }
                let capture = compute_move_b_thrust((piece, b, promote_piece), &mut game1);
                let is_check = is_attacked(false, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece <<9) + (b<<1)+promote, Piece::PAWN));
                    }
                    else {
                        legal_moves.push_back(((piece <<9) + (b<<1)+promote, Piece::PAWN));
                    }
                }
                possi_bp = possi_bp & (possi_bp - 1);
            }
        }
        //Knight
        //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
        let mut bn_test = game.bn;
        while bn_test != 0 {
            let piece = bn_test.tzcnt() ;
            let bn_extract = 1u64 << piece;
            bn_test = bn_test & (bn_test-1);
            let mut bn_possi = possibility_n(bn_extract) & !black;
            while bn_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = bn_possi.tzcnt() ;
                let capture = compute_move_b_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(false, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece <<9) + (b<<1), Piece::KNIGHT));
                    }
                    else {
                        legal_moves.push_back(((piece <<9) + (b<<1), Piece::KNIGHT));
                    }
                }
                bn_possi = bn_possi & (bn_possi - 1);
            }
        }
        
        //Bishop
        let mut bb_test = game.bb;
        while bb_test != 0 {
            let piece = bb_test.tzcnt();
            bb_test = bb_test & (bb_test - 1);
            let mut bb_possi = diag_antid_moves(piece, occupied) & !black;
            while bb_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = bb_possi.tzcnt();
                let capture = compute_move_b_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(false, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece <<9) + (b<<1), Piece::BISHOP));
                    }
                    else {
                        legal_moves.push_back(((piece <<9) + (b<<1), Piece::BISHOP));
                    }
                }
                bb_possi = bb_possi & (bb_possi - 1);
            }
        }
        //Rook
        let mut br_test = game.br;
        while br_test != 0 {
            let piece = br_test.tzcnt();
            br_test = br_test & (br_test - 1);
            let mut br_possi = hv_moves(piece, occupied) & !black;
            while br_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = br_possi.tzcnt();
                let capture = compute_move_b_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(false, &game1);
                if !is_check {
                    if capture > 0 {
                    legal_moves.push_front(((piece <<9) + (b<<1), Piece::ROOK));
                    }
                    else {
                        legal_moves.push_back(((piece <<9) + (b<<1), Piece::ROOK));
                    }
                }
                br_possi = br_possi & (br_possi - 1);
            }
        }

        //Queen
        if game.bq != 0 {
            let piece = game.bq.tzcnt();
            let mut bq_possi = (hv_moves(piece, occupied) | diag_antid_moves(piece, occupied)) & !black;
            while bq_possi != 0 {
                //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
                let mut game1 = *game;
                let b = bq_possi.tzcnt();
                let capture = compute_move_b_thrust((piece, b, Piece::NONE), &mut game1);
                let is_check = is_attacked(false, &game1);
                if !is_check {
                    if capture > 0 {
                        legal_moves.push_front(((piece <<9) + (b<<1), Piece::QUEEN));
                    }
                    else {
                        legal_moves.push_back(((piece <<9) + (b<<1), Piece::QUEEN));
                    }
                }
                bq_possi = bq_possi & (bq_possi - 1);
            }
        }
        
        //King
        let mut possi_bk = possibility_k(game.bk) & !black;
        let piece = game.bk.tzcnt();
        while possi_bk != 0 {
            //let (mut wp, mut wn, mut wb, mut wr, mut wq, mut wk, mut bp, mut bn, mut bb, mut br, mut bq, mut bk) = copy_bitboard(wp1, wn1, wb1, wr1, wq1, wk1, bp1, bn1, bb1, br1, bq1, bk1);
            let mut game1 = *game;
            let b = possi_bk.tzcnt();
            let capture = compute_move_b_thrust((piece, b, Piece::NONE), &mut game1);
            let is_check = is_attacked(false, &game1);
            if !is_check {
                if capture > 0 {
                    legal_moves.push_front(((piece <<9) + (b<<1), Piece::KING));
                }
                else {
                    legal_moves.push_back(((piece <<9) + (b<<1), Piece::KING));
                }
            }
            possi_bk = possi_bk & (possi_bk - 1);
        }
    }
    legal_moves
}

pub fn print_custum_move(a_move : (u64,Piece)) {
    let a = a_move.0>>9;
    let b = (a_move.0 & 510)>>1;
    if a_move.0 & 1 == 1 {
        println!("{}{}q {:?}", convert_square_to_move(a), convert_square_to_move(b), a_move.1);
    }
    else {
        println!("{}{} {:?}", convert_square_to_move(a), convert_square_to_move(b), a_move.1);
    }
}
pub fn convert_custum_move(the_move : (u64, Piece)) -> (u64, u64, Piece) {
    let a = the_move.0>>9;
    let b = (the_move.0 & 510)>>1;
    let c = match the_move.0 & 1 {
        1 => { Piece::QUEEN },
        _ => { Piece::NONE }
    };
    (a, b, c)
}

pub fn convert_move_to_str(a:u64, b:u64, p : Piece) -> String {
    let mut a = convert_square_to_move(a);
    let b = convert_square_to_move(b);
    a.push_str( &*b);
    match p {
        Piece::QUEEN => {
            a.push('q');
        },
        _ => {}
    }
    a
}

pub fn convert_custum_to_str(moveto: u64) -> String {
    let (a,b,p) = convert_custum_move((moveto, Piece::QUEEN));
    convert_move_to_str(a, b, p)
}

