mod chess;
mod zobrist;
mod pext_slider;
use bitintr::Tzcnt;
use chess::*;
use zobrist::*;
use pext_slider::*;
fn main() {
    //let game = Game::default();
    let fen = "position fen rnbqkbnr/1pppppp1/8/p6p/P6P/8/1PPPPPP1/RNBQKBNR w KQkq - 0 1";
    //let fen = "position fen 8/3k2pp/8/8/8/8/5r2/2K5 w - - 21 84";
    let game = get_bitboard_from_fen(fen.trim().split_ascii_whitespace().collect());
    
    let hv = TABLE_SLIDING[get_rook_moves_index(game.wr.tzcnt(), game.occupied())];
    //let hv = hv_moves(game.wr.tzcnt(), game.occupied());
    println!("PEXT {}", pext_u64(43, game.occupied()));
    _draw_bitboard(game.occupied());
    _draw_bitboard(hv as u64);
    
    /*let c = get_checked_mask_b(&game);
    _draw_bitboard(c);

    //let (mut capture, quiet, score_move) = get_legal_moves_fast_c(&mut game);
    let moves = get_legal_moves_fast(&mut game);

    println!("\nQUIET");
    for movto in moves {
        _print_custum_move2(movto);
    }*/
    
}
pub fn get_bitboard_from_fen(fen : Vec<&str>) -> Game {
    let mut game = Game::empty();
    let board = fen[2];
    let toplay = fen[3];
    let castling_right = fen[4];
    let en_passant = fen[5];
    let nb_coup = fen[6];
    let mut i : i32 = 63-7;
    for ligne in board.split('/') { //Transform fen to board
        //println!("ligne : {ligne}");
        for c in ligne.chars() {
            match c {
                'p' => { game.bp |= 1<<i },
                'n' => { game.bn |= 1<<i },
                'b' => { game.bb |= 1<<i },
                'r' => { game.br |= 1<<i },
                'q' => { game.bq |= 1<<i },
                'k' => { game.bk |= 1<<i },
                'P' => { game.wp |= 1<<i },
                'N' => { game.wn |= 1<<i },
                'B' => { game.wb |= 1<<i },
                'R' => { game.wr |= 1<<i },
                'Q' => { game.wq |= 1<<i },
                'K' => { game.wk |= 1<<i },
                n => {
                    if n.is_alphanumeric() {
                        let k = n.to_digit(10).unwrap() as i32;
                        i = i+k-1;
                    }
                }
            }
            i+=1;
        }
        i-=16;
    }

    game.white_to_play = match toplay.chars().next().unwrap() {
        'w' => true,
        'b' => false,
        _ => { true }
    };

    for right in castling_right.chars() {
        match right {
            'K' => game.wking_castle = true,
            'Q' => game.wqueen_castle = true,
            'k' => game.bking_castle = true,
            'q' => game.bqueen_castle = true,
            _=> {}
        }
    }
    let en_passant_check = match en_passant.chars().nth(0) {
        Some(s) => {
            match s {
                '-' => { false },
                _ => { true },
            }
        },
        None => { false }
    };
    if en_passant_check {
        game.en_passant |= 1<<convert_move_str_to_bitboard(en_passant);
        _draw_bitboard(game.en_passant);
    }
    _draw_board(&game);
    game.hash = init_zobrist_key(&game);
    game
}