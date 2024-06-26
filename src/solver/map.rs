use std::ops::{Index, IndexMut};

use crate::types::{to_var, Lit};

// TODO: make it a struct?
// would need to implement iter, index, swap
pub type VarMap<T> = Vec<T>;

pub fn var_map<T: Clone + Default>(var_count: usize) -> VarMap<T> {
    vec![Default::default(); var_count + 1]
}

pub struct LitMap<T>(Vec<T>);

impl<T> LitMap<T> {
    fn position(lit: Lit) -> usize {
        2 * to_var(lit) - (lit.is_negative() as usize)
    }
}

impl<T: Clone + Default> LitMap<T> {
    pub fn new(var_count: usize) -> Self {
        Self(vec![Default::default(); 2 * var_count + 1])
    }
}

impl<T> Index<Lit> for LitMap<T> {
    type Output = T;

    fn index(&self, index: Lit) -> &Self::Output {
        &self.0[LitMap::<T>::position(index)]
    }
}

impl<T> IndexMut<Lit> for LitMap<T> {
    fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
        &mut self.0[LitMap::<T>::position(index)]
    }
}
