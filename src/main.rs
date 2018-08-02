
#[macro_use]
extern crate failure;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod parse;
mod program;
mod builtins;

use std::{env, fs};

type Result<T> = ::std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
    match env::args().nth(1) {
        Some(file_name) => {
            let data = fs::read(file_name)?;
            let text = String::from_utf8(data)?;
            let mut ast = parse::parse(text)?;
            ast.push(parse::zero_expr());
            let prog = program::Program::new(&ast)?;
            println!("{:?}", prog);
            println!("\n\n -- ASM --\n");
            println!("{}", prog.code());
        },
        None => {
            println!("please provide source file to compile");
        }
    }

    Ok(())
}

