use std::collections::HashMap;

pub struct SymbolInfo {
    pub stack_offset: u32, // Currently each variable takes 4 bytes
}

pub struct SymbolTable {
    map: HashMap<String, SymbolInfo>,
    label_idx_counter: u32,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            label_idx_counter: 0,
        }
    }

    pub fn generate_variable(&mut self) -> String {
        let info = SymbolInfo {
            stack_offset: (self.map.len() + 1) as u32 * 4,
        };
        let name = format!("var.{}", self.map.len());
        self.map.insert(name.clone(), info);
        name
    }

    pub fn generate_label_idx(&mut self) -> u32 {
        self.label_idx_counter += 1;
        self.label_idx_counter
    }

    pub fn stack_size(&self) -> u32 {
        (self.map.len() as u32) * 4
    }

    pub fn get(&self, name: &str) -> &SymbolInfo {
        self.map
            .get(name)
            .expect(&format!("Variable {} not found in symbol table", name))
    }
}
