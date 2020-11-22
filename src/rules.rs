pub trait Ruleset {
    fn sees(&self, row: usize, col: usize) -> &[(usize, usize)];

    // Houses 0..9 are the rows
    // Houses 9..18 are the columns
    // Houses 18..27 are the boxes
    fn overlapping_houses(&self, house: usize) -> &[usize] {
        match house {
            0 | 1 | 2 => &[18, 19, 20],    // Top three rows
            3 | 4 | 5 => &[21, 22, 23],    // Next three
            6 | 7 | 8 => &[24, 25, 26],    // Bottom three
            9 | 10 | 11 => &[18, 21, 24],  // Left three boxes
            12 | 13 | 14 => &[19, 22, 25], // Next three
            15 | 16 | 17 => &[20, 23, 26], // Right three
            18 => &[0, 1, 2, 9, 10, 11],   // Top left box
            19 => &[0, 1, 2, 12, 13, 14],
            20 => &[0, 1, 2, 15, 16, 17], // Top right box
            21 => &[3, 4, 5, 9, 10, 11],
            22 => &[3, 4, 5, 12, 13, 14],
            23 => &[3, 4, 5, 15, 16, 17], // Middle right box
            24 => &[6, 7, 8, 9, 10, 11],
            25 => &[6, 7, 8, 12, 13, 14],
            26 => &[6, 7, 8, 15, 16, 17], // Bottom right box
            _ => unreachable!(),
        }
    }
}

pub static BOXES: &[[(usize, usize); 9]] = &[
    [
        (0, 0),
        (0, 1),
        (0, 2),
        (1, 0),
        (1, 1),
        (1, 2),
        (2, 0),
        (2, 1),
        (2, 2),
    ],
    [
        (0, 3),
        (0, 4),
        (0, 5),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (2, 4),
        (2, 5),
    ],
    [
        (0, 6),
        (0, 7),
        (0, 8),
        (1, 6),
        (1, 7),
        (1, 8),
        (2, 6),
        (2, 7),
        (2, 8),
    ],
    [
        (3, 0),
        (3, 1),
        (3, 2),
        (4, 0),
        (4, 1),
        (4, 2),
        (5, 0),
        (5, 1),
        (5, 2),
    ],
    [
        (3, 3),
        (3, 4),
        (3, 5),
        (4, 3),
        (4, 4),
        (4, 5),
        (5, 3),
        (5, 4),
        (5, 5),
    ],
    [
        (3, 6),
        (3, 7),
        (3, 8),
        (4, 6),
        (4, 7),
        (4, 8),
        (5, 6),
        (5, 7),
        (5, 8),
    ],
    [
        (6, 0),
        (6, 1),
        (6, 2),
        (7, 0),
        (7, 1),
        (7, 2),
        (8, 0),
        (8, 1),
        (8, 2),
    ],
    [
        (6, 3),
        (6, 4),
        (6, 5),
        (7, 3),
        (7, 4),
        (7, 5),
        (8, 3),
        (8, 4),
        (8, 5),
    ],
    [
        (6, 6),
        (6, 7),
        (6, 8),
        (7, 6),
        (7, 7),
        (7, 8),
        (8, 6),
        (8, 7),
        (8, 8),
    ],
];
/// Normal rules
///
/// Cells see their row, column, and sudoku box
/// Since this is entirely static, we could store it as a static set and not
/// need any data in the Normal struct, but we're lazy so we compute it on
/// startup.
pub struct Normal {
    sees: Vec<Vec<(usize, usize)>>,
}

impl Normal {
    fn boxcells(row: usize, col: usize) -> &'static [(usize, usize); 9] {
        match row {
            0 | 1 | 2 => match col {
                0 | 1 | 2 => &BOXES[0],
                3 | 4 | 5 => &BOXES[1],
                6 | 7 | 8 => &BOXES[2],
                _ => unimplemented!(),
            },
            3 | 4 | 5 => match col {
                0 | 1 | 2 => &BOXES[3],
                3 | 4 | 5 => &BOXES[4],
                6 | 7 | 8 => &BOXES[5],
                _ => unimplemented!(),
            },
            6 | 7 | 8 => match col {
                0 | 1 | 2 => &BOXES[6],
                3 | 4 | 5 => &BOXES[7],
                6 | 7 | 8 => &BOXES[8],
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
    pub fn new() -> Self {
        let mut ret = Normal { sees: Vec::new() };
        for row in 0..9 {
            for col in 0..9 {
                let mut seen = Vec::new();
                for col2 in 0..9 {
                    if col != col2 {
                        seen.push((row, col2));
                    }
                }
                for row2 in 0..9 {
                    if row != row2 {
                        seen.push((row2, col));
                    }
                }
                for &(brow, bcol) in Normal::boxcells(row, col) {
                    if brow != row && bcol != col {
                        seen.push((brow, bcol));
                    }
                }
                ret.sees.push(seen);
            }
        }
        ret
    }
}

impl Ruleset for Normal {
    fn sees(&self, row: usize, col: usize) -> &[(usize, usize)] {
        &self.sees[(row * 9) + col]
    }
}
