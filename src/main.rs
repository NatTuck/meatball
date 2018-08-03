
#[macro_use]
extern crate failure;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod parse;
mod program;
mod builtins;

use std::{env, fs};
use std::process::Command;

type Result<T> = ::std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
    match env::args().nth(1) {
        Some(file_name) => {
            let data = fs::read(file_name)?;
            let text = String::from_utf8(data)?;
            let ast = parse::parse(text)?;
            let prog = program::Program::new(&ast)?;
            println!("{:?}", prog);

            let asm_code = prog.code()?.to_string();
            println!("\n\n -- ASM --\n");
            println!("{}", &asm_code);

            fs::write("/tmp/mbasm.s", &asm_code)?;

            let mut asm_cmd = Command::new("gcc")
                .arg("-no-pie")
                .arg("-o")
                .arg("/tmp/mbasm")
                .arg("/tmp/mbasm.s")
                .spawn()?;
            asm_cmd.wait()?;

            let mut run_cmd = Command::new("/tmp/mbasm")
                .spawn()?;
            run_cmd.wait()?;
        },
        None => {
            println!("please provide source file to compile");
        }
    }

    Ok(())
}

