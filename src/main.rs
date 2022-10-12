use anyhow::{bail, Context, Result};
use std::{
    collections::HashSet,
    env, fmt, fs,
    ops::{Index, IndexMut},
};

type Domain = HashSet<u8>;

#[derive(Clone, Debug)]
enum Variable {
    Assigned(u8),
    Unassigned(Domain),
}

#[derive(Clone, Debug)]
struct Assignment([Variable; 81]);

impl Assignment {
    fn from_file(filepath: &str) -> Result<Self> {
        let v = fs::read_to_string(filepath)?
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c.to_digit(10) {
                Some(0) => Ok(Variable::Unassigned((1..10).collect())),
                Some(n) => Ok(Variable::Assigned(n as u8)),
                None => bail!("non-digit in input"),
            })
            .collect::<Result<Vec<Variable>>>()?;

        match v.try_into() {
            Ok(array) => Ok(Assignment(array)),
            Err(e) => bail!("invalid length of input ({})", e.len()),
        }
    }

    fn assigned_variables(&self) -> Vec<(usize, u8)> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(x, var)| match var {
                Variable::Assigned(val) => Some((x, *val)),
                Variable::Unassigned(_) => None,
            })
            .collect()
    }

    fn unassigned_variable(&self) -> Option<(usize, &Domain)> {
        self.0.iter().enumerate().find_map(|(x, var)| match var {
            Variable::Assigned(_) => None,
            Variable::Unassigned(d) => Some((x, d)),
        })
    }
}

impl Index<usize> for Assignment {
    type Output = Variable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Assignment {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i % 9 == 0 && i != 0 {
                f.write_str("\n")?;
            }
            if let Variable::Assigned(val) = v {
                f.write_str(&val.to_string())?;
            } else {
                panic!("tried to print unassigned value {:?}", v);
            }
        }
        Ok(())
    }
}

fn backtrack(
    assignment: Assignment,
    mut called: i32,
    mut failed: i32,
) -> Result<(Assignment, i32, i32)> {
    called += 1;
    if let Some((x, domain)) = assignment.unassigned_variable() {
        for val in domain {
            let mut new_assignment = assignment.clone();
            new_assignment[x] = Variable::Assigned(*val);
            if let Ok(a) = ac3(new_assignment) {
                new_assignment = a;
            } else {
                continue;
            }

            let result = backtrack(new_assignment, called, failed);
            if result.is_ok() {
                return result;
            } else {
                failed += 1;
            }
        }
        bail!("failure")
    } else {
        Ok((assignment, called, failed))
    }
}

fn generate_constraints(x: usize) -> Vec<usize> {
    let mut constraints = Vec::with_capacity(20);
    let (col, row) = (x % 9, x / 9);
    for offset in 0..9 {
        let j = col + offset * 9;
        if x != j {
            constraints.push(j);
        }

        let j = row * 9 + offset;
        if x != j {
            constraints.push(j);
        }
    }

    let (box_base_col, box_base_row) = (3 * (col / 3), 3 * (row / 3));
    for col_offset in 0..3 {
        if box_base_col + col_offset == col {
            continue;
        }
        for row_offset in 0..3 {
            if box_base_row + row_offset == row {
                continue;
            }
            let j = (box_base_row + row_offset) * 9 + box_base_col + col_offset;
            if x != j {
                constraints.push(j);
            }
        }
    }
    constraints
}

fn ac3(mut assignment: Assignment) -> Result<Assignment> {
    let mut queue = assignment.assigned_variables();
    while let Some((x, val)) = queue.pop() {
        for y in generate_constraints(x) {
            match assignment[y] {
                Variable::Assigned(assigned) if assigned == val => bail!("inconsistent"),
                Variable::Assigned(_) => (),
                Variable::Unassigned(ref mut domain) => {
                    // if we removed value from domain such that the variable is assigned, we change it to assigned
                    if domain.remove(&val) && domain.len() == 1 {
                        let val = *domain.iter().next().unwrap();
                        assignment[y] = Variable::Assigned(val);
                        queue.push((y, val));
                    }
                }
            };
        }
    }
    Ok(assignment)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filepath = args.get(1).context("filepath missing as cmd arg")?;
    let assignment = Assignment::from_file(filepath)
        .with_context(|| format!("loading assignment from file"))?;

    let (solved_assignment, called, failed) = backtrack(assignment, 0, 0)?;
    println!("backtrack called: {called}");
    println!("backtrack failed: {failed}\n");
    println!("{solved_assignment}");
    Ok(())
}
