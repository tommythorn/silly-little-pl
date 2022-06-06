// So, you want to write a parser (without pulling in a crate)?

type Source<'a> = &'a[u8];

#[test]
fn number1() { assert_eq!(parse(b"-42"), Ok(-42)); }
#[test]
fn number2() { assert_eq!(parse(b" - 42"), Ok(-42)); }
#[test]
fn number3() { assert_eq!(parse(b" 84 - 42"), Ok(42)); }
#[test]
fn number4() { assert_eq!(parse(b" 84 + -42"), Ok(42)); }

fn main() {
    match parse(b" 42 + 56*78 - 40") {
	Ok(v) => println!("Result {v}"),
	Err(s) => println!("{s}"),
    }
}

fn parse(mut s: Source) -> Result<isize, String> {
    skip_whitespace(&mut s);
    let r = parse_expr(&mut s);
    if s.is_empty() {
	r
    } else {
	Err(format!("What the hell is {s:?}?"))
    }
}

fn parse_expr(s: &mut Source) -> Result<isize, String> {
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

fn parse_term(s: &mut Source) -> Result<isize, String> {
    let mut v = parse_factor(s)?;
    while token_match(s, b"*") {
	v *= parse_factor(s)?;
    }
    Ok(v)
}

fn parse_factor(s: &mut Source) -> Result<isize, String> {
    if token_match(s, b"-") {
	//return -(parse_factor(s)?);
	return Ok(-(parse_factor(s)?));
    } else {
	parse_uint(s)
    }
}

fn parse_uint(s: &mut Source) -> Result<isize, String> {
    if b'0' <= s[0] && s[0] <= b'9' {
	let mut v = 0;
	while !s.is_empty() && b'0' <= s[0] && s[0] <= b'9' {
	    v = v*10 + (s[0] as isize - b'0' as isize) as isize;
	    *s = &s[1..];
	}
	skip_whitespace(s);
	Ok(v)
    } else {
	Err("Parse error".to_string())
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
