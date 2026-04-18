//! Dependency graph for incremental formula recalculation.

use crate::core::CellAddress;
use std::collections::{HashMap, HashSet, VecDeque};

/// Bidirectional dependency graph between cells
#[derive(Debug, Default, Clone)]
pub struct DepGraph {
    /// cell → set of cells it directly depends on (reads from)
    pub deps: HashMap<CellAddress, HashSet<CellAddress>>,
    /// cell → set of cells that depend on it (readers)
    pub rdeps: HashMap<CellAddress, HashSet<CellAddress>>,
}

impl DepGraph {
    pub fn new() -> Self { DepGraph::default() }

    /// Register that `cell` depends on all cells in `dependencies`
    pub fn set_deps(&mut self, cell: CellAddress, dependencies: HashSet<CellAddress>) {
        // Remove old reverse deps for this cell
        if let Some(old_deps) = self.deps.get(&cell) {
            for dep in old_deps.clone() {
                if let Some(rdeps) = self.rdeps.get_mut(&dep) {
                    rdeps.remove(&cell);
                }
            }
        }

        // Add new reverse deps
        for dep in &dependencies {
            self.rdeps.entry(dep.clone()).or_default().insert(cell.clone());
        }

        self.deps.insert(cell, dependencies);
    }

    /// Remove all dependency information for a cell (when cell content changes to non-formula)
    pub fn remove_cell(&mut self, cell: &CellAddress) {
        if let Some(old_deps) = self.deps.remove(cell) {
            for dep in old_deps {
                if let Some(rdeps) = self.rdeps.get_mut(&dep) {
                    rdeps.remove(cell);
                }
            }
        }
        self.rdeps.remove(cell);
    }

    /// Compute the minimal ordered set of cells that need recalculation
    /// when `changed` is modified. Returns cells in topological order.
    pub fn dirty_set(&self, changed: &CellAddress) -> Vec<CellAddress> {
        let mut result = vec![];
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // BFS over reverse dependency graph
        if let Some(first_rdeps) = self.rdeps.get(changed) {
            for dep in first_rdeps {
                if visited.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }

        while let Some(cell) = queue.pop_front() {
            result.push(cell.clone());
            if let Some(rdeps) = self.rdeps.get(&cell) {
                for dep in rdeps {
                    if visited.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        // Topological sort
        self.topological_sort(&result)
    }

    /// Topological sort of the given cells based on their dependencies
    fn topological_sort(&self, cells: &[CellAddress]) -> Vec<CellAddress> {
        let cell_set: HashSet<&CellAddress> = cells.iter().collect();
        let mut in_degree: HashMap<&CellAddress, usize> = HashMap::new();
        let mut adj: HashMap<&CellAddress, Vec<&CellAddress>> = HashMap::new();

        for cell in cells {
            in_degree.entry(cell).or_insert(0);
            if let Some(rdeps) = self.rdeps.get(cell) {
                for rdep in rdeps {
                    if cell_set.contains(rdep) {
                        adj.entry(cell).or_default().push(rdep);
                        *in_degree.entry(rdep).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut queue: VecDeque<&CellAddress> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(cell, _)| *cell)
            .collect();

        let mut result = vec![];
        while let Some(cell) = queue.pop_front() {
            result.push(cell.clone());
            if let Some(neighbors) = adj.get(cell) {
                for &neighbor in neighbors {
                    let deg = in_degree.entry(neighbor).or_insert(0);
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        result
    }

    /// Check if adding `new_dep` for `cell` would create a circular reference
    pub fn would_create_cycle(&self, cell: &CellAddress, new_dep: &CellAddress) -> bool {
        // DFS from new_dep to see if we can reach cell
        let mut visited = HashSet::new();
        let mut stack = vec![new_dep.clone()];
        while let Some(cur) = stack.pop() {
            if &cur == cell { return true; }
            if visited.insert(cur.clone()) {
                if let Some(deps) = self.deps.get(&cur) {
                    for d in deps { stack.push(d.clone()); }
                }
            }
        }
        false
    }

    pub fn dep_count(&self) -> usize { self.deps.len() }
    pub fn rdep_count(&self) -> usize { self.rdeps.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(row: u32, col: u32) -> CellAddress { CellAddress::new(row, col) }

    #[test]
    fn test_simple_dirty_set() {
        let mut graph = DepGraph::new();
        // A1 depends on B1
        graph.set_deps(addr(0, 0), [addr(0, 1)].into_iter().collect());
        // C1 depends on A1
        graph.set_deps(addr(0, 2), [addr(0, 0)].into_iter().collect());

        // When B1 changes, A1 and C1 should be dirty
        let dirty = graph.dirty_set(&addr(0, 1));
        assert!(dirty.contains(&addr(0, 0)));
        assert!(dirty.contains(&addr(0, 2)));
        // A1 should come before C1
        let a1_pos = dirty.iter().position(|a| a == &addr(0, 0)).unwrap();
        let c1_pos = dirty.iter().position(|a| a == &addr(0, 2)).unwrap();
        assert!(a1_pos < c1_pos);
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = DepGraph::new();
        graph.set_deps(addr(0, 0), [addr(0, 1)].into_iter().collect());
        assert!(!graph.would_create_cycle(&addr(0, 1), &addr(0, 2)));
        assert!(graph.would_create_cycle(&addr(0, 1), &addr(0, 0)));
    }
}
