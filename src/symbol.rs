use std::collections::HashMap;

#[derive(PartialEq)]
pub enum Type {
    Int,
    FunType(usize), // Number of parameters
}

pub enum SymbolInfo {
    Variable {
        stack_offset: u32, // Currently each variable takes 4 bytes
        ty: Type,
    },
    Function {
        defined: bool,
        ty: Type,
    },
}

pub struct SymbolTable {
    pub map: HashMap<String, SymbolInfo>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn generate_variable(&mut self, name: &String) {
        // stack offset is calculated based on the number of variables already in the map
        let var_count = self
            .map
            .values()
            .filter(|s| matches!(s, SymbolInfo::Variable { .. }))
            .count();

        let info = SymbolInfo::Variable {
            stack_offset: (var_count + 1) as u32 * 4,
            ty: Type::Int,
        };
        self.map.insert(name.clone(), info);
    }

    pub fn generate_function(&mut self, name: &String, param_count: usize, defined: bool) {
        let info = SymbolInfo::Function {
            defined: defined,
            ty: Type::FunType(param_count),
        };
        self.map.insert(name.clone(), info);
    }

    pub fn stack_size(&self) -> u32 {
        let var_count = self
            .map
            .values()
            .filter(|s| matches!(s, SymbolInfo::Variable { .. }))
            .count();

        (var_count as u32) * 4
    }
}
