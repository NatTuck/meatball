
use std::collections::BTreeMap;

use parse::{Ast, Src};
use builtins::Builtin;
use ::Result;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct Program {
    pub entry: String,
    pub mods: BTreeMap<String, Func>,
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct Func {
    // Vars have access to bindings:
    //  - In parent scopes.
    //  - That came earlier in the same scope.
    //  - Of functions in the same scope.
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Expr>,
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Expr {
    I64(i64),
    Var(String),
    Call(String, Vec<Expr>),
    Cond(Vec<(Expr, Expr)>),
    Func(Box<Func>),
}

impl Program {
    pub fn new(ast: &[Ast]) -> Result<Self> {
        let mut wrap = vec![Ast::App(vec![])];
        wrap.extend_from_slice(ast);

        let main = Func::new(&wrap[..])?;
        let mut mods = BTreeMap::new();
        mods.insert("main".to_string(), main);
        let prgm = Program{
            entry: "main".to_string(),
            mods: mods,
        };
        Ok(prgm)
    }

    pub fn code(&self) -> Result<String> {
        let mut code_text = String::from("");

        for (name, func) in &self.mods {
            let text = func.code(&name)?;
            code_text.push_str(&text);
        }

        let code = format!(r####"
  .global main

  .text
main:

{}

  mov %rax, %rsi
  mov $_mb_main_result, %rdi
  mov $0, %al
  call printf
  ret


  .data
_mb_main_result: .string "result = %ld\n"

"####, code_text);
        Ok(code)
    }
}

fn ast2var(ast: &[Ast]) -> Result<(String, Expr)> {
    if ast.len() < 3 {
        bail!("Expected binding, got: {:?}", &ast);
    }

    match &ast[0] {
        Ast::Sym(ref op, _) if op == "let" => {
            if ast.len() < 3 {
                bail!("Wrong length let");
            }

            if let Ast::Sym(ref name, _) = ast[1] {
                let expr = Expr::new(&ast[2])?;
                Ok((name.to_string(), expr))
            }
            else {
                bail!("Can't bind {:?} in let", &ast[1]);
            }
        },
        Ast::Sym(ref op, _) if op == "def" => {
            let ff = Func::new(&ast[1..])?;
            let nn = ff.name.clone().unwrap_or("??".into());
            Ok((nn, Expr::Func(Box::new(ff))))
        },
        other => {
            bail!("Expected let, got: {:?}", other);
        }
    }
}

impl Expr {
    pub fn new(ast: &Ast) -> Result<Self> {
        match ast {
            Ast::App(items) => {
                if items.is_empty() {
                    bail!("empty App");
                }

                let (head, tail) = items.split_first().unwrap();

                if let Ast::Sym(name, _) = head {
                    if name == "def" {
                        bail!("def isn't an expr");
                    }
                    else {
                        let mut args = vec![];
                        for xx in tail {
                            let expr = Expr::new(xx)?;
                            args.push(expr);
                        }
                        Ok(Expr::Call(name.to_string(), args))
                    }
                }
                else {
                    bail!("Trying to apply non-symbol: {:?}", head);
                }
            },
            Ast::I64(nn, _) => {
                Ok(Expr::I64(*nn))
            },
            Ast::Sym(ss, _) => {
                Ok(Expr::Var(ss.to_string()))
            },
            other => {
                bail!("TODO: {:?}", other);
            }
        }
    }

    pub fn code(&self, prefix: &str) -> Result<String> {
        match self {
            Expr::I64(nn) => {
                Ok(format!("  mov ${}, %rax\n", nn))
            },
            Expr::Var(name) => {
                let vpos = format!("{}??(%rsp)", name);
                Ok(format!("  mov {}, %rax\n", vpos))
            },
            Expr::Call(name, args) => {
                // Allocate frame for {name}
                // for each arg:
                //   calc result
                //   mov result to slot in frame
                Ok(format!("  [[funcall: {} {:?}]]", name, args))
            },
            Expr::Cond(_parts) => {
                bail!("giving up");
            }
            Expr::Func(func) => {
                func.code(prefix)
            }
        }
    }
}

impl Func {
    pub fn new(ast: &[Ast]) -> Result<Self> {
        if let Some((Ast::App(pas), bas)) = ast.split_first() {
            let mut pas = pas.clone();
            if pas.is_empty() {
                pas.push(Ast::Sym(String::from("derp"), Src::default()));
            }

            let mut params = vec![];
            for pp in pas {
                if let Ast::Sym(ss, _) = pp {
                    params.push(ss.to_string());
                }
                else {
                    bail!("non-symbol in param list");
                }
            }

            let name = Some(params[0].clone());

            let mut body = vec![];
            for bb in bas {
                body.push(Expr::new(bb)?);
            }

            let params = params[1..].iter().cloned().collect();
            let ff = Func{ name, body, params };
            Ok(ff)
        }
        else {
            bail!("empty Func: {:?}", &ast);
        }
    }

    pub fn code(&self, prefix: &str) -> Result<String> {
        let mut code = String::from("");

        for exp in &self.body {
            code.push_str(&exp.code(prefix)?);
        }

        Ok(code)
    }
}

