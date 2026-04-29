use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum Type {
    Int,
    FunType(usize), // Number of parameters
}

#[derive(Debug)]
pub enum SymbolMetadata {
    Variable {
        stack_offset: isize, // Currently each variable takes 4 bytes
    },
    Function {
        defined: bool,
    },
}

#[derive(Debug)]
pub struct SymbolInfo {
    pub metadata: SymbolMetadata,
    pub ty: Type,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub map: HashMap<String, SymbolInfo>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn generate_variable(&mut self) -> String {
        // stack offset is calculated based on the number of variables already in the map
        let var_count = self
            .map
            .values()
            .filter(|s| matches!(s.metadata, SymbolMetadata::Variable { .. }))
            .count();

        let info = SymbolInfo {
            metadata: SymbolMetadata::Variable {
                stack_offset: -(((var_count + 1) * 4) as isize),
            },
            ty: Type::Int,
        };

        let name = format!("var.{}", var_count + 1);

        self.map.insert(name.clone(), info);
        name
    }

    pub fn generate_function(&mut self, name: &String, param_count: usize, defined: bool) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::Function { defined },
            ty: Type::FunType(param_count),
        };
        self.map.insert(name.clone(), info);
    }

    pub fn stack_size(&self) -> usize {
        let var_count = self
            .map
            .values()
            .filter(|s| matches!(s.metadata, SymbolMetadata::Variable { .. }))
            .count();

        (var_count + 1) * 4
    }
}
