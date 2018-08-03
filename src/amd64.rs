
pub struct Inst {
    pub name: String,
    pub args: Vec<String>,
}

impl Inst {
    pub fn new(name: String, args: Vec<String>) -> Inst {
        Inst{ name, args }
    }

    pub fn to_string(&self) -> String {
        format!("  {} {}", self.name, self.args.join(", "))
    }
}

pub struct BB {
    pub label: String,
    pub insts: Vec<Inst>,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_to_text() {

    }
}



