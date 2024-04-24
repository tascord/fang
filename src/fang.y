%start Expr
%%

Expr -> Result<f64, ()>:
    Expr 'ADD' Factor { Ok($1? + $3?) }
    | Term { $1 }
    ;

Term -> Result<f64, ()>:
    Term 'MULT' Factor { Ok($1? * $3?) }
    | Factor { $1 }
    ;

Term -> Result<f64, ()>:
    Term 'DIV' Factor { Ok($1? / $3?) }
    | Factor { $1 }
    ;

Term -> Result<f64, ()>:
    Term 'SUB' Factor { Ok($1? - $3?) }
    | Factor { $1 }
    ;

Factor -> Result<f64, ()>:
    'LPAREN' Expr 'RPAREN' { $2 }
    | 'INTEGER'
    | 'FLOAT'
    {
        let v = $1.map_err(|_| ())?;
        parse_num($lexer.span_str(v.span()))   
    }
    ;
%%

fn parse_num(s: &str) -> Result<f64, ()> {
    s.parse().map_err(|_| ())
}