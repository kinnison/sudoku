//! Techniques for solving Sudoku.
//!
//! Each technique must perform at most one kind of action and should
//! try not to replicate the capability of another technique.  Solving
//! a grid will apply techniques in some order, restarting the sequence
//! any time a technique acts.  If the final technique in a sequence
//! returns Stuck then solving has failed and the grid is considered
//! insoluable.

use super::SCell;
use super::SGrid;
use super::SResult;

use log::debug;

use std::collections::{HashMap, HashSet};

pub enum SolveStepResult {
    Stuck,
    Acted,
    Finished,
    Failed(SResult),
}

use SolveStepResult::*;

pub trait Technique {
    fn name(&self) -> &'static str;

    fn step(&mut self, grid: &mut SGrid) -> SolveStepResult {
        match grid.done() {
            SResult::Finished => Finished,
            _ => Stuck,
        }
    }
}

/// The Naked Single technique.
///
/// A 'naked single' is a cell which has not yet been fixed and
/// yet only has a single pencil mark and so is not possible to
/// be anything but the pencil mark present.
///
/// To act on a naked single we simply replace the possibility with
/// the fixed cell.
pub struct NakedSingle;

impl Technique for NakedSingle {
    fn name(&self) -> &'static str {
        "naked single"
    }

    fn step(&mut self, grid: &mut SGrid) -> SolveStepResult {
        for row in 0..9 {
            for col in 0..9 {
                match grid.cell(row, col) {
                    SCell::Fixed(_) => {}
                    cell @ SCell::Possible(_) => {
                        let mut values = cell.values();
                        if values.len() == 1 {
                            let val = values.next().unwrap();
                            match grid.set_cell(row, col, val) {
                                SResult::Continue | SResult::Finished => return Acted,
                                res => return Failed(res),
                            }
                        }
                    }
                }
            }
        }
        Stuck
    }
}

/// The hidden single technique
///
/// A hidden single is where a house (row, column, box) contains a cell
/// which on the face of it has more than one possibility, however on
/// closer inspection is the only cell in that house which can be one of
/// the values it could take.  For example, the cell might have 1,2,3 as
/// possibilities but if no other cell in the house could have the value 2
/// then this is a hidden single.  To act on a hidden single we simply
/// replace all possibilities in the cell with the fixed hidden single.
pub struct HiddenSingle;

impl Technique for HiddenSingle {
    fn name(&self) -> &'static str {
        "hidden single"
    }

    fn step(&mut self, grid: &mut SGrid) -> SolveStepResult {
        for house in 0..27 {
            let content = grid.house(house);
            let mut found = HashMap::new();
            for (n, cell) in content.iter().enumerate() {
                match cell {
                    SCell::Fixed(_) => {}
                    SCell::Possible(_) => {
                        for value in cell.values() {
                            found.entry(value).or_insert(HashSet::new()).insert(n);
                        }
                    }
                };
            }
            for value in 1..=9 {
                if let Some(s) = found.get_mut(&value) {
                    if s.len() == 1 {
                        let cell = s.iter().copied().next().unwrap();
                        debug!("{:?}", content[cell]);
                        grid.set_house(house, cell, value);
                        return Acted;
                    }
                }
            }
        }
        Stuck
    }
}

/// The naked pair technique
///
/// A naked pair is where two unfixed cells in a house have the
/// same two possibilities.  Where you have a naked pair you can
/// then eliminate those two possibilities from any other cell in
/// that house.
pub struct NakedPair;

impl Technique for NakedPair {
    fn name(&self) -> &'static str {
        "naked pair"
    }

    fn step(&mut self, grid: &mut SGrid) -> SolveStepResult {
        for house in 0..27 {
            let cells = grid.house(house);
            for a in 0..8 {
                if cells[a].possibilities() != 2 {
                    continue;
                }
                for b in (a + 1)..9 {
                    if cells[a] == cells[b] {
                        // This is a naked pair, but can we do anything?
                        let mut changed = false;
                        for other in 0..9 {
                            if other == a || other == b {
                                continue;
                            }
                            changed |= grid.house_cell_mut(house, other).remove_all(cells[a]);
                        }
                        if changed {
                            return Acted;
                        }
                    }
                }
            }
        }
        Stuck
    }
}

pub struct SolverSet {
    techniques: Vec<Box<dyn Technique>>,
}

impl SolverSet {
    pub fn new() -> SolverSet {
        Self {
            techniques: Vec::new(),
        }
    }

    pub fn add_technique<T>(&mut self, t: T)
    where
        T: Technique + 'static,
    {
        self.techniques.push(Box::new(t))
    }

    pub fn solve_grid(&mut self, grid: &mut SGrid) -> SolveStepResult {
        let mut tnum = 0;
        loop {
            if let SResult::Finished = grid.done() {
                break Finished;
            }
            if tnum == self.techniques.len() {
                break Stuck;
            }
            debug!("Applying {}", self.techniques[tnum].name());
            match self.techniques[tnum].step(grid) {
                Stuck => {
                    tnum += 1;
                    continue;
                }
                Acted => {
                    tnum = 0;
                    continue;
                }
                res => {
                    break res;
                }
            }
        }
    }
}
