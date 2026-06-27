// type = Int | Long | UInt | ULong | Float | Double | FunType(type* params, type ret)
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum Type {
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    FunType(Vec<Type>, Box<Type>), // (param_types, return_type)
}

impl Type {
    pub fn byte_size(&self) -> usize {
        match self {
            Type::Int | Type::UInt | Type::Float => 4,
            Type::Long | Type::ULong => 8,
            Type::Double => 8,
            Type::FunType(_, _) => panic!("Functions do not have a byte size"),
        }
    }

    pub fn signed(&self) -> bool {
        match self {
            Type::Int | Type::Long | Type::Float | Type::Double => true,
            Type::UInt | Type::ULong => false,
            Type::FunType(_, _) => panic!("Functions do not have signedness"),
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Type::FunType(_, _))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int | Type::UInt | Type::Long | Type::ULong)
    }

    pub fn is_floating_point(&self) -> bool {
        matches!(self, Type::Double | Type::Float)
    }
}
