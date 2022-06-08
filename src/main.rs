// So, you want to write a parser (without pulling in a crate)?

type Source<'a> = &'a [u8];
type Compiled = isize;
#[allow(dead_code)]
#[derive(Debug)]
pub enum Insn {
    Const(isize),
    Dup,
    Add,
    Sub,
    Mul,
    Negate,
    Print,
    Block(Vec<Insn>),
    Restart(usize),
    Exit(usize),
    SkipNextIfFalse,
}

#[test]
fn number1() {
    assert_eq!(run(b"-42"), Ok(-42));
}
#[test]
fn number2() {
    assert_eq!(run(b" - 42"), Ok(-42));
}
#[test]
fn number3() {
    assert_eq!(run(b" 84 - 42"), Ok(42));
}
#[test]
fn number4() {
    assert_eq!(run(b" 84 + -42"), Ok(42));
}
#[test]
fn number5() {
    assert_eq!(run(b"(2)"), Ok(2));
}
#[test]
fn number6() {
    assert!(run(b"(2 + )").is_err());
}
#[test]
fn number7() {
    assert!(run(b" ( 2 ").is_err());
}

fn main() {
    let mut range = 1..5;

    'from_beginning: loop {
        for x in range {
            println!("{x}");
            if x == 3 {
                println!("changing gear");
                range = 10..15;
                continue 'from_beginning;
            }
        }
        break;
    }

    if false {
        for s in std::env::args().skip(1) {
            match run(s.as_bytes()) {
                Ok(v) => println!("Result: {v}"),
                Err(s) => println!("Error: {s}"),
            }
        }
    }
}

fn run(s: Source) -> Result<Compiled, String> {
    let mut _code: Vec<Insn> = vec![];
    parse(s)
    //execute(code)
}

#[allow(dead_code)]
pub struct VM {
    stack: Vec<isize>,
    tos: isize,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: vec![],
            tos: 0,
        }
    }

    fn pop(&mut self) -> Result<isize, String> {
        let res = self.tos;
        self.tos = self.stack.pop().ok_or("Run: stack empty")?;
        Ok(res)
    }

    pub fn execute(&mut self, mut code: &[Insn]) -> Result<(), String> {
        let mut suspended = vec![];
        let mut pc = 0;

        loop {
            let insn = if pc < code.len() { &code[pc] } else { &Exit(0) };
            pc += 1;
            use Insn::*;
            println!("Execute {insn:?} (tos {})", self.tos);
            match insn {
                Dup => self.stack.push(self.tos),
                Add => self.tos += self.stack.pop().ok_or("Run: stack empty")?,
                Sub => self.tos -= self.stack.pop().ok_or("Run: stack empty")?,
                Mul => self.tos *= self.stack.pop().ok_or("Run: stack empty")?,
                Negate => self.tos = -self.tos,
                Const(k) => {
                    self.stack.push(self.tos);
                    self.tos = *k;
                }
                Print => {
                    println!("{}", self.pop()?);
                }
                Block(inner) => {
                    suspended.push((code, pc));
                    println!("  Suspended: {suspended:?}");
                    code = inner;
                    pc = 0;
                }
                SkipNextIfFalse => {
                    if self.pop()? == 0 {
                        pc += 1;
                    }
                }
                Restart(n) => {
                    suspended.drain(suspended.len() - n..);
                    println!("  Suspended: {suspended:?}");
                    (code, _) = *suspended.last().ok_or("Illegal Restart({n})")?;
                    pc = 0;
                }
                Exit(n) => {
                    suspended.drain(suspended.len() - n..);
                    if suspended.is_empty() {
                        return Ok(());
                    }
                    println!("  Suspended: {suspended:?}");
                    (code, pc) = suspended.pop().ok_or("Illegal Exit({n})")?;
                }
            }
        }
    }
}

#[test]
fn execute_test() {
    use Insn::*;
    let mut vm = VM::new();
    assert_eq!(
        vm.execute(&vec![Block(vec![
            Const(43),
            Const(42),
            Add,
            Print,
            Const(84),
            Const(1),
            SkipNextIfFalse,
            Exit(0),
            Const(666),
            Print,
            Const(666),
        ])]),
        Ok(())
    );
    assert_eq!(vm.tos, 84);
}

#[test]
fn execute_nested() {
    use Insn::*;
    let mut vm = VM::new();
    let blk2 = Block(vec![Const(42), Const(1), SkipNextIfFalse, Exit(1)]);
    let blk1 = Block(vec![blk2, Const(666)]);

    assert_eq!(vm.execute(&vec![blk1]), Ok(()));
    assert_eq!(vm.tos, 42);
}

fn parse(mut s: Source) -> Result<Compiled, String> {
    skip_whitespace(&mut s);
    let r = parse_expr(&mut s);
    if r.is_ok() {
        if s.is_empty() {
            r
        } else {
            Err(format!("Junk at end: {}", std::str::from_utf8(s).unwrap()))
        }
    } else {
        r
    }
}

fn parse_expr(s: &mut Source) -> Result<Compiled, String> {
    let mut v = parse_term(s)?;
    loop {
        if token_match(s, b"+") {
            v += parse_term(s)?;
        } else if token_match(s, b"-") {
            v -= parse_term(s)?;
        } else {
            break;
        }
    }
    Ok(v)
}

fn parse_term(s: &mut Source) -> Result<Compiled, String> {
    let mut v = parse_factor(s)?;
    while token_match(s, b"*") {
        v *= parse_factor(s)?;
    }
    Ok(v)
}

fn parse_factor(s: &mut Source) -> Result<Compiled, String> {
    let begin = &<&[u8]>::clone(s);
    if token_match(s, b"(") {
        let r = parse_expr(s)?;
        if token_match(s, b")") {
            Ok(r)
        } else {
            Err(format!(
                "{} lack closing parens",
                std::str::from_utf8(begin).unwrap()
            ))
        }
    } else if token_match(s, b"-") {
        Ok(-parse_factor(s)?)
    } else {
        parse_uint(s)
    }
}

fn parse_uint(s: &mut Source) -> Result<Compiled, String> {
    if b'0' <= s[0] && s[0] <= b'9' {
        let mut v = 0;
        while !s.is_empty() && b'0' <= s[0] && s[0] <= b'9' {
            v = v * 10 + (s[0] as isize - b'0' as isize) as isize;
            *s = &s[1..];
        }
        skip_whitespace(s);
        Ok(v)
    } else {
        Err(format!(
            "Expected uint, not {}",
            std::str::from_utf8(s).unwrap()
        ))
    }
}

fn token_match<'a>(s: &mut Source<'a>, expected: &[u8]) -> bool {
    if s.len() >= expected.len() && s[0..expected.len()] == *expected {
        *s = &s[expected.len()..];
        skip_whitespace(s);
        true
    } else {
        false
    }
}

fn skip_whitespace(s: &mut Source) {
    while !s.is_empty() && s[0] == b' ' {
        *s = &s[1..];
    }
}
