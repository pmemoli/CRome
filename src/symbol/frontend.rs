use super::*;

// Frontend Symbol Table
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub map: HashMap<String, SymbolInfo>,
    pub unique_counter: usize,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub metadata: SymbolMetadata,
    pub ty: Type,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum InitialValue {
    Tentative,
    Initial(StaticInit),
}

// static_init = IntInit(int) | LongInit(int) | UIntInit(int) | ULongInit(int) | DoubleInit(double) | FloatInit(float)
#[derive(Debug, Clone)]
pub enum StaticInit {
    IntInit(i32),
    UIntInit(u32),
    LongInit(i64),
    ULongInit(u64),
    FloatInit(f32),
    DoubleInit(f64),
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            unique_counter: 0,
        }
    }

    pub fn get(&self, name: &String) -> Option<&SymbolInfo> {
        self.map.get(name)
    }

    pub fn unique_var_name(&mut self) -> String {
        self.unique_counter += 1;
        format!("tmp.{}", self.unique_counter)
    }

    pub fn insert_static_variable(
        &mut self,
        name: &String,
        global: bool,
        initial_value: Option<InitialValue>,
        ty: &Type,
    ) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::StaticVariable {
                global,
                initial_value,
            },
            ty: ty.clone(),
        };
        self.map.insert(name.clone(), info);
    }

    pub fn insert_local_variable(&mut self, name: &String, ty: &Type) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::LocalVariable,
            ty: ty.clone(),
        };
        self.map.insert(name.clone(), info);
    }

    pub fn insert_function(&mut self, name: &String, ty: &Type, defined: bool, global: bool) {
        let info = SymbolInfo {
            metadata: SymbolMetadata::Function { defined, global },
            ty: ty.clone(),
        };
        self.map.insert(name.clone(), info);
    }

    pub fn identifier_type(&self, name: &String) -> Option<&Type> {
        self.map.get(name).map(|info| &info.ty)
    }
}
