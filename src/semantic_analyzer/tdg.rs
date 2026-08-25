use std::collections::{HashMap, HashSet, VecDeque};

use super::*;

pub struct Tdg<'a> {
    tasks: Vec<(&'a str, TaskType)>,
    methods: Vec<(&'a Symbol<'a>, HTN<'a>)>,
    edges_from_tasks: HashMap<usize, HashSet<usize>>,
    edges_to_tasks: HashMap<usize, HashSet<usize>>,
}

impl<'a> Tdg<'a> {
    pub fn new(domain: &'a DomainAST<'a>) -> Tdg<'a> {
        // collect task names
        let mut tasks: Vec<(&str, TaskType)> = vec![];
        tasks.extend(
            domain
                .compound_tasks
                .iter()
                .map(|x| (x.name, TaskType::Compound)),
        );
        tasks.extend(domain.actions.iter().map(|x| (x.name, TaskType::Primitive)));

        // edges
        let mut to_methods = HashMap::new();
        let mut to_tasks = HashMap::new();

        // compute index of tasks and methods for efficiency
        let mut task_indices = HashMap::new();
        for (index, (task, _)) in tasks.iter().enumerate() {
            task_indices.insert(*task, index);
            to_methods.insert(index, HashSet::new());
        }

        let mut methods = vec![];
        // collect "task to method" edges
        for (method_index, method) in domain.methods.iter().enumerate() {
            methods.push((&method.name, method.tn.clone()));
            match task_indices.get(method.task.name) {
                Some(task_index) => match to_methods.get_mut(task_index) {
                    Some(set) => {
                        set.insert(method_index);
                    }
                    None => panic!("{} not found", task_index),
                },
                None => panic!("{} is not defined", method.task.name),
            }
        }

        // collect "method to task" edges
        for (method_index, method) in methods.iter().enumerate() {
            let tasks: HashSet<usize> = method
                .1
                .subtasks
                .iter()
                .map(|x| match task_indices.get(x.task.name) {
                    Some(id) => *id,
                    None => panic!("{} not found", x.task.name),
                })
                .collect();
            to_tasks.insert(method_index, tasks);
        }
        Tdg {
            tasks,
            methods,
            edges_from_tasks: to_methods,
            edges_to_tasks: to_tasks,
        }
    }

    pub fn reachable(&self, task_name: &str) -> ReachableSet<'_> {
        let mut reach_t = HashSet::new();
        let task_index = match self
            .tasks
            .iter()
            .enumerate()
            .find(|(_, (name, _))| *name == task_name)
            .unwrap()
        {
            // if primitive, the only reachable task is itself
            (_, (name, TaskType::Primitive)) => {
                return ReachableSet {
                    primitives: HashSet::from([*name]),
                    compounds: HashSet::new(),
                    nullable: false,
                };
            }
            // if compound, add the index for further processing
            (i, (_, TaskType::Compound)) => i,
        };
        reach_t.insert(task_index);
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([task_index]);
        while !queue.is_empty() {
            let task = queue.pop_front().unwrap();
            if !visited.contains(&task) {
                visited.insert(task);
                if let Some(methods) = self.edges_from_tasks.get(&task) {
                    for m in methods {
                        let new_tasks = self.edges_to_tasks.get(m).unwrap();
                        for new_task in new_tasks.iter() {
                            reach_t.insert(*new_task);
                            queue.push_back(*new_task);
                        }
                    }
                }
            }
        }

        let nullables = self.compute_nullables();
        let mut primitives = HashSet::new();
        let mut compounds = HashSet::new();
        for (index, (reachable_name, reachable_type)) in self.tasks.iter().enumerate() {
            if reach_t.contains(&index) {
                match reachable_type {
                    TaskType::Primitive => {
                        primitives.insert(*reachable_name);
                    }
                    TaskType::Compound => {
                        compounds.insert(*reachable_name);
                    }
                }
            }
        }
        ReachableSet {
            primitives,
            compounds,
            nullable: nullables.contains(task_name),
        }
    }

    pub fn find_cycles(&self) -> Vec<Vec<(usize, usize)>> {
        let mut cycles = vec![];
        let num_tasks = self.tasks.len();

        // Run the circuit finder treating every single node as a root.
        for root in 0..num_tasks {
            let mut blocked = vec![false; num_tasks];
            let mut b_list = vec![vec![]; num_tasks];
            let mut path = vec![];

            self.circuit(
                root,
                root,
                &mut path,
                &mut cycles,
                &mut blocked,
                &mut b_list,
            );
        }
        cycles
    }

    fn circuit(
        &self,
        root: usize,
        current: usize,
        path: &mut Vec<(usize, usize)>,
        cycles: &mut Vec<Vec<(usize, usize)>>,
        blocked: &mut Vec<bool>,
        b_list: &mut Vec<Vec<usize>>,
    ) -> bool {
        let mut found_cycle = false;
        blocked[current] = true;

        if let Some(methods) = self.edges_from_tasks.get(&current) {
            for method in methods {
                if let Some(next_tasks) = self.edges_to_tasks.get(method) {
                    for next in next_tasks {
                        if *next == root {
                            // Cycle closed!
                            let mut cycle = path.clone();
                            cycle.push((current, *method));
                            cycles.push(cycle);
                            found_cycle = true;
                        } else if !blocked[*next] {
                            path.push((current, *method));

                            // Recurse deeper into the graph
                            if self.circuit(root, *next, path, cycles, blocked, b_list) {
                                found_cycle = true;
                            }

                            path.pop();
                        }
                    }
                }
            }
        }

        if found_cycle {
            self.unblock(current, blocked, b_list);
        } else {
            // If no cycle was found, add 'current' to the B-lists of its neighbors.
            // It stays blocked until one of its neighbors successfully finds a cycle.
            if let Some(methods) = self.edges_from_tasks.get(&current) {
                for method in methods {
                    if let Some(next_tasks) = self.edges_to_tasks.get(method) {
                        for next in next_tasks {
                            if *next != root && !b_list[*next].contains(&current) {
                                b_list[*next].push(current);
                            }
                        }
                    }
                }
            }
        }

        found_cycle
    }

    fn unblock(&self, v: usize, blocked: &mut Vec<bool>, b_list: &mut Vec<Vec<usize>>) {
        blocked[v] = false;
        let to_unblock = std::mem::take(&mut b_list[v]);
        for u in to_unblock {
            if blocked[u] {
                self.unblock(u, blocked, b_list);
            }
        }
    }

    /// Classifies the cycles produced by find_cycles and fills RecursionInfo.
    pub fn classify_cycles(&self, nullable_symbols: &HashSet<&'a str>) -> RecursionInfo {
        let cycles = self.find_cycles();
        let nullables: HashSet<usize> = nullable_symbols
            .iter()
            .map(|x| self.get_task_index(x))
            .collect();

        let mut info = RecursionInfo {
            acyclic_tasks: vec![],
            recursive_tasks: HashMap::new(),
            eps_prefix_tasks: HashMap::new(),
            empty_recursive_tasks: HashMap::new(),
            growing_empty_recursive_tasks: HashMap::new(),
            grow_and_shrink_tasks: HashMap::new(),
        };
        let mut on_some_cycle = HashSet::new();

        for cycle in &cycles {
            let k = cycle.len();
            let mut is_eps_prefix = true;
            let mut suffix: Vec<usize> = vec![];
            for i in 0..k {
                let (_, m_id) = cycle[i];
                let (t_id, _) = cycle[(i + 1) % k];
                let prefix = self.get_prefix(t_id, m_id);
                if !prefix.iter().all(|t| nullables.contains(t)) {
                    is_eps_prefix = false;
                }
                suffix.extend(self.get_suffix(t_id, m_id));
            }
            let suffix_nullable = suffix.iter().all(|t| nullables.contains(t));

            // --- record the cycle under its initiators ---
            let initiator_index = cycle[0].0;
            let initiator = self.tasks[initiator_index].0.to_string();
            on_some_cycle.insert(initiator_index);
            let named_cycle: Vec<_> = cycle
                .iter()
                .map(|(t_id, m_id)| {
                    (
                        self.tasks[*t_id].0.to_string(),
                        self.methods[*m_id].0.name.to_string(),
                    )
                })
                .collect();
            info.recursive_tasks
                .entry(initiator.clone())
                .or_default()
                .push(named_cycle.clone());
            if is_eps_prefix {
                info.eps_prefix_tasks
                    .entry(initiator.clone())
                    .or_default()
                    .push(named_cycle.clone());
                // check if it is an empty cycle
                if suffix.is_empty() {
                    info.empty_recursive_tasks
                        .entry(initiator.clone())
                        .or_default()
                        .push(named_cycle.clone());
                // suffix is not empty, hence it is either growing cycle or grow and shrink cycle
                } else {
                    info.growing_empty_recursive_tasks
                        .entry(initiator.clone())
                        .or_default()
                        .push(named_cycle.clone());
                    if suffix_nullable {
                        info.grow_and_shrink_tasks
                            .entry(initiator)
                            .or_default()
                            .push(named_cycle.clone());
                    }
                }
            }
        }
        // compound tasks that lie on no cycle
        info.acyclic_tasks = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(index, (_, t_type))| {
                matches!(t_type, TaskType::Compound) && !on_some_cycle.contains(index)
            })
            .map(|(_, (name, _))| name.to_string())
            .collect();
        info
    }

    fn get_prefix(&self, task_index: usize, method_index: usize) -> Vec<usize> {
        let (_, method) = &self.methods[method_index];
        let (task, _) = &self.tasks[task_index];
        match &method.orderings {
            TaskOrdering::Total => {
                let pos = method
                    .subtasks
                    .iter()
                    .position(|s| s.task.name == *task)
                    .unwrap_or_else(|| {
                        panic!(
                            "{} not in {:?}",
                            task,
                            method
                                .subtasks
                                .iter()
                                .map(|x| { x.task.name })
                                .collect::<Vec<_>>()
                        )
                    });
                method.subtasks[..pos]
                    .iter()
                    .map(|s| self.get_task_index(s.task.name))
                    .collect()
            }
            TaskOrdering::Partial(orderings) => {
                let id_to_task: HashMap<&str, &str> = method
                    .subtasks
                    .iter()
                    .filter_map(|s| s.id.as_ref().map(|i| (i.name, s.task.name)))
                    .collect();
                // reversed edges, id -> direct predecessors
                let mut preds: HashMap<&str, Vec<&str>> = HashMap::new();
                for (a, b) in orderings {
                    preds.entry(b).or_default().push(a);
                }
                // transitive predecessors of the occurrences of `task`
                let mut stack: Vec<&str> = method
                    .subtasks
                    .iter()
                    .filter(|s| s.task.name == *task)
                    .filter_map(|s| s.id.as_ref().map(|i| i.name))
                    .collect();
                let mut before: HashSet<&str> = HashSet::new();
                while let Some(t) = stack.pop() {
                    for &p in preds.get(t).into_iter().flatten() {
                        if before.insert(p) {
                            stack.push(p);
                        }
                    }
                }
                before
                    .iter()
                    .map(|id| self.get_task_index(id_to_task[id]))
                    .collect()
            }
        }
    }

    fn get_suffix(&self, task_index: usize, method_index: usize) -> Vec<usize> {
        let (_, method) = &self.methods[method_index];
        // multiset of indices to remove: the prefix, plus the consumed occurrence
        let mut remove: HashMap<usize, usize> = HashMap::new();
        for t in self.get_prefix(task_index, method_index) {
            *remove.entry(t).or_default() += 1;
        }
        *remove.entry(task_index).or_default() += 1;
        method
            .subtasks
            .iter()
            .map(|s| self.get_task_index(s.task.name))
            .filter(|t| match remove.get_mut(t) {
                Some(c) if *c > 0 => {
                    *c -= 1;
                    false
                }
                _ => true,
            })
            .collect()
    }

    fn get_task_index(&self, task_name: &str) -> usize {
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, (name, _))| *name == task_name)
            .unwrap()
            .0
    }

    pub fn compute_nullables(&self) -> HashSet<&'a str> {
        // nullable base case
        let mut nullables: HashSet<usize> = self
            .edges_from_tasks
            .iter()
            .filter_map(|(task, methods)| {
                for method in methods.iter() {
                    let tasks = self.edges_to_tasks.get(method).unwrap();
                    if tasks.is_empty() {
                        return Some(*task);
                    }
                }
                None
            })
            .collect();

        // unit reachability base case
        let mut unit_reachability: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (t, t_type) in self.tasks.iter() {
            match *t_type {
                TaskType::Primitive => {}
                TaskType::Compound => {
                    let task_index = self.get_task_index(t);
                    let mut value = HashSet::from([task_index]);
                    if let Some(methods) = self.edges_from_tasks.get(&task_index) {
                        for method in methods {
                            let tasks = self.edges_to_tasks.get(method).unwrap();
                            if tasks.len() == 1 {
                                value.insert(*tasks.iter().next().unwrap());
                            }
                        }
                    }

                    unit_reachability.insert(task_index, value);
                }
            }
        }
        let mut changed_nullables = true;
        let mut changed_unit_reachability = true;
        let mut new_nullables = HashSet::new();
        let mut new_unit_reachable: HashMap<usize, HashSet<usize>> = HashMap::new();
        while changed_nullables || changed_unit_reachability {
            // nullables induction step
            for (t, methods) in self.edges_from_tasks.iter() {
                for method in methods {
                    if let Some(tasks) = self.edges_to_tasks.get(method)
                        && tasks.iter().all(|x| match unit_reachability.get(x) {
                            Some(set) => {
                                let intersection: HashSet<&usize> =
                                    set.intersection(&nullables).collect();
                                !intersection.is_empty()
                            }
                            None => false,
                        })
                    {
                        new_nullables.insert(*t);
                    }
                }
            }

            // unit reachability induction step
            for (c, previous_reachables) in unit_reachability.iter() {
                let mut change = previous_reachables.clone();
                for previous_reachable in previous_reachables {
                    if let Some(tasks) = unit_reachability.get(previous_reachable) {
                        change = change.union(tasks).cloned().collect();
                    }
                }
                for method in self.edges_from_tasks.get(c).unwrap() {
                    if let Some(tasks) = self.edges_to_tasks.get(method) {
                        let mut not_nullable = None;
                        for task in tasks {
                            if !nullables.contains(task) {
                                if not_nullable.is_none() {
                                    not_nullable = Some(*task)
                                } else {
                                    break;
                                }
                            }
                        }
                        if let Some(val) = not_nullable {
                            change.insert(val);
                        }
                    }
                }
                if change == *previous_reachables {
                    changed_unit_reachability = false;
                } else {
                    new_unit_reachable.insert(*c, change);
                }
            }

            // commit to changes
            //// nullables
            if new_nullables.len() == nullables.len() {
                changed_nullables = false;
            } else {
                for n in new_nullables.iter() {
                    nullables.insert(*n);
                }
            }
            //// unit reachability
            for (task, new_reachable) in new_unit_reachable.iter() {
                let prev = unit_reachability.get_mut(task).unwrap();
                prev.extend(new_reachable);
            }
        }
        let mut result = HashSet::new();
        for task_index in nullables {
            result.insert(self.tasks[task_index].0);
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum TaskType {
    Primitive,
    Compound,
}

pub struct ReachableSet<'a> {
    pub primitives: HashSet<&'a str>,
    pub compounds: HashSet<&'a str>,
    pub nullable: bool,
}
