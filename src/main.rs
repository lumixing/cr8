use std::{env, fs};

use crate::{interp::interp, lexer::Lexer, parser::parse};

mod interp;
mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = &args[1];
    let output_path = &args[2];

    let source = fs::read_to_string(input_path).unwrap();
    // println!("{:?}", source);

    let lexer = Lexer::new(source.as_str());
    // for (token, _) in lexer {
    // println!("{:?}", token);
    // }

    let program = parse(lexer).unwrap();
    // for st in &program.statements {
    // println!("{:?}", st.node);
    // }

    let ins = interp(&program);
    // println!("{:#04X?}", ins);

    fs::write(output_path, ins.clone()).unwrap();
    println!(
        "successfully compiled {} instructions to {}",
        ins.len(),
        output_path
    );
}
