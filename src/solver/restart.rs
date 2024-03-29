pub struct Luby {
    base: usize,
    uv: (isize, isize),
}

impl Luby {
    pub fn new(base: usize) -> Self {
        Self { base, uv: (1, 1) }
    }
}

impl Iterator for Luby {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let (u, v) = self.uv;
        // Based on Knuth's formula, see https://oeis.org/A182105.
        self.uv = if u & -u == v { (u + 1, 1) } else { (u, 2 * v) };
        Some(self.base * v as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::Luby;

    #[test]
    fn basic() {
        let expected = vec![1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, 1, 1, 2, 1, 1];
        let actual: Vec<usize> = Luby::new(1).take(20).collect();
        assert_eq!(expected, actual);
    }
}
