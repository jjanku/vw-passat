use crate::types::{Problem, Solution};

pub struct Solver {}

impl Solver {
    pub fn new(problem: &Problem) -> Self {
        Solver {}
    }

    pub fn solve(&mut self) -> Solution {
        Solution::Unknown
    }
}
