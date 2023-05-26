mod chess;
mod zobrist;
use chess::*;
use zobrist::*;

fn main() {
    //let game = Game::default();
    let fen = "position fen r3kb1r/pppb1ppp/2nqpn2/1B4Q1/3P4/2N2N2/PPP2PPP/R1B1K2R b KQkq - 2 9";
    let game = get_bitboard_from_fen(fen.trim().split_ascii_whitespace().collect());
    //let a = get_pinned_mask_b(&game);
    //let  a = get_pinned_b(&game);
    //let a = get_checked_mask_b(&game);
    for movto in get_legal_moves_fast(&game) {
        _print_custum_move2(movto);
    }
    //_draw_bitboard(a);
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
    _draw_board(&game);
    game.hash = init_zobrist_key(&game);
    game
}