use std::collections::HashMap;

pub enum Type {
    Int,
    FunType(u32), // Number of parameters
}

pub enum SymbolInfo {
    Variable {
        stack_offset: u32, // Currently each variable takes 4 bytes
        ty: Type,
    },
    Function {
        param_count: u32,
        return_ty: Type,
        defined: bool,
    },
}

pub struct SymbolTable {
    map: HashMap<String, SymbolInfo>,
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
            .filter(|s| matches!(s, SymbolInfo::Variable { .. }))
            .count();

        let info = SymbolInfo::Variable {
            stack_offset: (var_count + 1) as u32 * 4,
            ty: Type::Int,
        };
        let name = format!("var.{}", self.map.len());
        self.map.insert(name.clone(), info);
        name
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
