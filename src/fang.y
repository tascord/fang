%start StatementList
%%

StatementList -> FRes<Vec<Node>>:
    StatementList Statement { append($1.map_err(|_| ())?, $2.map_err(|_| ())?) }
    | { Ok(vec![]) }
    ;

Statement -> FRes<Node>:
    ';' { Ok(Node::Empty) }
    | Expression ';' { $1 }
    | Function { $1 }
    ;

StatementOrReturnList -> FRes<Vec<Node>>:
    StatementOrReturnList StatementOrReturn { append($1.map_err(|_| ())?, $2.map_err(|_| ())?) }
    | { Ok(vec![]) }
    ;

StatementOrReturn -> FRes<Node>:
    Statement { $1 }
    | 'RETURN' Expression ';' { Ok(Node::Return { value: Box::new($2?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Expression -> FRes<Node>:
    Addition { $1 }
    | 'DECLARATION' TypedVariable 'ASSIGNMENT' Expression {
        match $2.map_err(|_| ())? {
            Node::TypedVariable { var_type, name, .. } => {
                Ok(Node::Declaration { name, var_type: Some(var_type), rhs: Some(Box::new($4?)), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
            },
            _ => Err(())
        }
    }
    | 'DECLARATION' PrimaryExpression 'ASSIGNMENT' Expression {
        match $2.map_err(|_| ())? {
            Node::Identifier { val, .. } => {
                Ok(Node::Declaration { name: val, var_type: None, rhs: Some(Box::new($4?)), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
            },
            _ => Err(())
        }
    }
    | PrimaryExpression 'ASSIGNMENT' Expression {
        match $1.map_err(|_| ())? {
            Node::Identifier { val, .. } => {
                Ok(Node::Assignment { name: val, rhs: Box::new($3?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
            },
            _ => Err(())
        }
    }
    ;

ExpressionList -> FRes<Vec<Node>>:
    ExpressionList ',' Expression { append($1.map_err(|_| ())?, $3.map_err(|_| ())?) }
    | Expression { Ok(vec![$1.map_err(|_| ())?]) }
    ;

Addition -> FRes<Node>:
    Subtraction { $1 }
    | Addition 'ADD' Subtraction { Ok(Node::Add { lhs: Box::new($1?), rhs: Box::new($3?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Subtraction -> FRes<Node>:
    Multiplication { $1 }
    | Subtraction 'SUB' Multiplication { Ok(Node::Subtract { lhs: Box::new($1?), rhs: Box::new($3?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Multiplication -> FRes<Node>:
    Division { $1 }
    | Multiplication 'MUL' Division { Ok(Node::Multiply { lhs: Box::new($1?), rhs: Box::new($3?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Division -> FRes<Node>:
    PrimaryExpression { $1 }
    | Division 'DIV' PrimaryExpression { Ok(Node::Divide { lhs: Box::new($1?), rhs: Box::new($3?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

TypedVariable -> FRes<Node>:
    'IDENTIFIER' 'COLON' 'IDENTIFIER' { Ok(type_var($lexer.span_str(($1.map_err(|_| ())?).span()), $lexer.span_str(($3.map_err(|_| ())?).span()), Spans::new($lexer.span_lines_str($span), $lexer.line_col($span))))? }
    | 'IDENTIFIER' 'COLON' 'SELF' { Ok(type_var($lexer.span_str(($1.map_err(|_| ())?).span()), "self", Spans::new($lexer.span_lines_str($span), $lexer.line_col($span))))? }
    | 'SELF' { Ok(Node::TypedVariable { var_type: "self".to_string(), name: "self".to_string(), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

TypedVariableList -> FRes<Vec<Node>>:
    TypedVariableList ',' TypedVariable { append($1.map_err(|_| ())?, $3.map_err(|_| ())?) }
    | TypedVariable { Ok(vec![$1.map_err(|_| ())?]) }
    ;

PrimaryExpression -> FRes<Node>:
    'IDENTIFIER' { Ok(Node::Identifier { val: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    | 'LPAREN' Expression 'RPAREN' { $2 }
    | 'INTEGER' { parse_int($lexer.span_str(($1.map_err(|_| ())?).span()), Spans::new($lexer.span_lines_str($span), $lexer.line_col($span))) }
    | 'FLOAT' { parse_float($lexer.span_str(($1.map_err(|_| ())?).span()), Spans::new($lexer.span_lines_str($span), $lexer.line_col($span))) }
    | 'BOOLEAN' { parse_bool($lexer.span_str(($1.map_err(|_| ())?).span()), Spans::new($lexer.span_lines_str($span), $lexer.line_col($span))) }
    | 'STRING' { parse_string($lexer.span_str(($1.map_err(|_| ())?).span()), Spans::new($lexer.span_lines_str($span), $lexer.line_col($span))) }
    | Object { $1 }
    | Struct { $1 }
    | Trait { $1 }
    | TraitImpl { $1 }
    | FunctionCall { $1 }
    ;

Function -> FRes<Node>:
    'FUNCTION' 'IDENTIFIER' 'LPAREN' TypedVariableList 'RPAREN' 'COLON' 'IDENTIFIER' 'LBRACE' StatementOrReturnList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new($4.map_err(|_| ())?), body: Box::new($9.map_err(|_| ())?), return_type: Some($lexer.span_str(($7.map_err(|_| ())?).span()).to_string()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' TypedVariableList 'RPAREN' 'COLON' 'SELF' 'LBRACE' StatementOrReturnList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new($4.map_err(|_| ())?), body: Box::new($9.map_err(|_| ())?), return_type: Some("self".to_string()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' TypedVariableList 'RPAREN' 'LBRACE' StatementOrReturnList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new($4.map_err(|_| ())?), body: Box::new($7.map_err(|_| ())?), return_type: None, span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' 'RPAREN' 'COLON' 'SELF' 'LBRACE' StatementOrReturnList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new(Vec::new()), body: Box::new($8.map_err(|_| ())?), return_type: Some("self".to_string()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' 'RPAREN' 'COLON' 'IDENTIFIER' 'LBRACE' StatementOrReturnList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new(Vec::new()), body: Box::new($8.map_err(|_| ())?), return_type: Some($lexer.span_str(($7.map_err(|_| ())?).span()).to_string()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' 'RPAREN' 'LBRACE' StatementOrReturnList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new(Vec::new()), body: Box::new($6.map_err(|_| ())?), return_type: None, span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    ;

FunctionCall -> FRes<Node>:
    'IDENTIFIER' 'LPAREN' ExpressionList 'RPAREN' { Ok(Node::Call { name: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), args: Box::new($3.map_err(|_| ())?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    | 'IDENTIFIER' 'LPAREN' 'RPAREN' { Ok(Node::Call { name: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), args: Box::new(Vec::new()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Struct -> FRes<Node>:
    'STRUCT' 'IDENTIFIER' 'LBRACE' TypedVariableList 'RBRACE' { Ok(Node::Struct { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), fields: Box::new($4.map_err(|_| ())?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Object -> FRes<Node>:
    'IDENTIFIER' 'LBRACE' 'RBRACE' { Ok(Node::Object { typed: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), fields: Box::new(vec![]), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    | 'IDENTIFIER' 'LBRACE' ObjectFields 'RBRACE' { Ok(Node::Object { typed:$lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), fields: Box::new($3.map_err(|_| ())?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

ObjectFields -> FRes<Vec<Node>>:
    ObjectFields ',' ObjectField { append($1.map_err(|_| ())?, $3.map_err(|_| ())?)}
    | ObjectField { Ok(vec![$1.map_err(|_| ())?]) }
    ;

ObjectField -> FRes<Node>:
    'IDENTIFIER' 'COLON' Expression { Ok(Node::Field { name: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), value: Box::new($3.map_err(|_| ())?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

Trait -> FRes<Node>:
    'TRAIT' 'IDENTIFIER' 'LBRACE' TraitFields 'RBRACE' { Ok(Node::Trait { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), fields: Box::new($4.map_err(|_| ())?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

TraitFields -> FRes<Vec<Node>>:
    TraitFields TraitField { append($1.map_err(|_| ())?, $2.map_err(|_| ())?)}
    | TraitField { Ok(vec![$1.map_err(|_| ())?]) }
    ;

TraitField -> FRes<Node>:
    Function { $1 }
    | FunctionOutline { $1 }
    ;

FunctionOutline -> FRes<Node>:
    'FUNCTION' 'IDENTIFIER' 'LPAREN' TypedVariableList 'RPAREN' 'COLON' 'IDENTIFIER' ';' {
        Ok(Node::FunctionOutline { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new($4.map_err(|_| ())?), return_type: Some($lexer.span_str(($7.map_err(|_| ())?).span()).to_string()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' TypedVariableList 'RPAREN' 'COLON' 'SELF' ';' {
        Ok(Node::FunctionOutline { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new($4.map_err(|_| ())?), return_type: Some("self".to_string()), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    | 'FUNCTION' 'IDENTIFIER' 'LPAREN' 'RPAREN' ';' {
        Ok(Node::FunctionOutline { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new(Vec::new()), return_type: None, span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) })
    }
    ;

TraitImpl -> FRes<Node>:
    'IMPL' 'IDENTIFIER' 'FOR' 'IDENTIFIER' 'LBRACE' TraitImplFields 'RBRACE' { Ok(Node::TraitImpl { trait_name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), type_name: $lexer.span_str(($4.map_err(|_| ())?).span()).to_string(), fields: Box::new($6.map_err(|_| ())?), span: Spans::new($lexer.span_lines_str($span), $lexer.line_col($span)) }) }
    ;

TraitImplFields -> FRes<Vec<Node>>:
    TraitImplFields TraitImplField { append($1.map_err(|_| ())?, $2.map_err(|_| ())?)}
    | TraitImplField { Ok(vec![$1.map_err(|_| ())?]) }
    ;

TraitImplField -> FRes<Node>:
    Function { $1 }
    ;

%%

use crate::ast::*;
type FRes<T> = Result<T, ()>;

fn parse_int(s: &str, sp: Spans) -> FRes<Node> {
   match s.parse::<u64>() {
    Ok(v) => Ok(Node::Integer { val: v, span: sp} ),
    Err(_) => {
        eprintln!("{} cannot be represented as an integer.", s);
        Err(())
    }
   }
}

fn parse_float(s: &str, sp: Spans) -> FRes<Node> {
   match s.parse::<f64>() {
    Ok(v) => Ok(Node::Float { val: v, span: sp} ),
    Err(_) => {
        eprintln!("{} cannot be represented as a float.", s);
        Err(())
    }
   }
}

fn parse_bool(s: &str, sp: Spans) -> FRes<Node> {
   match s.parse::<bool>() {
    Ok(v) => Ok(Node::Boolean { val: v, span: sp } ),
    Err(_) => {
        eprintln!("{} cannot be represented as a bool.", s);
        Err(())
    }
   }
}

fn parse_string(s: &str, sp: Spans) -> FRes<Node> {
    Ok(Node::String {val: s.to_string().split_at(1).1.split_at(s.len() - 2).0.to_string() , span: sp})
}

fn type_var(name: &str, var_type: &str, sp: Spans) -> FRes<Node> {
   Ok(Node::TypedVariable { var_type: var_type.to_string(), name: name.to_string(), span: sp })
}

fn append(mut lhs: Vec<Node>, rhs: Node ) -> Result<Vec<Node>, ()>{
    lhs.push(rhs);
    Ok(lhs)
}