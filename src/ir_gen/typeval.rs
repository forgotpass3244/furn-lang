use std::fmt::Display;


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeValEnum {
    Unit,
    Pointer(Box<TypeVal>),
    TaggedUnion(Vec<TypeVal>),
    UInt64,
    StringSlice,
    FunctionPointer(Vec<TypeVal>, Box<TypeVal>),
    MethodPointer, // also contains a reference to 'self'
}

impl TypeValEnum { 
    pub fn to_tval(self) -> TypeVal {
        TypeVal {
            t_enum: self,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeVal {
    pub t_enum: TypeValEnum,
}

impl Default for TypeVal {
    fn default() -> Self {
        Self {
            t_enum: TypeValEnum::Unit,
        }
    }
}

impl TypeVal {
    pub fn as_enum(&self) -> &TypeValEnum {
        &self.t_enum
    }

    pub fn is_ptr(&self) -> bool {
        match self.t_enum {
            TypeValEnum::Pointer(..) => true,
            _ => false,
        }
    }
    
    pub fn to_ptr(self) -> Self {
        TypeValEnum::Pointer(Box::new(self)).to_tval()
    }
    
    pub fn to_lessptr(self) -> Self {
        match self.as_enum() {
            TypeValEnum::Pointer(typeval) => *typeval.clone(),
            _ => self,
        }
    }
    
    pub fn size_of(&self) -> usize {
        match &self.t_enum {
            TypeValEnum::Unit => 0,
            TypeValEnum::Pointer(..) => 8,
            TypeValEnum::UInt64 => 8,
            TypeValEnum::StringSlice => 16,
            TypeValEnum::FunctionPointer(..) => 8,
            TypeValEnum::MethodPointer => 16,

            TypeValEnum::TaggedUnion(typevals) => {
                // LAYOUT
                // tag (8 bytes)
                // union (greatest size)

                let greatest = Self::greatest_size(typevals);
                greatest + 8 /*tag*/
            },
        }
    }

    pub fn greatest(typevals: &Vec<Self>) -> Self {
        let mut greatest: Option<&Self> = None;
        for typeval in typevals {
            let size = typeval.size_of();
            if let Some(cur_greatest) = greatest {
                if size > cur_greatest.size_of() {
                    greatest = Some(typeval);
                }
            } else {
                greatest = Some(typeval);
            }
        }

        greatest.cloned().unwrap_or(TypeValEnum::Unit.to_tval())
    }

    pub fn greatest_size(typevals: &Vec<Self>) -> usize {
        let greatest = Self::greatest(typevals);
        greatest.size_of()
    }
}

impl Display for TypeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let typeval = self;
        match typeval.as_enum() {
            TypeValEnum::Unit => write!(f, "void"),
            TypeValEnum::UInt64 => write!(f, "u64"),
            TypeValEnum::StringSlice => write!(f, "str"),
            TypeValEnum::FunctionPointer(..) => write!(f, "(...)"),
            TypeValEnum::MethodPointer => write!(f, "(self, ...)"),

            TypeValEnum::Pointer(sub_typeval) => {
                write!(f, "*{sub_typeval}")
            },

            TypeValEnum::TaggedUnion(typevals) => {
                write!(f, "(")?;
                
                for (i, typeval) in typevals.iter().enumerate() {
                    if (i + 1) >= typevals.len() {
                        write!(f, "{typeval}")?;
                    } else {
                        write!(f, "{typeval}|")?;
                    }
                }

                write!(f, ")")
            },
        }
    }
}
