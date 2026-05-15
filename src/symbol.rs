use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolTable {
    pub map: HashMap<String, SymbolInfo>,
}

#[derive(Debug)]
pub struct SymbolInfo {
    pub metadata: SymbolMetadata,
    pub ty: Type,
}

#[derive(Debug)]
pub enum SymbolMetadata {
    Function {
        defined: bool, // Defined function
        global: bool,  // External or internal linkage
    },
    StaticVariable {
        global: bool,                        // External linkage or internal/none
        initial_value: Option<InitialValue>, // None for uninitialized, Some for initialized
    },
    LocalVariable,
}

#[derive(Debug)]
pub enum InitialValue {
    Tentative,
    Initial(i32),
}

#[derive(PartialEq, Debug)]
pub enum Type {
    Int,
    FunType(usize), // Number of parameters
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, name: &String) -> Option<&SymbolInfo> {
        self.map.get(name)
    }

    pub fn unique_var_name(&self) -> String {
        let count = self.map.len();
        format!("var.{}", count)
    }

    pub fn insert_static_variable(
        &mut self,
        name: &String,
        global: bool,
        initial_value: Option<InitialValue>,
    ) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::StaticVariable {
                global,
                initial_value,
            },
            ty: Type::Int,
        };
        self.map.insert(name.clone(), info);
    }

    pub fn insert_local_variable(&mut self, name: &String) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::LocalVariable,
            ty: Type::Int,
        };
        self.map.insert(name.clone(), info);
    }

    pub fn insert_function(&mut self, name: &String, param_count: usize, defined: bool) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::Function {
                defined,
                global: true,
            },
            ty: Type::FunType(param_count),
        };
        self.map.insert(name.clone(), info);
    }
}
