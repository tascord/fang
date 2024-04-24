use lrlex::CTLexerBuilder;

fn main() {
    CTLexerBuilder::new()
        .rust_edition(lrlex::RustEdition::Rust2018)
        .lrpar_config(|c| {
            c.yacckind(cfgrammar::yacc::YaccKind::Grmtools)
                .rust_edition(lrpar::RustEdition::Rust2018)
                .grammar_in_src_dir("fang.y")
                .unwrap()
        })
        .lexer_in_src_dir("fang.l")
        .unwrap()
        .build()
        .unwrap();
}
