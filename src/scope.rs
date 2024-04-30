use std::{collections::HashMap, rc::Rc, vec};

use once_cell::sync::Lazy;

use crate::{
    ast::{BuiltinFnBody, Node},
    bytecode::{ast_to_bytecode, eval_bytecode, Op},
    errs::FangErr,
};

pub const GLOBAL_SCOPE: Lazy<Scope> = Lazy::new(|| {
    let mut globe = Scope::new("<Fang>".to_string(), None);
    globe
        .declare(
            "console".to_string(),
            Node::Object {
                typed: "<Internal>".to_string(),
                fields: Box::new(vec![Node::Field {
                    name: "ln".to_string(),
                    value: Box::new(Node::BuiltinFn {
                        name: "ln".to_string(),
                        args: Box::new(vec![Node::TypedVariable {
                            var_type: "string".to_string(),
                            name: "out".to_string(),
                        }]),
                        body: BuiltinFnBody(Rc::new(|scope| {
                            let out = scope.get("out").unwrap();
                            println!("{}", out.inspect());
                            None
                        })),
                        return_type: None,
                    }),
                }]),
            },
        )
        .unwrap();

    globe
});

#[derive(Debug, Clone)]
pub enum Type {
    Trait {
        name: String,
        functions: HashMap<String, (Vec<Node>, Vec<Node>)>,
    },
    Struct {
        name: String,
        fields: Vec<Node>,
    },
}

impl Type {
    pub fn validate_trait(&self, name: &str, args: Vec<Node>) -> Result<(), FangErr> {
        match self {
            Type::Trait { functions, .. } => {
                let (fn_args, _) = functions.get(name).expect("Function not found");

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
    pub functions: HashMap<String, (Vec<Node>, Vec<Node>, Option<String>)>,
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

    pub fn get(&self, name: &str) -> Option<&Node> {
        if name.contains('.') {
            let mut parts = name.split('.').collect::<Vec<&str>>();
            parts.reverse();

            let mut container = self.store.get(parts.pop().unwrap()).or(self
                .parent
                .as_ref()
                .map(|p| p.get(name))
                .flatten());

            for part in parts {
                container = match container {
                    Some(Node::Object { fields, .. }) => fields.iter().find(|p| match p {
                        Node::Field { name, .. } => name == part,
                        _ => false,
                    }),
                    _ => None,
                };
            }

            match container {
                Some(Node::Field { value, .. }) => Some(value),
                _ => return None,
            }
        } else {
            self.store
                .get(name)
                .or(self.parent.as_ref().map(|p| p.get(name)).flatten())
        }
    }

    pub fn get_args(&self, name: &str) -> Option<Vec<Node>> {
        self.functions
            .get(name)
            .map(|(args, _, _)| args.clone())
            .or(self.get(name).map(|n| match n {
                Node::Function { args, .. } => args.iter().cloned().collect(),
                Node::BuiltinFn { args, .. } => args.iter().cloned().collect(),
                _ => vec![],
            }))
    }

    pub fn call(&self, name: &str, args: Vec<Node>) -> Result<Vec<Op>, FangErr> {
        let func = self.functions.get(name).cloned().or(self
            .get(name)
            .cloned()
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
                    }],
                    return_type,
                )),
                _ => None,
            })
            .flatten());

        let (fn_args, body, _) = func.ok_or(FangErr::UndeclaredVariable {
            name: name.to_string(),
            scope: self.name.clone(),
        })?;

        let mut scope = Scope::new(name.to_string(), Some(Box::new(self.clone())));
        for (arg, val) in fn_args.iter().zip(args.iter()) {
            scope.declare(
                match arg {
                    Node::Identifier { val } => val.clone(),
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

    pub fn define_struct(&mut self, name: String, fields: Vec<Node>) -> Result<(), String> {
        if self.types.contains_key(&name) {
            return Err(format!("Type '{}' already declared", name));
        }

        self.types
            .insert(name.clone(), Type::Struct { name, fields });
        Ok(())
    }

    pub fn define_trait(
        &mut self,
        name: String,
        functions: HashMap<String, (Vec<Node>, Vec<Node>)>,
    ) -> Result<(), String> {
        if self.types.contains_key(&name) {
            return Err(format!("Type '{}' already declared", name));
        }

        self.types
            .insert(name.clone(), Type::Trait { name, functions });
        Ok(())
    }
}
