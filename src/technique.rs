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
                            found.entry(value).or_insert_with(HashSet::new).insert(n);
                        }
                    }
                };
            }
            for value in 1..=9 {
                if let Some(s) = found.get_mut(&value) {
                    if s.len() == 1 {
                        let cell = s.iter().copied().next().unwrap();
                        debug!("Cell {} in house {} is {:?}", cell, house, content[cell]);
                        debug!("Trying to isolate it down to {}", value);
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
                        debug!(
                            "Found a naked pair of {:?} in house {} cells {} and {}",
                            cells[a], house, a, b
                        );
                        let mut changed = false;
                        for other in 0..9 {
                            if other == a || other == b {
                                continue;
                            }
                            let this_changed =
                                grid.house_cell_mut(house, other).remove_all(cells[a]);
                            if this_changed {
                                debug!("We altered cell {} in the house", other);
                            }
                            changed |= this_changed;
                        }
                        if changed {
                            debug!("We changed some cells as a result");
                            return Acted;
                        }
                    }
                }
            }
        }
        Stuck
    }
}

/// The hidden pair technique.
///
/// Hidden pair is a technique where you look in a house for two cells
/// which, between them, are the only two cells which are possibly
/// a given number.  Having found them, you can eliminate any other
/// possiblities in those cells since those cells are limited to the
/// given hidden pair.  This transforms the hidden pair into a naked
/// pair, but that won't have any effect in *that* house (it may in
/// an overlapping house).
struct HiddenPair;

impl Technique for HiddenPair {
    fn name(&self) -> &'static str {
        "HiddenPair"
    }

    fn step(&mut self, grid: &mut SGrid) -> SolveStepResult {
        for house in 0..27 {
            let content = grid.house(house);
            let mut found = HashMap::new();
            // First up, iterate the cells in the house and map from cell value
            // to set of cells in the house which contain that value.
            for (n, cell) in content.iter().enumerate() {
                match cell {
                    SCell::Fixed(_) => {}
                    SCell::Possible(_) => {
                        for value in cell.values() {
                            found.entry(value).or_insert_with(HashSet::new).insert(n);
                        }
                    }
                };
            }
            // Now we're looking for *pairs* of values present in the same two cells
            for a in 0..8 {
                if found.get(&a).map(HashSet::len).unwrap_or(0) == 2 {
                    for b in a + 1..9 {
                        if found.get(&a) == found.get(&b) {
                            let mut hs = found.get(&a).unwrap().iter();
                            let c1 = *hs.next().unwrap();
                            let c2 = *hs.next().unwrap();
                            debug!(
                                "Found a {}/{} pair in cells {},{} of house {}",
                                a, b, c1, c2, house
                            );
                            let ncell = content[c1].intersect(&content[c2]);
                            let mut changed = grid.alter_house(house, c1, ncell);
                            changed |= grid.alter_house(house, c2, ncell);
                            if changed {
                                debug!("This resulted in an action");
                                return Acted;
                            }
                        }
                    }
                }
            }
        }
        Stuck
    }
}

/// The pointing technique
///
/// When all the cells in a given house which could be a particular
/// value are also in another house, any other cells in that other house
/// which could be that value should have it removed from them.
pub struct Pointing;

impl Technique for Pointing {
    fn name(&self) -> &'static str {
        "pointing"
    }

    fn step(&mut self, grid: &mut SGrid) -> SolveStepResult {
        for house in 0..27 {
            for value in 1..=9 {
                let mut found_in_house = HashSet::new();
                for cell in 0..9 {
                    if grid.house_cell(house, cell).values().any(|v| v == value) {
                        let (row, col) = SGrid::house_cell_to_row_col(house, cell);
                        found_in_house.insert((row, col));
                    }
                }
                if found_in_house.len() < 2 {
                    // No point looking at overlaps, there's fewer than 2 so not "pointing"
                    continue;
                }
                for overlapping_house in grid.rules().overlapping_houses(house).iter().copied() {
                    let mut found_in_overlap = HashSet::new();
                    for cell in 0..9 {
                        if grid
                            .house_cell(overlapping_house, cell)
                            .values()
                            .any(|v| v == value)
                        {
                            let (row, col) = SGrid::house_cell_to_row_col(overlapping_house, cell);
                            found_in_overlap.insert((row, col));
                        }
                    }
                    if found_in_overlap.len() < 3 {
                        // No point in looking at the overlapping cells, fewer than 3 means we're
                        // not pointing at anything *else* in that other house
                    }
                    debug!(
                        "Found {} in house {} points at house {}",
                        value, house, overlapping_house
                    );
                    let mut changed = false;
                    for (row, col) in found_in_overlap.into_iter() {
                        if !found_in_house.contains(&(row, col)) {
                            // This is a location in overlap which isn't in us,
                            // So we get to remove value from it
                            changed |= grid.cell_mut(row, col).remove(value);
                        }
                    }
                    if changed {
                        debug!("This did something");
                        return Acted;
                    }
                }
            }
        }
        Stuck
    }
}

pub struct SolverSet {
    techniques: Vec<Box<dyn Technique>>,
    actions: Vec<usize>,
    defers: Vec<usize>,
}

impl SolverSet {
    pub fn new() -> SolverSet {
        Self {
            techniques: Vec::new(),
            actions: Vec::new(),
            defers: Vec::new(),
        }
    }

    pub fn add_technique<T>(&mut self, t: T)
    where
        T: Technique + 'static,
    {
        self.techniques.push(Box::new(t));
        self.actions.push(0);
        self.defers.push(0);
    }

    pub fn solve_grid(&mut self, grid: &mut SGrid) -> SolveStepResult {
        let mut tnum = 0;
        'outer: loop {
            if let SResult::Finished = grid.done() {
                break Finished;
            }
            if tnum == self.techniques.len() {
                break Stuck;
            }
            debug!("Trying {}", self.techniques[tnum].name());
            match self.techniques[tnum].step(grid) {
                Stuck => {
                    debug!("{} is stuck", self.techniques[tnum].name());
                    self.defers[tnum] += 1;
                    tnum += 1;
                }
                Acted => {
                    debug!("{} acted", self.techniques[tnum].name());
                    self.actions[tnum] += 1;
                    tnum = 0;
                }
                res => {
                    break res;
                }
            }
            for row in 0..9 {
                for col in 0..9 {
                    if grid.cell(row, col).values().len() == 0 {
                        debug!("Well, that broke the grid!");
                        break 'outer Stuck;
                    }
                }
            }
        }
    }

    pub fn dump_actions(&self) {
        for ((technique, defer), action) in self
            .techniques
            .iter()
            .zip(self.defers.iter())
            .zip(self.actions.iter())
        {
            println!(
                "{} deferred {} times and acted {} times",
                technique.name(),
                defer,
                action
            );
        }
    }

    pub fn full() -> SolverSet {
        let mut ret = SolverSet::new();
        ret.add_technique(NakedSingle);
        ret.add_technique(HiddenSingle);
        ret.add_technique(NakedPair);
        ret.add_technique(HiddenPair);
        ret.add_technique(Pointing);
        ret
    }
}
