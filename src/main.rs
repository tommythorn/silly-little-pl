// So, you want to write a parser (without pulling in a crate)?

type Source<'a> = &'a[u8];

fn main() {
    let mut source: Source = b" 42 + 56*78 - 40!"; // End in a sentinel to avoid testing for length everywhere
    match parse(&mut source) {
	Ok(v) => println!("Result {v} + {source:?}"),
	_ => println!("Parse Error"),
    }
}

fn parse(s: &mut Source) -> Result<isize, String> {
    skip_whitespace(s);
    parse_expr(s)
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
    if b'0' <= s[0] && s[0] <= b'9' {
	let mut v = 0;
	while b'0' <= s[0] && s[0] <= b'9' {
	    v = v*10 + (s[0] as isize - b'0' as isize) as isize;
	    *s = &s[1..];
	}
	skip_whitespace(s);
	Ok(v)
    } else {
	Err("Parse error".to_string())
    }
}

fn skip_whitespace(s: &mut Source) {
    while s[0] == b' ' {
	*s = &s[1..];
    }
}

fn token_match<'a>(s: &mut Source<'a>, expected: &[u8]) -> bool {
    if s[0..expected.len()] == *expected {
	*s = &s[expected.len()..];
	skip_whitespace(s);
	true
    } else {
	false
    }
}
