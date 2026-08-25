use std::{
    collections::HashMap,
    fmt::{Display, Error, Formatter},
};

pub type Cycle = Vec<(String, String)>; // Vec of (task_name, method_name)

#[derive(PartialEq, Eq, Debug)]
pub struct RecursionInfo {
    pub acyclic_tasks: Vec<String>,
    pub recursive_tasks: HashMap<String, Vec<Cycle>>,
    pub eps_prefix_tasks: HashMap<String, Vec<Cycle>>,
    pub empty_recursive_tasks: HashMap<String, Vec<Cycle>>,
    pub growing_empty_recursive_tasks: HashMap<String, Vec<Cycle>>,
    pub grow_and_shrink_tasks: HashMap<String, Vec<Cycle>>,
}

impl Display for RecursionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let is_acyclic = self.recursive_tasks.is_empty();
        if is_acyclic {
            return write!(f, "Non-recursive");
        }
        let headline = if !self.grow_and_shrink_tasks.is_empty() {
            "Grow and shrink recursion"
        } else if !self.growing_empty_recursive_tasks.is_empty() {
            "Growing empty prefix recursion"
        } else if !self.empty_recursive_tasks.is_empty() {
            "Empty recursion"
        } else if !self.eps_prefix_tasks.is_empty() {
            "Epsilon-prefix recursion"
        } else if !self.recursive_tasks.is_empty() {
            "Recursive"
        } else {
            "Acyclic"
        };
        write!(f, "{}", headline)?;
        format_task_cycles(f, "Recursive tasks", &self.recursive_tasks)?;
        format_task_cycles(f, "Zero-Cost Recursive tasks", &self.eps_prefix_tasks)?;
        format_task_cycles(
            f,
            "Zero-Cost Empty Recursive tasks",
            &self.empty_recursive_tasks,
        )?;
        format_task_cycles(
            f,
            "Growing Zero-Cost Recursive tasks",
            &self.growing_empty_recursive_tasks,
        )?;
        format_task_cycles(f, "Grow and Shrink tasks", &self.grow_and_shrink_tasks)?;
        if !self.acyclic_tasks.is_empty() {
            writeln!(f, "\n\tAcyclic tasks: {}", self.acyclic_tasks.len(),)?;
            write!(f, "\t\t{}", self.acyclic_tasks.join("\n\t\t"))?;
        }
        Ok(())
    }
}

// Prints one category: the number of initiator tasks, then each initiator
// with all of its cycles.
fn format_task_cycles(
    f: &mut Formatter<'_>,
    name: &str,
    tasks: &HashMap<String, Vec<Cycle>>,
) -> std::fmt::Result {
    if tasks.is_empty() {
        return Ok(());
    }
    write!(f, "\n\t{}: {}", name, tasks.keys().len())?;
    for initiator in tasks.keys() {
        let cycles = &tasks[initiator];
        write!(f, "\n\t\t{} ({} cycle(s))", initiator, cycles.len())?;
    }
    Ok(())
}

pub struct MetaData {
    pub recursion: RecursionInfo,
    pub nullables: Vec<String>,
    pub domain_name: String,
    pub n_actions: u32,
    pub n_tasks: u32,
    pub n_methods: u32,
}

impl Display for MetaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "Description of Domain `{}`", self.domain_name)?;
        writeln!(f, "\tRecursion Structure: {}", self.recursion)?;
        if self.nullables.is_empty() {
            writeln!(f, "\tNullable Tasks: None")?;
        } else {
            writeln!(f, "\tNullable Tasks: {}", self.nullables.len())?;
            for nullable in self.nullables.iter() {
                writeln!(f, "\t\t{}", nullable)?
            }
        }
        writeln!(f, "\tNumber of actions: {}", self.n_actions)?;
        writeln!(f, "\tNumber of abstract tasks: {}", self.n_tasks)?;
        writeln!(f, "\tNumber of methods: {}", self.n_methods)?;
        Ok(())
    }
}
