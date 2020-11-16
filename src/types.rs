#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum SResult {
    Conflict(usize, usize),
    Continue,
    Insoluable(usize, usize),
    Finished,
}
