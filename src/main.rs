use anyhow::{anyhow, bail, Context, Result};
use std::{collections::HashSet, env, fs};

type Domain = HashSet<u8>;

#[derive(Clone, Debug)]
enum Variable {
    Assigned(u8),
    Unassigned(Domain),
}

type Assignment = [Variable; 81];

fn assignment_from_file(filepath: &str) -> Result<Assignment> {
    fs::read_to_string(filepath)?
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c.to_digit(10) {
            Some(0) => Ok(Variable::Unassigned((1..10).collect())),
            Some(n) => Ok(Variable::Assigned(n as u8)),
            None => bail!("non-digit in input"),
        })
        .collect::<Result<Vec<Variable>>>()?
        .try_into()
        .map_err(|_| anyhow!("invalid length of input"))
}

fn assigned_variables(assignment: &Assignment) -> Vec<(usize, u8)> {
    assignment
        .iter()
        .enumerate()
        .filter_map(|(x, var)| match var {
            Variable::Assigned(val) => Some((x, *val)),
            Variable::Unassigned(_) => None,
        })
        .collect()
}

fn unassigned_variable(assignment: &Assignment) -> Option<(usize, &Domain)> {
    assignment
        .iter()
        .enumerate()
        .filter_map(|(x, var)| match var {
            Variable::Assigned(_) => None,
            Variable::Unassigned(d) => Some((x, d)),
        })
        .min_by_key(|(_x, d)| d.len())
}

fn generate_constraints(x: usize) -> Vec<usize> {
    let mut constraints = Vec::with_capacity(20);
    let (col, row) = (x % 9, x / 9);
    for offset in 0..9 {
        let i = col + offset * 9;
        if x != i {
            constraints.push(i);
        }

        let i = row * 9 + offset;
        if x != i {
            constraints.push(i);
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
            let i = (box_base_row + row_offset) * 9 + box_base_col + col_offset;
            if x != i {
                constraints.push(i);
            }
        }
    }
    constraints
}

fn backtrack(
    assignment: Assignment,
    mut called: i32,
    mut failed: i32,
) -> Result<(Assignment, i32, i32)> {
    called += 1;
    if let Some((x, domain)) = unassigned_variable(&assignment) {
        for val in domain {
            let mut new_assignment = assignment.clone();
            new_assignment[x] = Variable::Assigned(*val);
            match ac3(new_assignment) {
                Ok(a) => new_assignment = a,
                _ => continue,
            }
            match backtrack(new_assignment, called, failed) {
                Ok(x) => return Ok(x),
                _ => failed += 1,
            }
        }
        bail!("failure")
    } else {
        Ok((assignment, called, failed))
    }
}

fn ac3(mut assignment: Assignment) -> Result<Assignment> {
    let mut queue = assigned_variables(&assignment);
    while let Some((x, val)) = queue.pop() {
        for y in generate_constraints(x) {
            match assignment[y] {
                Variable::Assigned(assigned) if assigned == val => bail!("inconsistent"),
                Variable::Assigned(_) => (),
                Variable::Unassigned(ref mut domain) => {
                    // if we remove value from domain such that the variable is assigned, we change it to assigned
                    if domain.remove(&val) && domain.len() == 1 {
                        let val = *domain.iter().next().unwrap();
                        assignment[y] = Variable::Assigned(val);
                        queue.push((y, val));
                    }
                }
            }
        }
    }
    Ok(assignment)
}

fn print_assignment(assignment: &Assignment) {
    for row in assignment.chunks(9) {
        let line = row
            .iter()
            .map(|v| match v {
                Variable::Assigned(val) => val.to_string(),
                Variable::Unassigned(_) => " ".to_owned(),
            })
            .collect::<String>();
        println!("{line}");
    }
}

fn main() -> Result<()> {
    let filepath = env::args().nth(1).context("filepath missing as cmd arg")?;
    let assignment = assignment_from_file(&filepath).context("loading assignment from file")?;

    let (solved_assignment, called, failed) = backtrack(assignment, 0, 0)?;
    println!("backtrack called: {called}");
    println!("backtrack failed: {failed}\n");
    print_assignment(&solved_assignment);
    Ok(())
}
