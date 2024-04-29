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

Expression -> FRes<Node>:
    Addition { $1 }
    | FunctionCall { $1 }
    | 'DECLARATION' TypedVariable 'ASSIGNMENT' Expression {
        match $2.map_err(|_| ())? {
            Node::TypedVariable { var_type, name } => {
                Ok(Node::Declaration { name, var_type: Some(var_type), rhs: Some(Box::new($4?)) })
            },
            _ => Err(())
        }
    }
    | 'DECLARATION' PrimaryExpression 'ASSIGNMENT' Expression {
        match $2.map_err(|_| ())? {
            Node::Identifier { val } => {
                Ok(Node::Declaration { name: val, var_type: None, rhs: Some(Box::new($4?)) })
            },
            _ => Err(())
        }
    }
    | PrimaryExpression 'ASSIGNMENT' Expression {
        match $1.map_err(|_| ())? {
            Node::Identifier { val } => {
                Ok(Node::Assignment { name: val, rhs: Box::new($3?) })
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
    | Addition 'ADD' Subtraction { Ok(Node::Add { lhs: Box::new($1?), rhs: Box::new($3?) }) }
    ;

Subtraction -> FRes<Node>:
    Multiplication { $1 }
    | Subtraction 'SUB' Multiplication { Ok(Node::Subtract { lhs: Box::new($1?), rhs: Box::new($3?) }) }
    ;

Multiplication -> FRes<Node>:
    Division { $1 }
    | Multiplication 'MUL' Division { Ok(Node::Multiply { lhs: Box::new($1?), rhs: Box::new($3?) }) }
    ;

Division -> FRes<Node>:
    PrimaryExpression { $1 }
    | Division 'DIV' PrimaryExpression { Ok(Node::Divide { lhs: Box::new($1?), rhs: Box::new($3?) }) }
    ;

TypedVariable -> FRes<Node>:
    'IDENTIFIER' 'COLON' 'IDENTIFIER' { Ok(type_var($lexer.span_str(($1.map_err(|_| ())?).span()), $lexer.span_str(($3.map_err(|_| ())?).span())))? }
    ;

TypedVariableList -> FRes<Vec<Node>>:
    TypedVariableList ',' TypedVariable { append($1.map_err(|_| ())?, $3.map_err(|_| ())?) }
    | TypedVariable { Ok(vec![$1.map_err(|_| ())?]) }
    ;

PrimaryExpression -> FRes<Node>:
    'IDENTIFIER' { Ok(Node::Identifier { val: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string() }) }
    | 'LPAREN' Expression 'RPAREN' { $2 }
    | 'INTEGER' { parse_int($lexer.span_str(($1.map_err(|_| ())?).span())) }
    | 'FLOAT' { parse_float($lexer.span_str(($1.map_err(|_| ())?).span())) }
    | 'BOOLEAN' { parse_bool($lexer.span_str(($1.map_err(|_| ())?).span())) }
    | 'STRING' { parse_string($lexer.span_str(($1.map_err(|_| ())?).span())) }
    | Object { $1 }
    ;

Function -> FRes<Node>:
    'FUNCTION' 'IDENTIFIER' 'LPAREN' TypedVariableList 'RPAREN' 'LBRACE' StatementList 'RBRACE' {
        Ok(Node::Function { name: $lexer.span_str(($2.map_err(|_| ())?).span()).to_string(), args: Box::new($4.map_err(|_| ())?), body: Box::new($7.map_err(|_| ())?)})
    }
    ;

FunctionCall -> FRes<Node>:
    'IDENTIFIER' 'LPAREN' ExpressionList 'RPAREN' { Ok(Node::Call { name: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), args: Box::new($3.map_err(|_| ())?) }) }
    ;

Object -> FRes<Node>:
    'LBRACE' 'RBRACE' { Ok(Node::Object { fields: Box::new(vec![]) }) }
    | 'LBRACE' ObjectFields 'RBRACE' { Ok(Node::Object { fields: Box::new($2.map_err(|_| ())?) }) }
    ;

ObjectFields -> FRes<Vec<Node>>:
    ObjectFields ',' ObjectField { append($1.map_err(|_| ())?, $3.map_err(|_| ())?)}
    | ObjectField { Ok(vec![$1.map_err(|_| ())?]) }
    ;

ObjectField -> FRes<Node>:
    'IDENTIFIER' 'COLON' Expression { Ok(Node::Field { name: $lexer.span_str(($1.map_err(|_| ())?).span()).to_string(), value: Box::new($3.map_err(|_| ())?) }) }
    ;

%%

use crate::ast::*;
type FRes<T> = Result<T, ()>;

fn parse_int(s: &str) -> FRes<Node> {
   match s.parse::<u64>() {
    Ok(v) => Ok(Node::Integer { val: v } ),
    Err(_) => {
        eprintln!("{} cannot be represented as an integer.", s);
        Err(())
    }
   }
}

fn parse_float(s: &str) -> FRes<Node> {
   match s.parse::<f64>() {
    Ok(v) => Ok(Node::Float { val: v } ),
    Err(_) => {
        eprintln!("{} cannot be represented as a float.", s);
        Err(())
    }
   }
}

fn parse_bool(s: &str) -> FRes<Node> {
   match s.parse::<bool>() {
    Ok(v) => Ok(Node::Boolean { val: v } ),
    Err(_) => {
        eprintln!("{} cannot be represented as a bool.", s);
        Err(())
    }
   }
}

fn parse_string(s: &str) -> FRes<Node> {
    Ok(Node::String {val: s.to_string().split_at(1).1.split_at(s.len() - 2).0.to_string() })
}

fn type_var(name: &str, var_type: &str) -> FRes<Node> {
   Ok(Node::TypedVariable { var_type: var_type.to_string(), name: name.to_string() })
}

fn append(mut lhs: Vec<Node>, rhs: Node ) -> Result<Vec<Node>, ()>{
    lhs.push(rhs);
    Ok(lhs)
}
