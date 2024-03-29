use crate::types::{to_var, Lit};

use super::map::{var_map, VarMap};

#[derive(Clone, Copy, Debug)]
pub enum Reason {
    Decision,
    Propagation { i_clause: usize },
}

#[derive(Clone)]
struct VarData {
    value: bool,
    level: usize,
    reason: Reason,
}

pub struct Assignment {
    data: VarMap<Option<VarData>>,
    trail: Vec<Lit>,
    levels: Vec<usize>,
}

impl Assignment {
    pub fn new(var_count: usize) -> Self {
        Self {
            data: var_map(var_count),
            trail: vec![],
            levels: vec![],
        }
    }

    pub fn eval(&self, lit: Lit) -> Option<bool> {
        self.data[to_var(lit)]
            .as_ref()
            .map(|data| data.value == lit.is_positive())
    }

    pub fn set(&mut self, lit: Lit, reason: Reason) {
        self.trail.push(lit);

        if let Reason::Decision = reason {
            self.levels.push(self.trail.len() - 1);
        }

        let data = VarData {
            value: lit.is_positive(),
            level: self.last_level(),
            reason,
        };
        self.data[to_var(lit)] = Some(data);
    }

    pub fn trail(&self) -> &[Lit] {
        &self.trail
    }

    pub fn level(&self, lit: Lit) -> Option<usize> {
        self.data[to_var(lit)].as_ref().map(|data| data.level)
    }

    pub fn reason(&self, lit: Lit) -> Option<Reason> {
        self.data[to_var(lit)].as_ref().map(|data| data.reason)
    }

    pub fn last_level(&self) -> usize {
        self.levels.len()
    }

    /// Revert all changes at `level` (incl.) and above.
    pub fn backtrack(&mut self, level: usize) {
        self.levels.drain(level..);
        let i = self.levels.pop().unwrap_or(0);
        for lit in self.trail.drain(i..) {
            self.data[to_var(lit)] = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Assignment, Reason};

    #[test]
    fn basic() {
        let mut ass = Assignment::new(2);

        assert_eq!(ass.last_level(), 0);

        ass.set(1, Reason::Decision);
        ass.set(-2, Reason::Propagation { i_clause: 0 });

        assert_eq!(ass.last_level(), 1);
        assert_eq!(ass.level(1), Some(1));
        assert_eq!(ass.level(2), Some(1));

        ass.backtrack(1);
        assert_eq!(ass.eval(2), None);
        assert_eq!(ass.eval(1), None);
    }
}
