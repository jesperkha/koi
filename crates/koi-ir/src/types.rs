use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRType {
    Primitive(Primitive),
    Function(Vec<IRType>, Box<IRType>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Primitive {
    Void,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    String,
}

const PTR_SIZE: usize = 8;

impl IRType {
    pub fn size(&self) -> usize {
        match self {
            IRType::Primitive(primitive) => match primitive {
                Primitive::Void => 0,
                Primitive::U8 | Primitive::I8 => 1,
                Primitive::U16 | Primitive::I16 => 2,
                Primitive::F32 | Primitive::I32 | Primitive::U32 => 4,
                Primitive::F64 | Primitive::U64 | Primitive::I64 | Primitive::String => PTR_SIZE,
            },
            IRType::Function(_, _) => 8,
        }
    }
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IRType::Primitive(p) => write!(f, "{}", format!("{:?}", p).to_lowercase()),
            IRType::Function(params, ret) => write!(
                f,
                "func({})->{}",
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                ret
            ),
        }
    }
}

pub type IRTypeId = usize;

pub struct IRTypeInterner {
    types: Vec<IRType>,
    cache: HashMap<IRType, IRTypeId>,
}

impl Default for IRTypeInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl IRTypeInterner {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn get_or_intern(&mut self, ty: IRType) -> IRTypeId {
        if let Some(id) = self.cache.get(&ty) {
            return *id;
        }
        let id = self.types.len();
        self.types.push(ty.clone());
        self.cache.insert(ty, id);
        id
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        for (id, ty) in self.types.iter().enumerate() {
            s += &format!("{} -> {}\n", id, ty);
        }
        s
    }

    pub fn type_to_string(&self, id: IRTypeId) -> String {
        self.types[id].to_string()
    }

    pub fn sizeof(&self, id: IRTypeId) -> usize {
        let ty = &self.types[id];
        ty.size()
    }

    pub fn get(&self, id: IRTypeId) -> &IRType {
        &self.types[id]
    }
}
