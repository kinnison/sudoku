use super::rules::Ruleset;
use super::types::SResult;

#[derive(Debug, Copy, Clone)]
pub enum SCell {
    Fixed(u8),
    Possible(u16),
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

    pub fn naked_single(&self) -> Option<u8> {
        match self {
            SCell::Fixed(_) => None,
            SCell::Possible(v) => match v {
                0b000_000_001_0 => Some(1),
                0b000_000_010_0 => Some(2),
                0b000_000_100_0 => Some(3),
                0b000_001_000_0 => Some(4),
                0b000_010_000_0 => Some(5),
                0b000_100_000_0 => Some(6),
                0b001_000_000_0 => Some(7),
                0b010_000_000_0 => Some(8),
                0b100_000_000_0 => Some(9),
                _ => None,
            },
        }
    }
}

pub struct SGrid<'a, R> {
    cells: [SCell; 81],
    rules: &'a R,
}

impl<'a, R> std::fmt::Display for SGrid<'a, R>
where
    R: Ruleset,
{
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

macro_rules! stry {
    ($expr:expr) => {
        match $expr {
            $crate::SResult::Continue => (),
            other => return other,
        }
    };
}

impl<'a, R> SGrid<'a, R>
where
    R: Ruleset,
{
    pub fn new(rules: &'a R) -> Self {
        Self {
            cells: [SCell::default(); 81],
            rules,
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
        println!("Attempting to set row {} col {} to {}", row, col, val);
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
                    for pos in self.rules.sees(row, col) {
                        match self.cell_mut(pos.0, pos.1) {
                            SCell::Fixed(v) => println!(
                                "Cell at row {} col {} already fixed as {}",
                                pos.0, pos.1, v
                            ),
                            p => {
                                println!("Removing from cell at row {} col {}", pos.0, pos.1);
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

    /// Make a single run through the naked singles
    pub fn naked_singles(&mut self) -> SResult {
        for row in 0..9 {
            for col in 0..9 {
                if let Some(val) = self.cell(row, col).naked_single() {
                    stry!(self.set_cell(row, col, val));
                }
            }
        }
        self.done()
    }
}
