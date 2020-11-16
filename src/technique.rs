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
