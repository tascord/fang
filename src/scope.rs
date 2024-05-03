use std::{collections::HashMap, rc::Rc, vec};

use once_cell::sync::Lazy;

use crate::{
    ast::{BuiltinFnBody, Node, Spans},
    bytecode::{ast_to_bytecode, eval_bytecode, Op},
    errs::FangErr,
};

type Func = (Vec<Node>, Vec<Node>, Option<String>);

macro_rules! builtin_fn {
    ($name:literal, $body:expr, $ret:expr, $( $an: literal, $at: literal ),*) => {
        Node::BuiltinFn {
            span: Spans::empty(),
            name: $name.to_string(),
            body: BuiltinFnBody(Rc::new($body)),
            return_type: $ret,
            args: Box::new(vec![
                $(
                    Node::TypedVariable {
                        span: Spans::empty(),
                        var_type: $at.to_string(),
                        name: $an.to_string(),
                    }
                ),*
            ]),
        }
    };
}

macro_rules! builtin_obj {
    ($name:literal, $( $fn:literal, $fv: expr );*) => {
        Node::Object {
            span: Spans::empty(),
            typed: "<Internal>".to_string(),
            fields: Box::new(vec![
                $(
                    Node::Field {
                        name: $fn.to_string(),
                        value: Box::new($fv),
                        span: Spans::empty()
                    }
                ),*
            ]),
        }
    };
}

macro_rules! builtin {
    ($g:ident, $n:literal, $b:expr) => {
        $g.declare($n.to_string(), $b).unwrap()
    };
}

pub const GLOBAL_SCOPE: Lazy<Scope> = Lazy::new(|| {
    let mut globe = Scope::new("<Fang>".to_string(), None);

    builtin!(
        globe,
        "console",
        builtin_obj!(
            "console",
            "log",
            builtin_fn!(
                "log",
                |args| {
                    println!("{}", args.get("msg").unwrap().inspect());
                    None
                },
                None,
                "msg",
                "string"
            )
        )
    );

    // globe.define_trait("ToString".to_string(), {
    //     let mut m = HashMap::new();
    //     m.insert("to_string", (
    //         vec![Node::TypedVariable {
    //             var_type: "Self".to_string(),
    //             name: "self".to_string(),
    //         }],
    //     ))
    // })

    globe
});

