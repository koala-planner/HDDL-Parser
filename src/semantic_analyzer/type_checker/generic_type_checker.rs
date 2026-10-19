use core::panic;
use std::collections::HashSet;

use super::*;
use petgraph::visit::Reversed;

#[derive(Clone)]
pub struct TypeChecker<'a> {
    pub type_hierarchy: GraphMap<&'a str, (), Directed>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(types: &Option<Vec<Symbol<'a>>>) -> TypeChecker<'a> {
        let mut type_graph = GraphMap::new();
        type_graph.add_node("object");

        if let Some(type_deps) = types {
            for declared_type in type_deps {
                type_graph.add_node(declared_type.name);
                match declared_type.symbol_type {
                    Some(parent) => {
                        type_graph.add_edge(declared_type.name, parent, ());
                    }
                    None => {
                        type_graph.add_edge(declared_type.name, "object", ());
                    }
                }
            }
        }

        let roots: Vec<_> = type_graph
            .nodes()
            .filter(|n| *n != "object")
            .filter(|n| {
                type_graph
                    .neighbors_directed(*n, petgraph::Direction::Outgoing)
                    .next()
                    .is_none()
            })
            .collect();

        for root in roots {
            type_graph.add_edge(root, "object", ());
        }

        TypeChecker {
            type_hierarchy: type_graph,
        }
    }

    pub fn get_types(&self) -> HashSet<&'a str> {
        HashSet::from_iter(self.type_hierarchy.nodes())
    }

    pub fn get_supertypes(&self, of_type: &'a str) -> HashSet<&'a str> {
        let mut supertypes = HashSet::new();
        if !self.type_hierarchy.contains_node(of_type) {
            panic!("Type {} is not defined.", of_type)
        }
        let mut dfs = Dfs::new(&self.type_hierarchy, of_type);
        while let Some(node) = dfs.next(&self.type_hierarchy) {
            if node != of_type {
                supertypes.insert(node);
            }
        }
        supertypes
    }

    pub fn get_subtypes(&self, of_type: &'a str) -> HashSet<&'a str> {
        let mut subtypes = HashSet::new();
        if !self.type_hierarchy.contains_node(of_type) {
            panic!("Type {} is not defined.", of_type)
        }
        let reversed = Reversed(&self.type_hierarchy);
        let mut dfs = Dfs::new(reversed, of_type);
        while let Some(node) = dfs.next(reversed) {
            if node != of_type {
                subtypes.insert(node);
            }
        }
        subtypes
    }

    pub fn verify_type_hierarchy(&self) -> Result<(), SemanticErrorType> {
        match toposort(&self.type_hierarchy, None) {
            Ok(_) => Ok(()),
            Err(_) => Err(SemanticErrorType::CyclicTypeDeclaration),
        }
    }

    pub fn check_type_declarations(
        &self,
        parameters: &Vec<Symbol<'a>>,
    ) -> Option<SemanticErrorType> {
        for parameter in parameters.iter() {
            if let Some(t) = parameter.symbol_type
                && !self.type_hierarchy.contains_node(t)
            {
                return Some(SemanticErrorType::UndefinedType(UndefinedSymbolError {
                    symbol: t.to_string(),
                    position: parameter.type_pos.unwrap_or_default(),
                }));
            }
        }
        None
    }

    pub fn is_var_type_consistent(
        &self,
        found: Option<&'a str>,
        expected: Option<&'a str>,
    ) -> bool {
        match (found, expected) {
            (Some(found_typing), Some(defined_typing)) => {
                // type matches exactly
                if found_typing == defined_typing {
                    return true;
                }
                // search whether there is a path from current type to a super type
                has_path_connecting(&self.type_hierarchy, found_typing, defined_typing, None)
            }
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
        }
    }
}
