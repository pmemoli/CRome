// type = Int | Long | UInt | ULong | Double
//     | FunType(type* params, type ret)
//     | Pointer(type referenced)
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum Type {
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    Pointer(Box<Type>),
    FunType(Vec<Type>, Box<Type>), // (param_types, return_type)
}

impl Type {
    pub fn byte_size(&self) -> usize {
        match self {
            Type::Int | Type::UInt | Type::Float => 4,
            Type::Long | Type::ULong => 8,
            Type::Double => 8,
            _ => panic!("Type {:?} does not have a defined byte size", self),
        }
    }

    pub fn signed(&self) -> bool {
        match self {
            Type::Int | Type::Long | Type::Float | Type::Double => true,
            Type::UInt | Type::ULong => false,
            _ => panic!("Type {:?} does not have a defined signedness", self),
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

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_floating_point()
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }
}
