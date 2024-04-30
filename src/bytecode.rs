use crate::{
    ast::{standardize_types, BuiltinFnBody, Node},
    errs::FangErr,
    scope::Scope,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Push {
        value: Node,
    },

    Add,
    Subtract,
    Divide,
    Multiply,

    Assign {
        name: String,
    },
    Declare {
        name: String,
        var_type: Option<String>,
    },
    Load {
        name: String,
    },

    Call {
        name: String,
    },
    Function {
        name: String,
        args: Vec<Node>,
        body: Vec<Node>,
        return_type: Option<String>,
    },

    BuiltinCall {
        body: BuiltinFnBody,
    },

    Return,
}

pub fn ast_to_bytecode(node: Node, ops: &mut Vec<Op>) {
    match node {
        Node::Add { lhs, rhs } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Add {});
        }
        Node::Subtract { lhs, rhs } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Subtract {});
        }
        Node::Multiply { lhs, rhs } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Multiply {});
        }
        Node::Divide { lhs, rhs } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Divide {});
        }
        Node::Declaration {
            name,
            var_type,
            rhs,
        } => {
            if let Some(rhs) = rhs {
                ast_to_bytecode(*rhs, ops);
            }

            ops.push(Op::Declare { name, var_type });
        }
        Node::Assignment { name, rhs } => {
            ast_to_bytecode(*rhs, ops);
            ops.push(Op::Assign { name });
        }
        Node::Identifier { val } => {
            ops.push(Op::Load { name: val });
        }
        Node::Object { fields, typed } => ops.push(Op::Push {
            value: Node::Object { fields, typed },
        }),
        Node::Function {
            name,
            args,
            body,
            return_type,
        } => {
            ops.push(Op::Function {
                name,
                args: *args,
                body: *body,
                return_type,
            });
        }
        Node::Call { name, args } => {
            for arg in args.iter().rev() {
                ast_to_bytecode(arg.clone(), ops);
            }
            ops.push(Op::Call { name });
        }
        Node::BuiltinFn { body, .. } => ops.push(Op::BuiltinCall { body }),

        Node::Return { value } => {
            ast_to_bytecode(*value, ops);
            ops.push(Op::Return);
        }

        Node::Empty => (),
        literal => ops.push(Op::Push { value: literal }),
    }
}

pub fn eval_bytecode(ast: Vec<Node>, scope: &mut Scope) -> Result<Option<Node>, FangErr> {
    let mut ops = Vec::new();

    for node in ast {
        ast_to_bytecode(node, &mut ops);
    }

    let mut stack = Vec::<Node>::new();
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            Op::Push { value } => stack.push(value.clone()),
            Op::Add => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a }, Node::Integer { val: b }) => {
                        stack.push(Node::Integer { val: a + b })
                    }
                    (Node::Float { val: a }, Node::Float { val: b }) => {
                        stack.push(Node::Float { val: a + b })
                    }
                    (Node::String { val: a }, Node::String { val: b }) => {
                        stack.push(Node::String { val: a + &b })
                    }
                    (a, b) => {
                        return Err(FangErr::OperationUnsupported {
                            op: "add".to_string(),
                            lhs: a.inspect(),
                            rhs: b.inspect(),
                            scope: scope.name.clone(),
                        })
                    }
                }
            }
            Op::Subtract => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a }, Node::Integer { val: b }) => {
                        stack.push(Node::Integer { val: a - b })
                    }
                    (Node::Float { val: a }, Node::Float { val: b }) => {
                        stack.push(Node::Float { val: a - b })
                    }
                    (a, b) => {
                        return Err(FangErr::OperationUnsupported {
                            op: "subtract".to_string(),
                            lhs: a.inspect(),
                            rhs: b.inspect(),
                            scope: scope.name.clone(),
                        })
                    }
                }
            }
            Op::Multiply => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a }, Node::Integer { val: b }) => {
                        stack.push(Node::Integer { val: a * b })
                    }
                    (Node::Float { val: a }, Node::Float { val: b }) => {
                        stack.push(Node::Float { val: a * b })
                    }
                    (a, b) => {
                        return Err(FangErr::OperationUnsupported {
                            op: "multiply".to_string(),
                            lhs: a.inspect(),
                            rhs: b.inspect(),
                            scope: scope.name.clone(),
                        })
                    }
                }
            }
            Op::Divide => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a }, Node::Integer { val: b }) => {
                        stack.push(Node::Integer {
                            val: a.clone() / b.clone(),
                        })
                    }
                    (Node::Float { val: a }, Node::Float { val: b }) => stack.push(Node::Float {
                        val: a.clone() / b.clone(),
                    }),
                    (a, b) => {
                        return Err(FangErr::OperationUnsupported {
                            op: "divide".to_string(),
                            lhs: a.inspect(),
                            rhs: b.inspect(),
                            scope: scope.name.clone(),
                        })
                    }
                }
            }
            Op::Assign { name } => {
                let val = stack.pop().unwrap();
                scope.assign(name.clone(), val)?;
            }
            Op::Declare { name, var_type } => {
                let val = stack.pop().unwrap();

                if let Some(t) = var_type {
                    if *t != val.get_type() {
                        return Err(FangErr::TypeMismatch {
                            expected: t.clone(),
                            found: val.get_type(),
                            scope: scope.name.clone(),
                        });
                    }
                }

                scope.declare(name.clone(), val)?;
            }
            Op::Load { name } => {
                if let Some(value) = scope.get(&name) {
                    stack.push(value.clone());
                } else {
                    return Err(FangErr::UndeclaredVariable {
                        name: name.clone(),
                        scope: scope.name.clone(),
                    });
                }
            }
            Op::Function {
                name,
                args,
                body,
                return_type,
            } => {
                scope.put_fn(
                    name.clone(),
                    args.clone(),
                    body.clone(),
                    return_type.clone(),
                )?;
            }
            Op::Call { name } => {
                let args = scope.get_args(&name).expect(&format!(
                    "Function '{}' not found in scope {}",
                    name, scope.name
                ));
                let mut props = Vec::<Node>::new();
                for arg in args {
                    let prop = stack
                        .pop()
                        .expect(&format!("Expected argument for function '{}'", name));

                    if !prop.compare_type(&arg) {
                        return Err(FangErr::TypeMismatch {
                            expected: arg.get_type(),
                            found: prop.get_type(),
                            scope: scope.name.clone(),
                        });
                    }

                    props.push(prop);
                }

                let insert = scope.call(&name, props)?;
                ops = [ops[..=i].to_vec(), insert, ops[i + 1..].to_vec()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Op>>();

            }
            Op::BuiltinCall { body } => {
                if let Some(val) = body.0(scope) {
                    stack.push(val);
                };
            }

            Op::Return => {
                return Ok(stack.pop());
            }
        }

        i += 1;
    }

    // assert!(stack.is_empty());
    Ok(None)
}
