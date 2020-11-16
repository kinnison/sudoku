mod grid;
mod rules;
mod types;

use grid::*;
use rules::*;
use types::*;

fn apply<'a, R>(grid: &mut SGrid<'a, R>, input: &str) -> SResult
where
    R: Ruleset,
{
    let mut ch = input.chars();
    for row in 0..9 {
        for col in 0..9 {
            let ch = ch.next().unwrap() as u8;
            if ch != b' ' {
                let val = ch - b'0';
                match grid.set_cell(row, col, val) {
                    SResult::Continue => {}
                    v => return v,
                }
            }
            println!("Grid now:\n{}", grid);
        }
    }
    SResult::Continue
}

fn main() {
    let rules = Normal::new();
    let mut grid = SGrid::new(&rules);

    if apply(
        &mut grid,
        "   26 7 168  7  9 19   45  82 1   4   46 29   5   3 28  93   74 4  5  367 3 18   ",
    ) != SResult::Continue
    {
        panic!("Failure applying input");
    }

    println!("Grid:\n{}", grid);
    println!("Applying naked singles...");
    loop {
        match grid.naked_singles() {
            SResult::Continue => {
                println!("Round we go");
            }
            SResult::Finished => break,
            other => panic!("Failure applying naked singles! {:?}", other),
        }
    }
    println!("Finished grid:\n{}", grid);
}
