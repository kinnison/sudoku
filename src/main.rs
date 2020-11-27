mod grid;
mod rules;
mod technique;
mod types;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use grid::*;
use rules::*;
use technique::*;
use types::*;

fn apply(grid: &mut SGrid, input: &str) -> SResult {
    let mut ch = input.chars();
    for row in 0..9 {
        for col in 0..9 {
            let ch = ch.next().unwrap() as u8;
            if ch != b' ' && ch != b'.' {
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

fn solve_grid(mut grid: SGrid) -> bool {
    println!("Grid:\n{}", grid);
    let mut solver = SolverSet::full();
    match solver.solve_grid(&mut grid) {
        SolveStepResult::Failed(e) => panic!("{:?}", e),
        SolveStepResult::Stuck => {
            println!("Failed");
            solver.dump_actions();
            eprintln!("Grid insoluable.  Final state:\n{}", grid);
            return false;
        }
        SolveStepResult::Finished => {}
        SolveStepResult::Acted => unreachable!(),
    }
    println!("Finished grid:\n{}", grid);
    solver.dump_actions();
    true
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init_custom_env("SUDOKU_LOG");

    let fname = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| "grids.txt".into());
    let input = File::open(fname)?;
    let input = BufReader::new(input);
    let mut grids = Vec::new();
    let mut gridlines = String::new();
    for line in input.lines() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        gridlines.extend(line.chars().filter(|&c| ". 123456789".contains(c)));
        match gridlines.len() {
            n if n == 81 => {
                let mut grid = SGrid::new(Normal::new());
                if apply(&mut grid, &gridlines) != SResult::Continue {
                    panic!("Could not build grid from input");
                }
                grids.push(grid);
                gridlines = String::new();
            }
            n if n > 81 => {
                panic!("Unable to load grids from input, got more than 81 chars in a grid?");
            }
            _ => {}
        }
    }

    let mut failcount = 0;
    let gridcount = grids.len();
    for (n, grid) in grids.into_iter().enumerate() {
        println!("Grid {}...", n + 1);
        if !solve_grid(grid) {
            failcount += 1;
        }
    }
    println!("Failed to solve {} of {} grids", failcount, gridcount);
    println!(
        "That is a {}% success rate.",
        ((gridcount - failcount) * 100) / gridcount
    );
    Ok(())
}
