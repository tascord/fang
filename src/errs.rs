use std::fmt::Display;

use crate::ast::Spans;

#[derive(Debug)]
pub enum FangErr {
    TypeMismatch {
        span: Spans,
        expected: String,
        found: String,
        scope: String,
    },
    OperationUnsupported {
        span: Spans,
        op: String,
        lhs: String,
        rhs: String,
        scope: String,
    },
    UndeclaredVariable {
        span: Spans,
        name: String,
        scope: String,
    },
    UndeclaredType {
        span: Spans,
        name: String,
        scope: String,
    },
    UndeclaredFunction {
        span: Spans,
        name: String,
        scope: String,
    },
    AlreadyDeclaredVariable {
        span: Spans,
        name: String,
        scope: String,
    },
    AlreadyDeclaredFunction {
        span: Spans,
        name: String,
        scope: String,
    },
    AlreadyDeclaredTrait {
        span: Spans,
        name: String,
        scope: String,
    },
    AlreadyDeclaredStruct {
        span: Spans,
        name: String,
        scope: String,
    },
    AlreadyImplementedTrait {
        span: Spans,
        name: String,
        scope: String,
    },
    UnexpectedToken {
        span: Spans,
        expected: String,
        found: String,
        scope: String,
    },
    ArgumentLengthMismatch {
        span: Spans,
        expected: usize,
        found: usize,
        scope: String,
    },
    UnexpectedType {
        span: Spans,
        expected: String,
        found: String,
        scope: String,
    },
}

impl Display for FangErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FangErr::TypeMismatch {
                expected,
                found,
                scope,
                span,
            } => {
                write!(
                    f,
                    "[Type mismatch]: Expected {}, found {} in scope {}\n{}",
                    expected,
                    found,
                    scope,
                    span.snippet()
                )
            }
            FangErr::OperationUnsupported {
                op,
                lhs,
                rhs,
                scope,
                span,
            } => {
                write!(
                    f,
                    "[Operation unsupported]: Tried to {} {} and {} in scope {}\n{}",
                    lhs,
                    op,
                    rhs,
                    scope,
                    span.snippet()
                )
            }
            FangErr::UndeclaredVariable { name, scope, span } => {
                write!(
                    f,
                    "[Undeclared variable]: Variable {} not found in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::UndeclaredFunction { name, scope, span } => {
                write!(
                    f,
                    "[Undeclared function]: Function {} not found in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::UndeclaredType { name, scope, span } => {
                write!(
                    f,
                    "[Undeclared type]: Type {} not found in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::AlreadyDeclaredVariable { name, scope, span } => {
                write!(
                    f,
                    "[Already declared]: Variable {} already declared in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::AlreadyDeclaredFunction { name, scope, span } => {
                write!(
                    f,
                    "[Already declared]: Function {} already declared in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::AlreadyDeclaredTrait { name, scope, span } => {
                write!(
                    f,
                    "[Already declared]: Trait {} already declared in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::AlreadyDeclaredStruct { name, scope, span } => {
                write!(
                    f,
                    "[Already declared]: Struct {} already declared in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::AlreadyImplementedTrait { name, scope, span } => {
                write!(
                    f,
                    "[Already implemented]: Trait {} already implemented in scope {}\n{}",
                    name,
                    scope,
                    span.snippet()
                )
            }
            FangErr::UnexpectedToken {
                expected,
                found,
                scope,
                span,
            } => {
                write!(
                    f,
                    "[Unexpected token]: Expected {}, found {} in scope {}\n{}",
                    expected,
                    found,
                    scope,
                    span.snippet()
                )
            }
            FangErr::ArgumentLengthMismatch {
                expected,
                found,
                scope,
                span,
            } => {
                write!(
                    f,
                    "[Argument mismatch]: Expected {} arguments, found {} in scope {}\n{}",
                    expected,
                    found,
                    scope,
                    span.snippet()
                )
            }
            FangErr::UnexpectedType {
                expected,
                found,
                scope,
                span,
            } => {
                write!(
                    f,
                    "[Unexpected type]: Expected {}, found {} in scope {}\n{}",
                    expected,
                    found,
                    scope,
                    span.snippet()
                )
            }
        }
    }
}
