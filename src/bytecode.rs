use crate::{
    ast::{standardize_types, BuiltinFnBody, Node},
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
    },

    Print,
    BuiltinCall {
        body: BuiltinFnBody,
    },
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
        Node::Object { fields } => ops.push(Op::Push {
            value: Node::Object { fields },
        }),
        Node::Function { name, args, body } => {
            ops.push(Op::Function {
                name,
                args: *args,
                body: *body,
            });
        }
        Node::Call { name, args } => {
            for arg in args.iter().rev() {
                ast_to_bytecode(arg.clone(), ops);
            }
            ops.push(Op::Call { name });
        }
        Node::BuiltinFn { body, .. } => ops.push(Op::BuiltinCall { body }),

        Node::Empty => (),
        Node::Out { val } => {
            ast_to_bytecode(*val, ops);
            ops.push(Op::Print {});
        }

        literal => ops.push(Op::Push { value: literal }),
    }
}

pub fn eval_bytecode(ast: Vec<Node>, scope: &mut Scope) -> Result<(), String> {
    let mut ops = Vec::new();

    for node in ast {
        ast_to_bytecode(node, &mut ops);
    }

    let mut stack = Vec::<Node>::new();
    for op in ops {
        match op {
            Op::Push { value } => stack.push(value),
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
                    _ => return Err("Failed to add".to_string()),
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
                    _ => return Err("Failed to subtract".to_string()),
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
                    _ => return Err("Failed to multiply".to_string()),
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
                        stack.push(Node::Integer { val: a / b })
                    }
                    (Node::Float { val: a }, Node::Float { val: b }) => {
                        stack.push(Node::Float { val: a / b })
                    }
                    _ => return Err("Failed to divide".to_string()),
                }
            }
            Op::Assign { name } => {
                let val = stack.pop().unwrap();
                scope.assign(name, val)?;
            }
            Op::Declare { name, var_type } => {
                let val = stack.pop().unwrap();

                if let Some(t) = var_type {
                    if t != val.get_type() {
                        return Err(format!(
                            "Specified type '{}' does not match given type, '{}'",
                            t,
                            val.get_type()
                        ));
                    }
                }

                scope.declare(name, val)?;
            }
            Op::Load { name } => {
                if let Some(value) = scope.get(&name) {
                    stack.push(value.clone());
                } else {
                    return Err(format!(
                        "Variable '{}' not found in scope {}",
                        name, scope.name
                    ));
                }
            }
            Op::Function { name, args, body } => {
                scope.put_fn(name, args, body)?;
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
                        return Err(format!(
                            "Type mismatch for argument '{}'.\nExpected {}, got {}.",
                            arg.inspect(),
                            arg.get_type(),
                            prop.get_type()
                        ));
                    }

                    props.push(prop);
                }

                scope.call(&name, props)?;
            }
            Op::BuiltinCall { body } => {
                if let Some(val) = body.0(scope) {
                    stack.push(val);
                };
            }

            Op::Print => println!("{}", stack.pop().unwrap().inspect()),
        }
    }

    Ok(())
}
