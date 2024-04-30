use std::fmt::Display;

#[derive(Debug)]
pub enum FangErr {
    TypeMismatch { expected: String, found: String, scope: String },
    OperationUnsupported { op: String, lhs: String, rhs: String, scope: String, },
    UndeclaredVariable { name: String, scope: String },
    AlreadyDeclaredVariable { name: String, scope: String },
    AlreadyDeclaredFunction { name: String, scope: String },
    UnexpectedToken { expected: String, found: String, scope: String },
    ArgumentLengthMismatch { expected: usize, found: usize, scope: String },
    UnexpectedType { expected: String, found: String, scope: String },
}

impl Display for FangErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FangErr::TypeMismatch { expected, found, scope } => {
                write!(f, "[Type mismatch]: Expected {}, found {} in scope {}", expected, found, scope)
            }
            FangErr::OperationUnsupported { op, lhs, rhs, scope } => {
                write!(f, "[Operation unsupported]: Tried to {} {} and {} in scope {}", lhs, op, rhs, scope)
            },
            FangErr::UndeclaredVariable { name, scope } => {
                write!(f, "[Undeclared variable]: Variable {} not found in scope {}", name, scope)
            },
            FangErr::AlreadyDeclaredVariable { name, scope } => {
                write!(f, "[Already declared]: Variable {} already declared in scope {}", name, scope)
            },
            FangErr::AlreadyDeclaredFunction { name, scope } => {
                write!(f, "[Already declared]: Function {} already declared in scope {}", name, scope)
            },
            FangErr::UnexpectedToken { expected, found, scope } => {
                write!(f, "[Unexpected token]: Expected {}, found {} in scope {}", expected, found, scope)
            },
            FangErr::ArgumentLengthMismatch { expected, found, scope } => {
                write!(f, "[Argument mismatch]: Expected {} arguments, found {} in scope {}", expected, found, scope)
            },
            FangErr::UnexpectedType { expected, found, scope } => {
                write!(f, "[Unexpected type]: Expected {}, found {} in scope {}", expected, found, scope)
            }
        }
    }
}