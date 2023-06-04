
use bitintr::Pext;
use lazy_static::lazy_static;

pub const SLIDING_MOVE_TABLE_SIZE: usize = INDEX_DATA.table_size;

use crate::chess::*;
lazy_static! {
    pub static ref TABLE_SLIDING : [u64; SLIDING_MOVE_TABLE_SIZE] = {
        let mut table = [0u64; SLIDING_MOVE_TABLE_SIZE];
        println!("{}", SLIDING_MOVE_TABLE_SIZE);
        /*write_moves(
            &mut table,
            get_rook_relevant_blockers,
            get_rook_moves_index,
            get_rook_moves_slow
        );*/
        for square in 0..64 {
            let mask = get_rook_relevant_blockers(square);
            let d = mask;
                let mut n : u64 = 0;
                loop {
                    table[get_rook_moves_index(square, n)] = hv_moves(square, n);
                    n = (n - d) & d;
                    if n == 0 {
                        break;
                    }
                } ;
            
            /*for blockers in mask.iter_subsets() {
            }*/
        }
        /*
        write_moves(
            &mut table,
            get_bishop_relevant_blockers,
            get_bishop_moves_index,
            get_bishop_moves_slow
        );*/
        for square in 0..64 {
            let mask = get_bishop_relevant_blockers(square);
            let d = mask;
            let mut n : u64 = 0;
            loop {
                //doSomeThingWithSubset(n);
                table[get_bishop_moves_index(square, n)] = diag_antid_moves(square, n);
                n = (n - d) & d;
                if n == 0 {
                    break;
                }
            };
        }
        table
    };

}
//#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
//compile_error!("pext feature can only be enabled if target has BMI2.");

pub fn pext_u64(a: u64, mask: u64) -> u64 {
    // SAFETY: A compile error is raised if PEXT is not available. PEXT is always safe if available.
    unsafe { core::arch::x86_64::_pext_u64(a, mask) }
    //a.pext(mask)
}

struct PextEntry {
    offset: u32,
    mask: u64
}

const EMPTY_ENTRY: PextEntry = PextEntry {
    offset: 0,
    mask: 0
};
struct PextIndexData {
    pub rook_data: [PextEntry; 64],
    bishop_data: [PextEntry; 64],
    table_size: usize
}

const INDEX_DATA: &PextIndexData = {
    let mut offset = 0;

    let mut rook_data = [EMPTY_ENTRY; 64];
    let mut i = 0;
    while i < rook_data.len() {
        let square = i;
        let mask = get_rook_relevant_blockers(square as u64);
        rook_data[i] = PextEntry { offset, mask };
        offset += 1 << mask.count_ones();
        i += 1;
    }

    let mut bishop_data = [EMPTY_ENTRY; 64];
    let mut i = 0;
    while i < bishop_data.len() {
        let square = i;
        let mask = get_bishop_relevant_blockers(square as u64);
        bishop_data[i] = PextEntry { offset, mask };
        offset += 1 << mask.count_ones();
        i += 1;
    }

    &PextIndexData {
        rook_data,
        bishop_data,
        table_size: offset as usize
    }
};

fn get_pext_index(index_data: &[PextEntry; 64], square: u64, blockers: u64) -> usize {
    let index_data = &index_data[square as usize];
    let index = pext_u64(blockers, index_data.mask);
    println!("{} {} {}", index_data.offset, index, index_data.mask);
    index_data.offset as usize + index as usize
}

pub fn get_rook_moves_index(square: u64, blockers: u64) -> usize {
    get_pext_index(&INDEX_DATA.rook_data, square, blockers)
}

pub fn get_bishop_moves_index(square: u64, blockers: u64) -> usize {
    get_pext_index(&INDEX_DATA.bishop_data, square, blockers)
}

pub const fn get_rook_relevant_blockers(square: u64) -> u64 {
    let rank_moves = (1<<rank(square)) & !(FILE_MASKSC[0] | FILE_MASKSC[7]);
    let file_moves = (1<<file(square)) & !(RANK_MASKC[0] | RANK_MASKC[7]);
    (rank_moves | file_moves) & !(1<<square)
}

pub const fn get_bishop_relevant_blockers(square: u64) -> u64 {
    let mut rays = 0;
    let mut i = 0;
    while i < 64 {
        let target = i;
        let rd = (rank(square) as i8 - rank(target) as i8).abs();
        let fd = (file(square) as i8 - file(target) as i8).abs();
        if rd == fd && rd != 0 {
            rays |= 1 << i;
        }
        i += 1;
    }
    rays & !EDGES
}

#[inline(always)]
pub const fn file(a : u64) -> u64 {
    a & 0b000111
}
#[inline(always)]
pub const fn rank(a : u64) -> u64 {
    a  >> 3
}


fn write_moves(
    table: &mut [u64],
    relevant_blockers: impl Fn(u64) -> u64,
    table_index: impl Fn(u64, u64) -> usize,
    slider_moves: impl Fn(u64, u64) -> u64
) {
    for square in 0..64 {
        let mask = relevant_blockers(square);
        let d = mask;
            let mut n : u64 = 0;
            loop {
                //doSomeThingWithSubset(n);
                table[table_index(square, n)] = slider_moves(square, n);
                n = (n - d) & d;
                if n == 0 {
                    break;
                }
            } ;
        
        /*for blockers in mask.iter_subsets() {
        }*/
    }
}