#[derive(Debug, Clone)]
pub enum TraitFn {
    Default {
        name: String,
        args: Vec<Node>,
        body: Vec<Node>,
        return_type: Option<String>,
    },
    NoBody {
        name: String,
        args: Vec<Node>,
        return_type: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum Type {
    Trait {
        name: String,
        functions: HashMap<String, TraitFn>,
    },
    Struct {
        name: String,
        fields: Vec<Node>,
        implements: Vec<String>,                     // traits
        implementations: Vec<HashMap<String, Func>>, // fns from traits
    },
}

impl Type {
    pub fn validate_trait(&self, name: &str, args: Vec<Node>) -> Result<(), FangErr> {
        match self {
            Type::Trait { functions, .. } => {
                let fn_args = match functions.get(name).ok_or(FangErr::UndeclaredFunction {
                    name: name.to_string(),
                    scope: "?".to_string(),
                })? {
                    TraitFn::Default { args, .. } => args,
                    TraitFn::NoBody { args, .. } => args,
                };

                if fn_args.len() != args.len() {
                    return Err(FangErr::ArgumentLengthMismatch {
                        expected: fn_args.len(),
                        found: args.len(),
                        scope: name.to_string(),
                    });
                }

                for (arg, val) in fn_args.iter().zip(args.iter()) {
                    if arg.compare_type(val) {
                        return Err(FangErr::TypeMismatch {
                            expected: arg.get_type(),
                            found: val.get_type(),
                            scope: name.to_string(),
                        });
                    }
                }

                Ok(())
            }
            _ => Err(FangErr::UnexpectedType {
                expected: "Trait".to_string(),
                found: name.to_string(),
                scope: name.to_string(),
            }),
        }
    }

    pub fn validate_struct(&self, name: &str, fields: Vec<Node>) -> Result<(), FangErr> {
        match self {
            Type::Struct {
                fields: expected, ..
            } => {
                if expected.len() != fields.len() {
                    return Err(FangErr::ArgumentLengthMismatch {
                        expected: expected.len(),
                        found: fields.len(),
                        scope: name.to_string(),
                    });
                }

                for (exp, val) in expected.iter().zip(fields.iter()) {
                    if exp.compare_type(val) {
                        return Err(FangErr::TypeMismatch {
                            expected: exp.get_type(),
                            found: val.get_type(),
                            scope: name.to_string(),
                        });
                    }
                }

                Ok(())
            }
            _ => Err(FangErr::UnexpectedType {
                expected: "Struct".to_string(),
                found: name.to_string(),
                scope: name.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub store: HashMap<String, Node>,
    pub functions: HashMap<String, Func>,
    pub types: HashMap<String, Type>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new(name: String, parent: Option<Box<Scope>>) -> Self {
        Scope {
            name,
            store: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            parent,
        }
    }

    pub fn declare(&mut self, name: String, val: Node) -> Result<(), FangErr> {
        if self.store.contains_key(&name) {
            return Err(FangErr::AlreadyDeclaredVariable {
                name,
                scope: self.name.clone(),
            });
        }

        self.store.insert(name, val);
        Ok(())
    }

    pub fn assign(&mut self, name: String, val: Node) -> Result<(), FangErr> {
        if !self.store.contains_key(&name) {
            return Err(FangErr::UndeclaredVariable {
                name,
                scope: self.name.clone(),
            });
        }

        if !self.store.get(&name).unwrap().compare_type(&val) {
            return Err(FangErr::TypeMismatch {
                expected: self.store.get(&name).unwrap().get_type(),
                found: val.get_type(),
                scope: self.name.clone(),
            });
        }

        self.store.insert(name, val);
        Ok(())
    }

    pub fn put_fn(
        &mut self,
        name: String,
        args: Vec<Node>,
        body: Vec<Node>,
        return_type: Option<String>,
    ) -> Result<(), FangErr> {
        if self.functions.contains_key(&name) {
            return Err(FangErr::AlreadyDeclaredFunction {
                name,
                scope: self.name.clone(),
            });
        }

        if return_type.is_some() {}

        self.functions.insert(name, (args, body, return_type));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Node> {
        if name.contains('.') {
            let mut parts = name.split('.').collect::<Vec<&str>>();
            parts.reverse();

            let mut container = self.store.get(parts.pop().unwrap()).cloned().or(self
                .parent
                .as_ref()
                .map(|p| p.get(name))
                .flatten());

            for part in parts {
                container = match container {
                    Some(Node::Object { fields, typed, .. }) => fields
                        .iter()
                        .find(|p| match p {
                            Node::Field { name, .. } => name == part,
                            _ => false,
                        })
                        .cloned()
                        .or(self.get_implementations_for(&typed).iter().find_map(|i| {
                            i.get(part).map(|f| Node::Function {
                                name: part.to_string(),
                                args: Box::new(f.0.clone()),
                                body: Box::new(f.1.clone()),
                                return_type: f.2.clone(),
                                span: Spans::empty(),
                            })
                        })),
                    _ => None,
                };
            }

            match container {
                Some(Node::Field { value, .. }) => Some(*value),
                _ => return None,
            }
        } else {
            self.store
                .get(name)
                .cloned()
                .or(self.parent.as_ref().map(|p| p.get(name)).flatten())
        }
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types
            .get(name)
            .or(self.parent.as_ref().map(|p| p.get_type(name)).flatten())
    }

    pub fn get_args(&self, name: &str) -> Option<Vec<Node>> {
        self.get_fn(name)
            .map(|(args, _, _)| args.clone())
            .or(self.get(name).map(|n| match n {
                Node::Function { args, .. } => args.iter().cloned().collect(),
                Node::BuiltinFn { args, .. } => args.iter().cloned().collect(),
                _ => vec![],
            }))
    }

    pub fn get_fn(&self, name: &str) -> Option<Func> {
        self.functions
            .get(name)
            .cloned()
            .or(self
                .get(name)
                .map(|n| match n {
                    Node::Function {
                        args,
                        body,
                        return_type,
                        ..
                    } => Some((*args, *body, return_type)),
                    Node::BuiltinFn {
                        name,
                        args,
                        body,
                        return_type,
                        ..
                    } => Some((
                        *args.clone(),
                        vec![Node::BuiltinFn {
                            name: name.clone(),
                            args: args.clone(),
                            body: body.clone(),
                            return_type: return_type.clone(),
                            span: Spans::empty(),
                        }],
                        return_type,
                    )),
                    _ => None,
                })
                .flatten())
            .or(self.parent.as_ref().map(|p| p.get_fn(name)).flatten())
    }

    pub fn call(&self, name: &str, args: Vec<Node>) -> Result<Vec<Op>, FangErr> {
        let func = self.get_fn(name);
        let (fn_args, body, _) = func.ok_or(FangErr::UndeclaredVariable {
            name: name.to_string(),
            scope: self.name.clone(),
        })?;

        let mut scope = Scope::new(name.to_string(), Some(Box::new(self.clone())));
        for (arg, val) in fn_args.iter().zip(args.iter()) {
            scope.declare(
                match arg {
                    Node::Identifier { val, .. } => val.clone(),
                    Node::TypedVariable { name, .. } => name.clone(),
                    _ => {
                        return Err(FangErr::UnexpectedToken {
                            expected: "Identifier".to_string(),
                            found: val.get_type(),
                            scope: name.to_string(),
                        })
                    }
                },
                val.clone(),
            )?;
        }

        let mut ops = Vec::<Op>::new();
        eval_bytecode(body.to_vec(), &mut scope)?.map(|n| ast_to_bytecode(n, &mut ops));
        Ok(ops)
    }

    pub fn define_struct(&mut self, name: String, fields: Vec<Node>) -> Result<(), FangErr> {
        if self.types.contains_key(&name) {
            return Err(FangErr::AlreadyDeclaredStruct {
                name: name,
                scope: self.name.clone(),
            });
        }

        self.types.insert(
            name.clone(),
            Type::Struct {
                name,
                fields,
                implements: Vec::new(),
                implementations: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn define_trait(
        &mut self,
        name: String,
        functions: HashMap<String, TraitFn>,
    ) -> Result<(), FangErr> {
        if self.types.contains_key(&name) {
            return Err(FangErr::AlreadyDeclaredTrait {
                name,
                scope: self.name.clone(),
            });
        }

        self.types
            .insert(name.clone(), Type::Trait { name, functions });
        Ok(())
    }

    pub fn get_mut_type(&mut self, name: String) -> Option<&mut Type> {
        self.types
            .get_mut(&name)
            .or(self.parent.as_mut().map(|p| p.get_mut_type(name)).flatten())
    }

    pub fn implement(
        &mut self,
        struct_name: String,
        trait_name: String,
        implementation: HashMap<String, Func>,
    ) -> Result<(), FangErr> {
        let scope_name = self.name.clone();

        self.get_type(&trait_name)
            .map(|t| {
                Ok(match t {
                    Type::Trait { functions, .. } => {
                        for (name, args, ret) in functions
                            .iter()
                            .filter(|f| matches!(f.1, TraitFn::NoBody { .. }))
                            .map(|f| match f {
                                (
                                    name,
                                    TraitFn::NoBody {
                                        args, return_type, ..
                                    },
                                ) => (name.clone(), args.clone(), return_type.clone()),
                                _ => unreachable!(),
                            })
                            .collect::<Vec<(String, Vec<Node>, Option<String>)>>()
                        {
                            let imple =
                                implementation
                                    .get(&name)
                                    .ok_or(FangErr::UndeclaredFunction {
                                        name: name.clone(),
                                        scope: scope_name.clone(),
                                    })?;

                            for (i, arg) in args.iter().enumerate() {
                                match (
                                    arg,
                                    imple.0.get(i).ok_or(FangErr::UndeclaredVariable {
                                        name: name.clone(),
                                        scope: scope_name.clone(),
                                    })?,
                                ) {
                                    (Node::SelfRef { .. }, Node::SelfRef { .. }) => (),
                                    (
                                        Node::TypedVariable { var_type, .. },
                                        Node::TypedVariable {
                                            var_type: imple_type,
                                            ..
                                        },
                                    ) => {
                                        if var_type != imple_type {
                                            return Err(FangErr::TypeMismatch {
                                                expected: var_type.clone(),
                                                found: imple_type.clone(),
                                                scope: scope_name.clone(),
                                            });
                                        }
                                    }
                                    _ => {
                                        return Err(FangErr::UnexpectedToken {
                                            expected: "TypedVariable".to_string(),
                                            found: arg.get_type(),
                                            scope: scope_name.clone(),
                                        })
                                    }
                                };
                            }

                            match (ret, imple.2.clone()) {
                                (Some(rt), Some(irt)) => {
                                    if rt != irt {
                                        return Err(FangErr::TypeMismatch {
                                            expected: rt.clone(),
                                            found: irt.clone(),
                                            scope: scope_name.clone(),
                                        });
                                    }
                                }
                                (None, None) => (),
                                _ => {
                                    return Err(FangErr::UnexpectedToken {
                                        expected: "None".to_string(),
                                        found: "Some".to_string(),
                                        scope: scope_name.clone(),
                                    })
                                }
                            };
                        }
                    }
                    _ => {
                        return Err(FangErr::UnexpectedType {
                            expected: "Trait".to_string(),
                            found: trait_name.clone(),
                            scope: scope_name.clone(),
                        })
                    }
                })
            })
            .unwrap_or(Err(FangErr::UndeclaredType {
                name: trait_name.clone(),
                scope: scope_name.clone(),
            }))?;

        let ty = self
            .get_mut_type(struct_name.clone())
            .ok_or(FangErr::UndeclaredType {
                name: struct_name.clone(),
                scope: scope_name.clone(),
            })?;

        match ty {
            Type::Struct { implements, .. } => {
                if implements.contains(&trait_name) {
                    return Err(FangErr::AlreadyImplementedTrait {
                        name: trait_name,
                        scope: self.name.clone(),
                    });
                }

                implements.push(trait_name);
            }
            _ => {
                return Err(FangErr::UnexpectedType {
                    expected: "Struct".to_string(),
                    found: struct_name,
                    scope: scope_name,
                })
            }
        }

        Ok(())
    }

    pub fn get_implementations_for(&self, name: &str) -> Vec<HashMap<String, Func>> {
        self.get_type(name)
            .map(|ty| match ty {
                Type::Struct { implements, .. } => implements
                    .clone()
                    .iter()
                    .map(|i| {
                        self.get_type(i)
                            .map(|t| match t {
                                Type::Trait { functions, .. } => functions
                                    .into_iter()
                                    .filter(|f| match f {
                                        (_, TraitFn::Default { .. }) => true,
                                        _ => false,
                                    })
                                    .map(|(name, f)| {
                                        (
                                            name,
                                            match f {
                                                TraitFn::Default {
                                                    args,
                                                    body,
                                                    return_type,
                                                    ..
                                                } => (
                                                    args.clone(),
                                                    body.clone(),
                                                    return_type.clone(),
                                                ),
                                                _ => unreachable!(),
                                            },
                                        )
                                    })
                                    .map(|(a, b)| (a.to_owned(), b.to_owned()))
                                    .collect(),
                                _ => unreachable!(),
                            })
                            .unwrap()
                    })
                    .collect(),
                _ => unreachable!(),
            })
            .unwrap_or(Vec::new())
    }
}
