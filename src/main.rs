use std::{
    env::args,
    fs::{self, File},
    io::Read,
};

use bytecode::eval_bytecode;
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use scope::GLOBAL_SCOPE;

lrlex_mod!("fang.l");
lrpar_mod!("fang.y");

pub mod ast;
pub mod bytecode;
pub mod errs;
pub mod scope;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 1 {
        eprintln!("Usage: fang <source file>");
        std::process::exit(1);
    }

    let input = {
        let mut s = String::new();
        let mut f = File::open(&args[1]).unwrap();
        f.read_to_string(&mut s).unwrap();
        s
    };

    let def = fang_l::lexerdef();
    let lexer = def.lexer(&input);
    let (res, err) = fang_y::parse(&lexer);

    if err.len() > 0 {
        eprintln!("Unable to parse:");
    }

    for e in err {
        eprintln!("\t- {}", e.pp(&lexer, &fang_y::token_epp));
    }

    match res {
        Some(Ok(ast)) => {
            fs::write("./fg.ast", format!("{ast:#?}")).unwrap();
            eval_bytecode(ast, &mut GLOBAL_SCOPE.clone())
                .map_err(|e| eprintln!("{}", e.to_string()))
                .ok()
        }
        _ => {
            eprintln!("\nFang Failed :(");
            panic!()
        }
    };
}
