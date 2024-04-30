use std::{
    fmt::{self, Debug, Formatter}, mem, rc::Rc
};

use crate::{errs::FangErr, scope::Scope};

#[derive(Clone)]
pub struct BuiltinFnBody(pub Rc<dyn Fn(&Scope) -> Option<Node>>);
impl Debug for BuiltinFnBody {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<Builtin Function>")
    }
}

impl PartialEq for BuiltinFnBody {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Subtract {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Multiply {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Divide {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },

    Integer {
        val: u64,
    },
    Float {
        val: f64,
    },
    String {
        val: String,
    },
    Boolean {
        val: bool,
    },

    Identifier {
        val: String,
    },
    Declaration {
        name: String,
        rhs: Option<Box<Node>>,
        var_type: Option<String>,
    },
    Assignment {
        name: String,
        rhs: Box<Node>,
    },
    TypedVariable {
        var_type: String,
        name: String,
    },

    Function {
        name: String,
        args: Box<Vec<Node>>,
        body: Box<Vec<Node>>,
        return_type: Option<String>,
    },
    Call {
        name: String,
        args: Box<Vec<Node>>,
    },
    BuiltinFn {
        name: String,
        args: Box<Vec<Node>>,
        body: BuiltinFnBody,
        return_type: Option<String>,
    },

    Struct {
        name: String,
        fields: Box<Vec<Node>>,
    },
    Object {
        typed: String,
        fields: Box<Vec<Node>>,
    },
    Field {
        name: String,
        value: Box<Node>,
    },

    Trait {
        name: String,
        fields: Box<Vec<Node>>,
    },

    Return {
        value: Box<Node>,
    },

    Empty,
}

impl Node {
    pub fn is_int(&self) -> bool {
        match self {
            Node::Integer { .. } => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Node::Float { .. } => true,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            Node::String { .. } => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Node::Boolean { .. } => true,
            _ => false,
        }
    }

    pub fn is_id(&self) -> bool {
        match self {
            Node::Identifier { .. } => true,
            _ => false,
        }
    }

    pub fn is_op(&self) -> bool {
        match &self {
            Node::Add { .. } => true,
            Node::Subtract { .. } => true,
            Node::Multiply { .. } => true,
            Node::Divide { .. } => true,

            _ => false,
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Node::Integer { val } => val.to_string(),
            Node::Float { val } => val.to_string(),
            Node::String { val } => val.to_string(),
            Node::Boolean { val } => val.to_string(),
            Node::Identifier { val } => val.to_string(),
            Node::TypedVariable { name, .. } => name.to_string(),
            Node::Function { name, .. } => format!("<Function: {name}>"),
            Node::Object { typed, fields } => {
                format!(
                    "{typed} {{{}}}",
                    fields
                        .iter()
                        .map(|field| match field {
                            Node::Field { name, value } => format!("{}: {}", name, value.inspect()),
                            Node::Function { name, .. } => format!("<Function: {name}>"),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }

            a => format!("<Internal: {:?}>", a.get_type()),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Node::Integer { .. } => "int".to_string(),
            Node::Float { .. } => "float".to_string(),
            Node::String { .. } => "string".to_string(),
            Node::Boolean { .. } => "bool".to_string(),
            Node::TypedVariable { var_type, .. } => var_type.clone(),
            Node::Function { name, .. } => format!("<Function: '{}'>", name),

            _ => self.inspect(),
        }
    }

    pub fn boxed(self) -> Box<Node> {
        Box::new(self)
    }

    pub fn compare_type(&self, other: &Node) -> bool {
        match (self, other) {
            (Node::TypedVariable { var_type, .. }, n) => var_type == &n.get_type(),
            (n, Node::TypedVariable { var_type, .. }) => var_type == &n.get_type(),
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

fn eval_expr(expr: Node, scope: &Scope) -> Result<Node, FangErr> {
    match expr {
        Node::Add { lhs, rhs } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a }, Node::Integer { val: b }) => {
                    Ok(Node::Integer { val: a + b })
                }
                (Node::Float { val: a }, Node::Float { val: b }) => Ok(Node::Float { val: a + b }),
                (Node::String { val: a }, Node::String { val: b }) => {
                    Ok(Node::String { val: a + &b })
                }
                (a, b) => Err(FangErr::OperationUnsupported {
                    op: "add".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        Node::Subtract { lhs, rhs } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a }, Node::Integer { val: b }) => {
                    Ok(Node::Integer { val: a - b })
                }
                (Node::Float { val: a }, Node::Float { val: b }) => Ok(Node::Float { val: a - b }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    op: "subtract".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        Node::Multiply { lhs, rhs } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a }, Node::Integer { val: b }) => {
                    Ok(Node::Integer { val: a * b })
                }
                (Node::Float { val: a }, Node::Float { val: b }) => Ok(Node::Float { val: a * b }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    op: "multiply".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        Node::Divide { lhs, rhs } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a }, Node::Integer { val: b }) => {
                    Ok(Node::Integer { val: a / b })
                }
                (Node::Float { val: a }, Node::Float { val: b }) => Ok(Node::Float { val: a / b }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    op: "divide".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        a => Ok(a),
    }
}

pub fn standardize_types(
    mut a: Box<Node>,
    mut b: Box<Node>,
    scope: &Scope,
) -> Result<(Node, Node), FangErr> {
    if a.is_id() {
        a = match scope.get(&a.inspect()) {
            Some(n) => Box::new(n.clone()),
            None => {
                return Err(FangErr::UndeclaredVariable {
                    name: a.inspect(),
                    scope: scope.name.clone(),
                })
            }
        };
    }

    if b.is_id() {
        b = match scope.get(&b.inspect()) {
            Some(n) => Box::new(n.clone()),
            None => {
                return Err(FangErr::UndeclaredVariable {
                    name: a.inspect(),
                    scope: scope.name.clone(),
                })
            }
        };
    }

    while a.is_op() {
        a = Box::new(eval_expr(*a, scope)?);
    }

    while b.is_op() {
        b = Box::new(eval_expr(*b, scope)?);
    }

    if a.is_str() || b.is_str() {
        return Ok((
            Node::String { val: a.inspect() },
            Node::String { val: b.inspect() },
        ));
    }

    if a.is_float() || b.is_float() {
        let a = match *a {
            Node::Integer { val } => Node::Float { val: val as f64 },
            a => a,
        };

        let b = match *b {
            Node::Integer { val } => Node::Float { val: val as f64 },
            b => b,
        };

        return Ok((a, b));
    }

    if a.is_int() && b.is_int() {
        return Ok((*a, *b));
    }

    if a.is_bool() && b.is_bool() {
        return Ok((*a, *b));
    }

    // TODO: Exhaust
    Err(FangErr::OperationUnsupported {
        op: "coerce".to_string(),
        lhs: a.get_type(),
        rhs: b.get_type(),
        scope: scope.name.clone(),
    })
}
