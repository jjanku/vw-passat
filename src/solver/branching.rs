use std::cmp::Ordering;

use crate::types::{Lit, Var};

use super::assignment::Assignment;

#[derive(Clone, Copy, PartialEq)]
struct OrdF64(f64);

impl Eq for OrdF64 {}

impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl OrdF64 {
    fn new(value: f64) -> Self {
        assert!(!value.is_nan());
        Self(value)
    }
}

struct VarHeap<T> {
    heap: Vec<(T, Var)>,
    index: Vec<usize>,
    size: usize,
}

impl<T: Ord + Copy> VarHeap<T> {
    fn new(var_count: usize, default: T) -> Self {
        let mut heap = vec![];
        // 0 inserted for indexing by variables
        let mut index = vec![0];

        for var in 1..=var_count {
            index.push(heap.len());
            heap.push((default, var));
        }

        let size = var_count;

        Self { heap, index, size }
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.heap.swap(i, j);
        self.index.swap(self.heap[i].1, self.heap[j].1);
    }

    fn sift_up(&mut self, mut pos: usize) {
        while pos > 0 {
            let parent = (pos - 1) / 2;
            if self.heap[pos].0 <= self.heap[parent].0 {
                break;
            }
            self.swap(pos, parent);
            pos = parent;
        }
    }

    fn sift_down(&mut self, mut pos: usize) {
        loop {
            let mut max = pos;
            let left = 2 * pos + 1;
            if left < self.size && self.heap[left].0 > self.heap[max].0 {
                max = left;
            }
            let right = left + 1;
            if right < self.size && self.heap[right].0 > self.heap[max].0 {
                max = right;
            }

            if max != pos {
                self.swap(pos, max);
                pos = max;
            } else {
                break;
            }
        }
    }

    fn set(&mut self, var: Var, val: T) {
        let pos = self.index[var];
        self.heap[pos] = (val, var);

        self.sift_up(pos);
        self.sift_down(pos);
    }

    fn get(&self, var: Var) -> T {
        self.heap[self.index[var]].0
    }

    /// Applies `f` to every value in the heap.
    /// The supplied function must preserve the ordering of the items!
    fn transform(&mut self, mut f: impl FnMut(T) -> T) {
        for (val, _) in &mut self.heap {
            *val = f(*val);
        }
    }

    fn max(&self) -> Option<Var> {
        if self.size != 0 {
            Some(self.heap[0].1)
        } else {
            None
        }
    }

    fn extract(&mut self) -> Option<Var> {
        if self.size != 0 {
            let var = self.heap[0].1;
            self.swap(0, self.size - 1);
            self.size -= 1;
            self.sift_down(0);
            Some(var)
        } else {
            None
        }
    }

    fn restore(&mut self) -> Option<Var> {
        if self.size != self.heap.len() {
            self.size += 1;
            let var = self.heap[self.size - 1].1;
            self.sift_up(self.size - 1);
            Some(var)
        } else {
            None
        }
    }
}

pub struct Chooser {
    k: f64,
    seen: Vec<bool>,
    heap: VarHeap<OrdF64>,
}

impl Chooser {
    pub fn new(var_count: usize) -> Self {
        Self {
            k: 1.0,
            seen: vec![false; var_count + 1],
            heap: VarHeap::new(var_count, OrdF64::new(0.0)),
        }
    }

    pub fn touch(&mut self, var: Var) {
        if !self.seen[var] {
            self.seen[var] = true;

            let val = self.heap.get(var);
            self.heap.set(var, OrdF64::new(val.0 + self.k));
        }
    }

    pub fn rescale(&mut self) {
        self.k *= 1.01;

        const THRESHOLD: f64 = 10e100;
        if self.k > THRESHOLD {
            self.heap
                .transform(|OrdF64(val)| OrdF64::new(val / THRESHOLD));
            self.k /= THRESHOLD;
        }

        for var_seen in &mut self.seen {
            *var_seen = false;
        }
    }

    pub fn choose(&mut self, assignment: &Assignment) -> Option<Var> {
        let mut res = None;

        while let Some(var) = self.heap.max() {
            if assignment.eval(var as Lit).is_none() {
                res = Some(var);
                break;
            }
            self.heap.extract();
        }
        while self.heap.restore().is_some() {}

        res
    }
}
