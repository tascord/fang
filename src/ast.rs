use std::{
    fmt::{self, Debug, Formatter},
    mem,
    rc::Rc,
};

use crate::{errs::FangErr, scope::Scope, FILE_NAME};

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
pub struct Spans {
    line: String,
    line_span: (usize, usize),
    col_span: (usize, usize),
}

impl Spans {
    pub fn new(l: &str, s: ((usize, usize), (usize, usize))) -> Self {
        Self {
            line: l.to_string(),
            line_span: (s.0 .0, s.1 .0),
            col_span: (s.0 .1, s.1 .1),
        }
    }

    pub fn snippet(&self) -> String {
        vec![
            format!(
                "At {}:{}:{}",
                FILE_NAME.lock().unwrap(),
                self.line_span.0,
                self.col_span.0
            ),
            format!(""),
            format!("{}", self.line),
            format!(
                "{}{}",
                " ".repeat(self.col_span.0),
                "^".repeat(self.col_span.1 - self.col_span.0)
            ),
        ]
        .join("\n")
    }

    pub fn empty() -> Self {
        Self {
            line: "".to_string(),
            line_span: (0, 0),
            col_span: (0, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: Spans,
    },
    Subtract {
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: Spans,
    },
    Multiply {
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: Spans,
    },
    Divide {
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: Spans,
    },

    Integer {
        val: u64,
        span: Spans,
    },
    Float {
        val: f64,
        span: Spans,
    },
    String {
        val: String,
        span: Spans,
    },
    Boolean {
        val: bool,
        span: Spans,
    },

    Identifier {
        val: String,
        span: Spans,
    },
    Declaration {
        name: String,
        rhs: Option<Box<Node>>,
        var_type: Option<String>,
        span: Spans,
    },
    Assignment {
        name: String,
        rhs: Box<Node>,
        span: Spans,
    },
    TypedVariable {
        var_type: String,
        name: String,
        span: Spans,
    },
    SelfRef {
        span: Spans,
    },

    FunctionOutline {
        name: String,
        args: Box<Vec<Node>>,
        return_type: Option<String>,
        span: Spans,
    },
    Function {
        name: String,
        args: Box<Vec<Node>>,
        body: Box<Vec<Node>>,
        return_type: Option<String>,
        span: Spans,
    },
    Call {
        name: String,
        args: Box<Vec<Node>>,
        span: Spans,
    },
    BuiltinFn {
        name: String,
        args: Box<Vec<Node>>,
        body: BuiltinFnBody,
        return_type: Option<String>,
        span: Spans,
    },

    Struct {
        name: String,
        fields: Box<Vec<Node>>,
        span: Spans,
    },
    Object {
        typed: String,
        fields: Box<Vec<Node>>,
        span: Spans,
    },
    Field {
        name: String,
        value: Box<Node>,
        span: Spans,
    },

    Trait {
        name: String,
        fields: Box<Vec<Node>>,
        span: Spans,
    },
    TraitImpl {
        trait_name: String,
        type_name: String,
        fields: Box<Vec<Node>>,
        span: Spans,
    },

    Return {
        value: Box<Node>,
        span: Spans,
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
            Node::Integer { val, .. } => val.to_string(),
            Node::Float { val, .. } => val.to_string(),
            Node::String { val, .. } => val.to_string(),
            Node::Boolean { val, .. } => val.to_string(),
            Node::Identifier { val, .. } => val.to_string(),
            Node::TypedVariable { name, .. } => name.to_string(),
            Node::Function { name, .. } => format!("<Function: {name}>"),
            Node::Object { typed, fields, .. } => {
                format!(
                    "{typed} {{{}}}",
                    fields
                        .iter()
                        .map(|field| match field {
                            Node::Field { name, value, .. } =>
                                format!("{}: {}", name, value.inspect()),
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

    pub fn span(&self) -> Spans {
        match self {
            Node::Add { span, .. } => span.clone(),
            Node::Subtract { span, .. } => span.clone(),
            Node::Multiply { span, .. } => span.clone(),
            Node::Divide { span, .. } => span.clone(),

            Node::Integer { span, .. } => span.clone(),
            Node::Float { span, .. } => span.clone(),
            Node::String { span, .. } => span.clone(),
            Node::Boolean { span, .. } => span.clone(),

            Node::Identifier { span, .. } => span.clone(),
            Node::Declaration { span, .. } => span.clone(),
            Node::Assignment { span, .. } => span.clone(),
            Node::TypedVariable { span, .. } => span.clone(),
            Node::SelfRef { span, .. } => span.clone(),

            Node::FunctionOutline { span, .. } => span.clone(),
            Node::Function { span, .. } => span.clone(),
            Node::Call { span, .. } => span.clone(),
            Node::BuiltinFn { span, .. } => span.clone(),

            Node::Struct { span, .. } => span.clone(),
            Node::Object { span, .. } => span.clone(),
            Node::Field { span, .. } => span.clone(),

            Node::Trait { span, .. } => span.clone(),
            Node::TraitImpl { span, .. } => span.clone(),

            Node::Return { span, .. } => span.clone(),

            Node::Empty => Spans::empty(),
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
        Node::Add { lhs, rhs, span } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => Ok(Node::Integer {
                    val: a + b,
                    span: span.clone(),
                }),
                (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => Ok(Node::Float {
                    val: a + b,
                    span: span.clone(),
                }),
                (Node::String { val: a, .. }, Node::String { val: b, .. }) => Ok(Node::String {
                    val: a + &b,
                    span: span.clone(),
                }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    span,
                    op: "add".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        Node::Subtract { lhs, rhs, span } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => Ok(Node::Integer {
                    val: a - b,
                    span: span.clone(),
                }),
                (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => Ok(Node::Float {
                    val: a - b,
                    span: span.clone(),
                }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    span,
                    op: "subtract".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        Node::Multiply { lhs, rhs, span } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => Ok(Node::Integer {
                    val: a * b,
                    span: span.clone(),
                }),
                (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => Ok(Node::Float {
                    val: a * b,
                    span: span.clone(),
                }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    span,
                    op: "multiply".to_string(),
                    lhs: a.get_type(),
                    rhs: b.get_type(),
                    scope: scope.name.clone(),
                }),
            }
        }

        Node::Divide { lhs, rhs, span } => {
            let (a, b) = standardize_types(lhs, rhs, scope)?;
            match (a, b) {
                (Node::Integer { val: a, .. }, Node::Integer { val: b, .. }) => Ok(Node::Integer {
                    val: a / b,
                    span: span.clone(),
                }),
                (Node::Float { val: a, .. }, Node::Float { val: b, .. }) => Ok(Node::Float {
                    val: a / b,
                    span: span.clone(),
                }),
                (a, b) => Err(FangErr::OperationUnsupported {
                    span,
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
                    span: a.span(),
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
                    span: b.span(),
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
            Node::String {
                val: a.inspect(),
                span: a.span(),
            },
            Node::String {
                val: b.inspect(),
                span: b.span(),
            },
        ));
    }

    if a.is_float() || b.is_float() {
        let a = match *a {
            Node::Integer { val, span } => Node::Float {
                val: val as f64,
                span,
            },
            a => a,
        };

        let b = match *b {
            Node::Integer { val, span } => Node::Float {
                val: val as f64,
                span,
            },
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

    // TODO: Exhaust, span
    Err(FangErr::OperationUnsupported {
        span: a.span(),
        op: "coerce".to_string(),
        lhs: a.get_type(),
        rhs: b.get_type(),
        scope: scope.name.clone(),
    })
}
