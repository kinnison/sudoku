use super::rules::Ruleset;
use super::types::SResult;

use std::rc::Rc;

use log::debug;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SCell {
    Fixed(u8),
    Possible(u16),
}

impl std::fmt::Debug for SCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SCell::Fixed(n) => write!(f, "Fixed({})", n),
            SCell::Possible(v) => {
                write!(f, "Possible(")?;
                for i in 1..=9 {
                    if (v & (1 << i)) != 0 {
                        write!(f, "{}", i)?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}

impl Default for SCell {
    fn default() -> Self {
        SCell::Possible(0b111_111_111_0)
    }
}

impl SCell {
    pub fn has(&self, val: u8) -> bool {
        match *self {
            SCell::Fixed(v) => v == val,
            SCell::Possible(f) => (f & (1 << val)) != 0,
        }
    }

    pub fn remove(&mut self, val: u8) -> bool {
        match self {
            SCell::Fixed(_) => false,
            SCell::Possible(f) => {
                if (*f & (1 << val)) == 0 {
                    // Already doesn't contain this, so it's fine to remove
                    true
                } else {
                    let left = *f & !(1 << val);
                    *self = SCell::Possible(left);
                    true
                }
            }
        }
    }

    // Returns true if something changed
    pub fn remove_all(&mut self, other: SCell) -> bool {
        let val = match other {
            SCell::Fixed(v) => 1 << v,
            SCell::Possible(v) => v,
        };
        match self {
            SCell::Fixed(_) => false,
            SCell::Possible(f) => {
                let newf = *f & !val;
                if *f != newf {
                    *f = newf;
                    true
                } else {
                    // Nothing removed
                    false
                }
            }
        }
    }

    pub fn values(&self) -> CellValues {
        match *self {
            SCell::Fixed(n) => CellValues::new(1 << n),
            SCell::Possible(v) => CellValues::new(v),
        }
    }

    pub fn possibilities(&self) -> usize {
        match *self {
            SCell::Fixed(_) => 0,
            SCell::Possible(v) => v.count_ones() as usize,
        }
    }
}

pub struct CellValues {
    v: u16,
    pos: u8,
}

impl CellValues {
    fn new(v: u16) -> Self {
        Self { v, pos: 0 }
    }
}

impl Iterator for CellValues {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos > 9 {
                break None;
            }
            self.pos += 1;
            if (self.v & (1 << self.pos)) != 0 {
                break Some(self.pos);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let v = self.v & !((1 << self.pos) - 1);
        (v.count_ones() as usize, Some(v.count_ones() as usize))
    }
}
impl ExactSizeIterator for CellValues {}

pub struct SGrid {
    cells: [SCell; 81],
    rules: Rc<dyn Ruleset>,
}

impl std::fmt::Display for SGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..=8 {
            for col in 0..=8 {
                match self.cell(row, col) {
                    SCell::Fixed(v) => write!(f, "{}", v)?,
                    _ => write!(f, " ")?,
                }
                if col == 2 || col == 5 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
            if row == 2 || row == 5 {
                writeln!(f, "---+---+---")?;
            }
        }
        Ok(())
    }
}

impl SGrid {
    pub fn new<R>(rules: R) -> Self
    where
        R: Ruleset + 'static,
    {
        Self {
            cells: [SCell::default(); 81],
            rules: Rc::new(rules),
        }
    }

    fn _pos(&self, row: usize, col: usize) -> usize {
        (row * 9) + col
    }

    pub fn cell(&self, row: usize, col: usize) -> SCell {
        self.cells[self._pos(row, col)]
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut SCell {
        &mut self.cells[self._pos(row, col)]
    }

    pub fn done(&self) -> SResult {
        for cell in &self.cells {
            match cell {
                SCell::Fixed(_) => continue,
                SCell::Possible(_) => return SResult::Continue,
            }
        }
        SResult::Finished
    }

    pub fn set_cell(&mut self, row: usize, col: usize, val: u8) -> SResult {
        debug!(
            "Attempting to set row {} col {} to {} (is currently {:?})",
            row,
            col,
            val,
            self.cell(row, col)
        );
        match self.cell(row, col) {
            SCell::Fixed(v) => {
                if v == val {
                    self.done()
                } else {
                    SResult::Conflict(row, col)
                }
            }
            c => {
                if !c.has(val) {
                    SResult::Conflict(row, col)
                } else {
                    /* Here we actually try and do this.
                     * Our invariants force us to have the grid consistent and
                     * given that, "all" we need to do is to erase the given value
                     * from anything we we can see.  If that results in a cell which
                     * cannot be anything, we're insoluable.
                     */
                    *self.cell_mut(row, col) = SCell::Fixed(val);
                    for pos in self.rules.clone().sees(row, col) {
                        match self.cell_mut(pos.0, pos.1) {
                            SCell::Fixed(v) => {
                                debug!("Cell at row {} col {} already fixed as {}", pos.0, pos.1, v)
                            }
                            p => {
                                debug!("Removing from cell at row {} col {}", pos.0, pos.1);
                                if !p.remove(val) {
                                    return SResult::Insoluable(pos.0, pos.1);
                                }
                            }
                        }
                    }
                    self.done()
                }
            }
        }
    }

    pub fn row_house(&self, row: usize) -> [SCell; 9] {
        let mut ret = [SCell::default(); 9];
        for col in 0..9 {
            ret[col] = self.cell(row, col);
        }
        ret
    }

    pub fn col_house(&self, col: usize) -> [SCell; 9] {
        let mut ret = [SCell::default(); 9];
        for row in 0..9 {
            ret[row] = self.cell(row, col);
        }
        ret
    }

    pub fn box_house(&self, _box: usize) -> [SCell; 9] {
        let mut ret = [SCell::default(); 9];
        for (n, (row, col)) in super::BOXES[_box].iter().enumerate() {
            ret[n] = self.cell(*row, *col);
        }
        ret
    }

    pub fn house(&self, house: usize) -> [SCell; 9] {
        match house {
            0..=8 => self.row_house(house),
            9..=17 => self.col_house(house - 9),
            18..=26 => self.box_house(house - 18),
            _ => unreachable!(),
        }
    }

    pub fn house_cell_to_row_col(house: usize, cell: usize) -> (usize, usize) {
        match house {
            0..=8 => (house, cell),
            9..=17 => (cell, house - 8),
            18..=26 => super::BOXES[house - 18][cell],
            _ => unreachable!(),
        }
    }

    pub fn set_house(&mut self, house: usize, cell: usize, val: u8) -> SResult {
        let (row, col) = Self::house_cell_to_row_col(house, cell);
        self.set_cell(row, col, val)
    }

    #[allow(dead_code)]
    pub fn house_cell(&self, house: usize, cell: usize) -> SCell {
        let (row, col) = Self::house_cell_to_row_col(house, cell);
        self.cell(row, col)
    }

    pub fn house_cell_mut(&mut self, house: usize, cell: usize) -> &mut SCell {
        let (row, col) = Self::house_cell_to_row_col(house, cell);
        self.cell_mut(row, col)
    }
}
