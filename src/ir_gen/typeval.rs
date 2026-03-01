
#[derive(Clone, Debug)]
pub enum TypeValEnum {
    Unit,
    UInt64,
    StringSlice,
    FunctionPointer(Vec<TypeVal>, Box<TypeVal>),
    MethodPointer, // also contains a reference to 'self'
}

impl TypeValEnum { 
    pub fn to_tval(self) -> TypeVal {
        TypeVal {
            t_enum: self,
            is_ref: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeVal {
    pub t_enum: TypeValEnum,
    pub is_ref: bool,
}

impl TypeVal {
    pub fn as_enum(&self) -> &TypeValEnum {
        &self.t_enum
    }
    
    pub fn to_ref(mut self) -> Self {
        self.is_ref = true;
        self
    }
    
    pub fn to_nonref(mut self) -> Self {
        self.is_ref = false;
        self
    }
    
    pub fn size_of(&self) -> usize {
        if self.is_ref {
            8
        } else {
            match self.t_enum {
                TypeValEnum::Unit => 0,
                TypeValEnum::UInt64 => 8,
                TypeValEnum::StringSlice => 16,
                TypeValEnum::FunctionPointer(..) => 8,
                TypeValEnum::MethodPointer => 16,
            }
        }
    }
}
