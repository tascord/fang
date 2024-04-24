use std::env::args;

use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("fang.l");
lrpar_mod!("fang.y");

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 1 {
        eprintln!("Usage: fang <source file>");
        std::process::exit(1);
    }

    let input = &args[1];
    let def = fang_l::lexerdef();
    let lexer = def.lexer(&input);
    let (res, err) = fang_y::parse(&lexer);
    for e in err {
        eprintln!("{}", e.pp(&lexer, &fang_y::token_epp));
    }

    match res {
        Some(Ok(r)) => {
            println!("{r:?}");
        }
        Some(Err(e)) => {
            eprintln!("{e:?}");
        }
        None => {
            eprintln!("Parse failed");
        }
    }
}