
use std::rc::Rc;
use std::fmt;

use regex::Regex;

use ::Result;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Ast {
    I64(i64, Src),
    F64(f64, Src),
    Str(String, Src),
    Sym(String, Src),
    App(Vec<Ast>),
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct Tok {
    pub val: Option<Ast>,
    pub txt: String,
    pub src: Src,
}

#[derive(Clone, PartialOrd, PartialEq)]
pub struct Src {
    pub file: Rc<String>,
    pub offs: usize,
}

impl Default for Src {
    fn default() -> Self {
        Src{
            file: Rc::new(String::from("<default>")),
            offs: 0,
        }
    }
}

impl fmt::Debug for Src {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Src>")
    }
}

pub fn zero_expr() -> Ast {
    Ast::I64(0, Src::default())
}

pub fn parse(text: String) -> Result<Vec<Ast>> {
    let toks = tokenize(text)?;
    let (tree, nn) = parse_toks(&toks)?;
    if nn != toks.len() {
        bail!("Leftover tokens: ate {} of {} ", nn, toks.len());
    }
    Ok(tree)
}

fn parse_toks(toks: &[Tok]) -> Result<(Vec<Ast>, usize)> {
    let mut ii = 0;
    let mut ys = vec![];

    while let Some(xx) = toks.get(ii) {
        ii = ii + 1;
        let rest = &toks[ii..];

        if let Some(ref node) = xx.val {
            ys.push(node.clone());
        }
        else {
            if xx.txt == "(" || xx.txt == "[" {
                let (item, nn) = parse_toks(rest)?;
                ys.push(Ast::App(item));
                ii += nn;
            }
            else if xx.txt == ")" || xx.txt == "]" {
                break;
            }
            else {
                bail!("unknown token: {:?}", xx);
            }
        }
    }

    Ok((ys, ii))
}

fn tokenize(text: String) -> Result<Vec<Tok>> {
    lazy_static! {
        static ref WSP: Regex = Regex::new(r"^\s+").unwrap();
    }

    let text = Rc::new(text);

    let mut ii = 0;
    let mut ys = vec![];

    while ii < text.len() {
        let (_, rest) = text.split_at(ii);

        if let Some(sp) = WSP.find(rest) {
            ii += sp.end();
            continue;
        }

        let (tok, len) = next_token(text.clone(), ii)?;
        ys.push(tok);
        ii += len;
    }

    Ok(ys)
}

fn next_token(text: Rc<String>, ii: usize) -> Result<(Tok, usize)> {
    lazy_static! {
        static ref PAR: Regex = Regex::new(r"^[\(\[\)\]]").unwrap();
        static ref F64: Regex = Regex::new(r"^\d+\.\d*").unwrap();
        static ref I64: Regex = Regex::new(r"^\d+").unwrap();
        static ref SYM: Regex =
            Regex::new(r#"^[\p{L}_+\-!?][\p{L}\p{N}_+\-!?]*"#).unwrap();
    }

    let src = Src{ file: text.clone(), offs: ii };
    let (_, rest) = text.split_at(ii);

    if let Some(mm) = PAR.find(rest) {
        let tok = Tok{
            val: None,
            txt: mm.as_str().to_string(),
            src: src,
        };

        return Ok((tok, mm.end()));
    }

    if let Some(mm) = F64.find(rest) {
        let txt = mm.as_str().to_string();
        let val = Ast::F64(txt.parse()?, src.clone());

        let tok = Tok{
            val: Some(val),
            txt: txt,
            src: src,
        };

        return Ok((tok, mm.end()));
    }

    if let Some(mm) = I64.find(rest) {
        let txt = mm.as_str().to_string();
        let val = Ast::I64(txt.parse()?, src.clone());

        let tok = Tok{
            val: Some(val),
            txt: txt,
            src: src,
        };

        return Ok((tok, mm.end()));
    }

    if let Some(mm) = SYM.find(rest) {
        let txt = mm.as_str().to_string();
        let val = Ast::Sym(txt.clone(), src.clone());

        let tok = Tok{
            val: Some(val),
            txt: txt,
            src: src,
        };

        return Ok((tok, mm.end()));
    }

    if let Some((txt, len)) = read_string(rest) {
        let val = Ast::Str(txt.clone(), src.clone());

        let tok = Tok{
            val: Some(val),
            txt: txt,
            src: src,
        };

        return Ok((tok, len));
    }

    let prefix: String = rest.chars().take(20).collect();
    bail!("Invalid token near: [{}]", prefix);
}

fn read_string(text: &str) -> Option<(String, usize)> {
    let mut cs = text.chars();
    let mut nn = 2;
    let mut yy = vec![];

    if cs.next() != Some('"') {
        return None;
    }

    while let Some(cc) = cs.next() {
        if cc == '"' {
            break;
        }

        if cc == '\\' {
            if let Some(dd) = cs.next() {
                if let Some(ss) = unescape_char(dd) {
                    yy.push(ss);
                }
                else {
                    return None;
                }
            }

            nn += 2;
        }
        else {
            yy.push(cc);

            nn += 1;
        }
    }

    Some((yy.iter().collect(), nn))
}

fn unescape_char(cc: char) -> Option<char> {
    match cc {
        '"' => Some('"'),
        'n' => Some('\n'),
        't' => Some('\t'),
        '\\' => Some('\\'),
        _ => None,
    }
}

