mod chess;
mod zobrist;
use chess::*;
use zobrist::*;

fn main() {
    //let game = Game::default();
    let fen = "position fen rnbqkbnr/p1p1pppp/8/1p1p4/Q7/P1P5/1P1PPPPP/RNB1KBNR b KQkq - 0 1";
    //let fen = "position fen 8/3k2pp/8/8/8/8/5r2/2K5 w - - 21 84";
    let mut game = get_bitboard_from_fen(fen.trim().split_ascii_whitespace().collect());
    
    /*let playmove = convert_move_to_bitboard("g2g4");
    compute_move_w_thrust(playmove, &mut game);
    game.white_to_play^=true;
    _draw_bitboard(game.en_passant);
    let playmove = convert_move_to_bitboard("h4g3");
    compute_move_b_thrust(playmove, &mut game);
    game.white_to_play^=true;
    _draw_bitboard(game.en_passant);*/

    //let (mut capture, quiet, score_move) = get_legal_moves_fast_c(&mut game);
    let moves = get_legal_moves_fast(&mut game);
    _draw_bitboard(game.en_passant);
    println!("Capture");
    /*for movto in &capture {
        _print_custum_move2(*movto);
    }
    println!("SCORE");
    for movto in &score_move {
        print!("{} ", movto);
    }
    sort_move(&mut capture, score_move);
    println!("Capture");
    for movto in capture {
        _print_custum_move2(movto);
    }
    
*/  
    println!("\nQUIET");
    for movto in moves {
        _print_custum_move2(movto);
    }
    
    _draw_board(&game);
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