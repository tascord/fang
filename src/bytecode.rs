use std::collections::HashMap;

use crate::{
    ast::{standardize_types, BuiltinFnBody, Node, Spans},
    errs::FangErr,
    scope::Scope,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Push {
        value: Node,
    },

    Add {
        span: Spans,
    },
    Subtract {
        span: Spans,
    },
    Divide {
        span: Spans,
    },
    Multiply {
        span: Spans,
    },

    Assign {
        name: String,
        span: Spans,
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
        span: Spans,
    },
    Function {
        name: String,
        args: Vec<Node>,
        body: Vec<Node>,
        return_type: Option<String>,
        span: Spans,
    },

    BuiltinCall {
        body: BuiltinFnBody,
    },

    ImplTrait {
        trait_name: String,
        type_name: String,
        fields: Vec<Node>,
        span: Spans,
    },

    Return,
}

pub fn ast_to_bytecode(node: Node, ops: &mut Vec<Op>) {
    match node {
        Node::Add { lhs, rhs, span } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Add { span });
        }
        Node::Subtract { lhs, rhs, span } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Subtract { span });
        }
        Node::Multiply { lhs, rhs, span } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Multiply { span });
        }
        Node::Divide { lhs, rhs, span } => {
            ast_to_bytecode(*rhs, ops);
            ast_to_bytecode(*lhs, ops);
            ops.push(Op::Divide { span });
        }
        Node::Declaration {
            name,
            var_type,
            rhs,
            ..
        } => {
            if let Some(rhs) = rhs {
                ast_to_bytecode(*rhs, ops);
            }

            ops.push(Op::Declare { name, var_type });
        }
        Node::Assignment { name, rhs, span } => {
            ast_to_bytecode(*rhs, ops);
            ops.push(Op::Assign { name, span });
        }
        Node::Identifier { val, .. } => {
            ops.push(Op::Load { name: val });
        }
        Node::Object {
            fields,
            typed,
            span,
        } => ops.push(Op::Push {
            value: Node::Object {
                fields,
                typed,
                span,
            },
        }),
        Node::Function {
            name,
            args,
            body,
            return_type,
            span,
        } => {
            ops.push(Op::Function {
                name,
                args: *args,
                body: *body,
                return_type,
                span,
            });
        }
        Node::Call { name, args, span } => {
            for arg in args.iter().rev() {
                ast_to_bytecode(arg.clone(), ops);
            }
            ops.push(Op::Call { name, span });
        }
        Node::BuiltinFn { body, .. } => ops.push(Op::BuiltinCall { body }),

        Node::TraitImpl {
            trait_name,
            type_name,
            fields,
            span,
        } => {
            ops.push(Op::ImplTrait {
                trait_name,
                type_name,
                fields: *fields,
                span,
            });
        }

        Node::Return { value, .. } => {
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
            Op::Add { span } => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => {
                        stack.push(Node::Integer {
                            val: a + b,
                            span: span.clone(),
                        })
                    }
                    (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => {
                        stack.push(Node::Float {
                            val: a + b,
                            span: span.clone(),
                        })
                    }
                    (Node::String { val: a, .. }, Node::String { val: b, .. }) => {
                        stack.push(Node::String {
                            val: a + &b,
                            span: span.clone(),
                        })
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
            Op::Subtract { span } => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => {
                        stack.push(Node::Integer {
                            val: a - b,
                            span: span.clone(),
                        })
                    }
                    (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => {
                        stack.push(Node::Float {
                            val: a - b,
                            span: span.clone(),
                        })
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
            Op::Multiply { span } => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => {
                        stack.push(Node::Integer {
                            val: a * b,
                            span: span.clone(),
                        })
                    }
                    (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => {
                        stack.push(Node::Float {
                            val: a * b,
                            span: span.clone(),
                        })
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
            Op::Divide { span } => {
                let (a, b) = standardize_types(
                    stack.pop().unwrap().boxed(),
                    stack.pop().unwrap().boxed(),
                    scope,
                )?;
                match (a, b) {
                    (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => {
                        stack.push(Node::Integer {
                            val: a.clone() / b.clone(),
                            span: span.clone(),
                        })
                    }
                    (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => {
                        stack.push(Node::Float {
                            val: a.clone() / b.clone(),
                            span: span.clone(),
                        })
                    }
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
            Op::Assign { name, .. } => {
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
                ..
            } => {
                scope.put_fn(
                    name.clone(),
                    args.clone(),
                    body.clone(),
                    return_type.clone(),
                )?;
            }
            Op::Call { name , ..} => {
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
            Op::ImplTrait {
                trait_name,
                type_name,
                fields,
                ..
            } => {
                let mut m = HashMap::new();
                fields
                    .iter()
                    .map(|f| match f {
                        Node::Function {
                            name,
                            args,
                            body,
                            return_type,
                            ..
                        } => {
                            m.insert(
                                name.clone(),
                                (*args.clone(), *body.clone(), return_type.clone()),
                            );

                            Ok(())
                        }
                        _ => {
                            return Err(FangErr::UnexpectedToken {
                                expected: "Function".to_string(),
                                found: f.inspect(),
                                scope: scope.name.clone(),
                            })
                        }
                    })
                    .collect::<Result<Vec<_>, FangErr>>()?;

                scope.implement(trait_name.clone(), type_name.clone(), m)?;
            }
        }

        i += 1;
    }

    // assert!(stack.is_empty());
    Ok(None)
}
