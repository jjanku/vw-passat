use std::io::{BufRead, BufReader, Read};

pub struct Problem {
    pub var_count: usize,
    pub clauses: Vec<Vec<i32>>,
}

pub fn read_problem(reader: &mut impl Read) -> Problem {
    let mut lines = BufReader::new(reader).lines().map(|l| l.unwrap());

    let (var_count, clause_count) = loop {
        let line = lines.next().unwrap();

        if line.starts_with("c") {
            // comment line
            continue;
        }

        // problem line
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts[0], "p");
        assert_eq!(parts[1], "cnf");
        break (
            parts[2].parse::<usize>().unwrap(),
            parts[3].parse::<usize>().unwrap(),
        );
    };

    let mut clauses = vec![];
    let mut clause = vec![];

    for line in lines {
        for word in line.split_whitespace() {
            let lit = word.parse::<i32>().unwrap();
            match lit {
                0 => {
                    clauses.push(clause.clone());
                    clause.clear();
                }
                _ => {
                    clause.push(lit);
                }
            }
        }
    }

    assert_eq!(clause_count, clauses.len());

    Problem { var_count, clauses }
}

#[cfg(test)]
mod tests {
    use super::{read_problem, Problem};

    #[test]
    fn basic() {
        let input = b"c whatever\np cnf 2 2\n1 2 0\n1 -2 0";
        let Problem { var_count, clauses } = read_problem(&mut input.as_slice());
        assert_eq!(var_count, 2);
        assert_eq!(clauses.len(), 2);
        assert_eq!(clauses[0], vec![1, 2]);
        assert_eq!(clauses[1], vec![1, -2]);
    }

    #[test]
    fn split() {
        let input = b"c whatever\np cnf 1 1\n1 1\n-1 -1 0";
        let Problem { clauses, .. } = read_problem(&mut input.as_slice());
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0], vec![1, 1, -1, -1]);
    }
}
