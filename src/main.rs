mod grid;
mod rules;
mod technique;
mod types;

use grid::*;
use rules::*;
use technique::*;
use types::*;

fn apply(grid: &mut SGrid, input: &str) -> SResult {
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
        }
    }
    SResult::Continue
}

fn main() {
    pretty_env_logger::init_custom_env("SUDOKU_LOG");

    let rules = Normal::new();
    let mut grid = SGrid::new(rules);

    if apply(
        &mut grid,
        // Naked Single only
        //"   26 7 168  7  9 19   45  82 1   4   46 29   5   3 28  93   74 4  5  367 3 18   ",
        //"9 3 4 6182 4 81 5  8 35   2  8  5  6         5  4  9  1   94 3  3 12 7 4749 3 2 1",
        // Also needs HiddenSingle
        //"1 7  8   3        2  3 5 1  1653   85 3   6 17   1935  4 2 6  5        7   8  1 6",
        // Needs NakedPair
        //"4  27 6  798156234 2 84   7237468951849531726561792843 82 15479 7  243    4 87  2",
        " 9 3   14      23    52  6  5  678             394  5  8  79    71      56   3 8 ",
    ) != SResult::Continue
    {
        panic!("Failure applying input");
    }

    println!("Grid:\n{}", grid);
    let mut solver = SolverSet::new();
    solver.add_technique(NakedSingle);
    solver.add_technique(HiddenSingle);
    solver.add_technique(NakedPair);
    match solver.solve_grid(&mut grid) {
        SolveStepResult::Failed(e) => panic!("{:?}", e),
        SolveStepResult::Stuck => panic!("Grid insoluable.  Final state:\n{}", grid),
        SolveStepResult::Finished => {}
        SolveStepResult::Acted => unreachable!(),
    }
    println!("Finished grid:\n{}", grid);
    solver.dump_actions();
}
