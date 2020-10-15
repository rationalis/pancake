use std::collections::HashMap;
use std::rc::Rc;

use crate::vm2::Identifier;
use crate::typeck::freshen_above;

type SimpleTypeRef = Rc<SimpleType>;
type TVars<'a> = &'a mut TVarRegistry;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeScheme {
    PolymorphicType {
        level: i32,
        body: SimpleTypeRef
    },
    SimpleType(SimpleTypeRef)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SimpleType {
    Function {
        lhs: SimpleTypeRef,
        rhs: SimpleTypeRef
    },
    Record {
        fields: HashMap<Identifier, SimpleTypeRef>
    },
    Primitive {
        name: Identifier
    },
    Variable(TVarId)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeVariable {
    pub level: i32,
    pub lower_bounds: Vec<SimpleTypeRef>,
    pub upper_bounds: Vec<SimpleTypeRef>
}

pub struct TVarRegistry {
    tvars: Vec<TypeVariable>
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TVarId(usize);

impl TVarRegistry {
    pub fn new() -> Self {
        Self {
            tvars: Vec::new()
        }
    }

    pub fn fresh_var(&mut self, level: i32) -> SimpleTypeRef {
        self.tvars.push(TypeVariable {
            level,
            lower_bounds: Vec::new(),
            upper_bounds: Vec::new()
        });
        Rc::new(SimpleType::Variable(TVarId(self.tvars.len() - 1)))
    }

    pub fn get(&self, key: &TVarId) -> &TypeVariable {
        self.tvars.get(key.0).unwrap()
    }

    pub fn get_mut(&mut self, key: &TVarId) -> &mut TypeVariable {
        self.tvars.get_mut(key.0).unwrap()
    }
}

impl TypeScheme {
    pub fn instantiate(&self, tvars: TVars, lvl: i32) -> SimpleTypeRef {
        use TypeScheme::*;
        match self {
            PolymorphicType { level, body } =>
                freshen_above(*level, body.clone(), tvars, lvl,
                              &mut HashMap::new()),
            SimpleType(typ) => typ.clone()
        }
    }

    pub fn level(&self, tvars: &mut TVarRegistry) -> i32 {
        use TypeScheme::*;
        match self {
            PolymorphicType { level, body: _ } => *level,
            SimpleType(typ) => typ.level(tvars)
        }
    }
}

impl From<SimpleType> for TypeScheme {
    fn from(st: SimpleType) -> Self {
        Self::SimpleType(Rc::new(st))
    }
}

impl From<Rc<SimpleType>> for TypeScheme {
    fn from(st: Rc<SimpleType>) -> Self {
        Self::SimpleType(st)
    }
}

impl SimpleType {
    pub fn level(&self, tvars: &mut TVarRegistry) -> i32 {
        use SimpleType::*;
        match self {
            Function { lhs, rhs } =>
                std::cmp::max(lhs.level(tvars), rhs.level(tvars)),
            Record { fields } => {
                let mut max = 0;
                for (_, typ) in fields {
                    max = std::cmp::max(max, typ.level(tvars));
                }
                max
            }
            Primitive { name: _ } => 0,
            Variable(key) => tvars.get(key).level
        }
    }

    pub fn tvar_id(&self) -> Option<TVarId> {
        match self {
            Self::Variable(key) => Some(*key),
            _ => None
        }
    }
}
