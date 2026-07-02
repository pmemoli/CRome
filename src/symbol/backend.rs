use super::*;

#[derive(Debug, Clone)]
pub struct BackendSymbolTable {
    pub map: HashMap<String, BackendSymbolMetadata>,
}

// assembly_type = Longword | Quadword | Double
#[derive(Debug, Clone)]
pub enum AssemblyType {
    Longword,
    Quadword,
    Float,
    Double,
}

impl AssemblyType {
    pub fn size(&self) -> usize {
        match self {
            Self::Longword => 4,
            Self::Float => 4,
            Self::Quadword => 8,
            Self::Double => 8,
        }
    }
    pub fn alignment(&self) -> usize {
        match self {
            Self::Longword => 4,
            Self::Float => 4,
            Self::Quadword => 8,
            Self::Double => 8,
        }
    }
    pub fn is_floating_point(&self) -> bool {
        match self {
            Self::Float | Self::Double => true,
            _ => false,
        }
    }
    pub fn is_integer(&self) -> bool {
        match self {
            Self::Longword | Self::Quadword => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BackendSymbolMetadata {
    ObjEntry { ty: AssemblyType, is_static: bool },
    FunEntry { defined: bool },
}

impl BackendSymbolTable {
    pub fn new(frontend_symbol_table: SymbolTable) -> Self {
        let mut new_map = HashMap::new();

        for (name, info) in frontend_symbol_table.map.iter() {
            match &info.metadata {
                SymbolMetadata::Function { defined, .. } => {
                    let entry = BackendSymbolMetadata::FunEntry { defined: *defined };
                    new_map.insert(name.clone(), entry);
                }
                SymbolMetadata::StaticVariable { .. } | SymbolMetadata::LocalVariable => {
                    let ty = match info.ty {
                        Type::Int => AssemblyType::Longword,
                        Type::UInt => AssemblyType::Longword,
                        Type::Long => AssemblyType::Quadword,
                        Type::ULong => AssemblyType::Quadword,
                        Type::Double => AssemblyType::Double,
                        Type::Float => AssemblyType::Float,
                        Type::FunType(_, _) => panic!("Functions should not be treated as objects"),
                    };

                    let is_static = matches!(info.metadata, SymbolMetadata::StaticVariable { .. });

                    let entry = BackendSymbolMetadata::ObjEntry { ty, is_static };
                    new_map.insert(name.clone(), entry);
                }
            }
        }

        Self { map: new_map }
    }

    pub fn get(&self, name: &String) -> Option<&BackendSymbolMetadata> {
        self.map.get(name)
    }

    pub fn insert_object(&mut self, name: &String, ty: AssemblyType, is_static: bool) {
        let info = BackendSymbolMetadata::ObjEntry {
            ty: ty.clone(),
            is_static,
        };
        self.map.insert(name.clone(), info);
    }

    pub fn insert_function(&mut self, name: &String, defined: bool) {
        let info = BackendSymbolMetadata::FunEntry { defined };
        self.map.insert(name.clone(), info);
    }

    pub fn identifier_type(&self, name: &String) -> Option<&AssemblyType> {
        match self.map.get(name) {
            Some(BackendSymbolMetadata::ObjEntry { ty, .. }) => Some(ty),
            _ => None,
        }
    }
}
