mod chess;
mod zobrist;
use chess::*;
use zobrist::*;

fn main() {
    //let game = Game::default();
    let fen = "position fen 2k4p/7p/7p/4p3/3P1N1p/7p/7p/3K4 b - - 0 1";
    let game = get_bitboard_from_fen(fen.trim().split_ascii_whitespace().collect());
    //let a = get_pinned_mask_b(&game);
    //let  a = get_pinned_b(&game);
    //let a = get_checked_mask_b(&game);
    for movto in get_legal_moves_fast(&game) {
        print_custum_move2(movto);
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