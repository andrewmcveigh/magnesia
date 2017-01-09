use std::fmt;
use std::iter::Peekable;
use std::str::Chars;
use std::vec::*;
use types::*;
// use std::result::*;

#[derive(Debug, PartialEq, Eq)]
enum ReaderError {
    EOF,
    CannotUnreadChar,
    InvalidToken,
    InvalidCharacter,
    InvalidSymbol,
    InvalidKeyword
}

struct Reader<'a> {
    s:   Peekable<Chars<'a>>,
    buf: Vec<char>
}

type ReaderResult<T> = Result<T, ReaderError>;

fn string_reader(s : &str) -> Reader {
    let s = s.clone();
    Reader { s:  s.chars().peekable(), buf: Vec::new() }
}

fn peek_char(r : &mut Reader) -> ReaderResult<char> {
    if r.buf.len() > 0 {
        Ok(r.buf[0])
    } else {
        match r.s.peek() {
            Some(c) => Ok(*c),
            None => Err(ReaderError::EOF)
        }
    }
}

fn read_char(r : &mut Reader) -> ReaderResult<char> {
    match r.buf.pop() {
        Some (c) => Ok(c),
        None => match r.s.next() {
            Some(c) => Ok(c),
            None => Err(ReaderError::EOF)
        }
    }
}

fn unread_char(r : &mut Reader, c : char) -> ReaderResult<()> {
    Ok(r.buf.push(c))
}

fn read_while(r : &mut Reader, p : &Fn(char) -> bool, eof_err : bool) -> ReaderResult<String> {
    let mut s = "".to_string();
    while let Ok(c) = read_char(r) {
        if p(c) {
            unread_char(r, c);
            return Ok(s)
        } else {
            s.push(c)
        }
    };
    if eof_err { Err(ReaderError::EOF) } else { Ok(s) }
}

fn is_macro_terminating(c : char) -> bool {
    match c {
        '"' | ';' | '@' | '^' | '`' | '~' | '\\' => true,
        '(' | ')' | '[' | ']' | '{' | '}' => true,
        _ => false
    }
}

fn is_whitespace(c : char) -> bool {
    match c {
        ' ' | '\n' | '\r' => true,
        _ => false
    }
}

fn escape_char<'a>(c : char) -> ReaderResult<&'a str> {
    match c {
        't'  => Ok("\t"),
        'r'  => Ok("\r"),
        'n'  => Ok("\n"),
        '\\' => Ok("\\"),
        '"'  => Ok("\""),
        _    => Err(ReaderError::InvalidCharacter)
    }
}

fn read_string_type(r : &mut Reader, _ : char) -> ReaderResult<String> {
    let mut s = "".to_string();
    loop {
        let c = read_char(r);
        match c {
            Ok('"') => { return Ok(s) },
            Ok('\\') => try!(read_char(r)
                             .and_then(escape_char)
                             .map(|c| s.push_str(c))),
            Ok(c) => s.push(c),
            Err(e) => return Err(e)
        }
    };
    Ok(s)
}

fn read_regex(r : &mut Reader, _ : char) -> ReaderResult<Pattern> {
    read_while(r, &|c| c == '"', true).map(|s| Pattern(s))
}

fn read_token(r : &mut Reader, initch : char) -> ReaderResult<String> {
    read_while(r, &|c| is_macro_terminating(c) || is_whitespace(c), false)
        .map(|s| initch.to_string() + &s)
}

fn parse_symbol(token : String) ->
    Result<(Option<String>, String), ReaderError> {
    if token.is_empty() || token.ends_with(":") || token.starts_with("::") {
        Err(ReaderError::InvalidSymbol)
    } else {
        let mut tokens = token.rsplit("/");
        let name = tokens.next().map(str::to_string);
        let ns = tokens.next().map(str::to_string);
        match name {
            Some(name) => Ok((ns, name)),
            None => Err(ReaderError::InvalidSymbol)
        }
    }
}

fn read_symbol(r : &mut Reader, initch : char) -> ReaderResult<Symbol> {
    match read_token(r, initch) {
        Ok(ref s) if s == "/" => Ok(Symbol::SimpleSymbol("/".to_string())),
        Ok(s) => match parse_symbol(s) {
            Ok((None,     name)) => Ok(Symbol::SimpleSymbol(name)),
            Ok((Some(ns), name)) => Ok(Symbol::NamespacedSymbol(ns, name)),
            Err(e)               => Err(e)
        },
        Err(e) => Err(e)
    }
}

fn read_keyword(r : &mut Reader, _ : char) -> ReaderResult<Keyword> {
    match read_char(r) {
        Ok(c) if is_whitespace(c) => Err(ReaderError::InvalidToken),
        Ok(c) => {
            let token = try!(read_token(r, c));
            let (ns, name) = try!(parse_symbol(token.clone()));
            match token.chars().nth(0) {
                Some(':') => Err(ReaderError::InvalidKeyword),
                _ => match ns {
                    Some(ns) => Ok(Keyword::NamespacedKeyword(ns, name)),
                    None => Ok(Keyword::SimpleKeyword(name))
                }
            }
        },
        Err(e) => Err(e)
    }
}

#[test]
fn peek_char_test() {
    let mut r = string_reader("c");
    assert!(peek_char(&mut r).expect("Failed peek_char") == 'c');
}

#[test]
fn read_char_test() {
    let mut r = string_reader("c");
    assert!(read_char(&mut r).expect("Failed read_char") == 'c');
}

#[test]
fn unread_char_test() {
    let mut r = string_reader("bc");
    unread_char(&mut r, 'a');
    assert!(read_char(&mut r).expect("Failed read_char") == 'a');
    assert!(read_char(&mut r).expect("Failed read_char") == 'b');
    assert!(read_char(&mut r).expect("Failed read_char") == 'c');
}

#[test]
fn read_while_test() {
    let mut r = string_reader("abc ");
    let f = & |c| c == ' ';
    assert!(read_while(&mut r, f, false).expect("Failed read_while") == "abc");
    let mut r2 = string_reader("abc");
    match read_while(&mut r2, f, true) {
        Ok(_) => panic!("Shouldn't have succeeded read_while"),
        Err(ReaderError::EOF) => (),
        _ => panic!("Should have Err(ReaderError::EOF)")
    }
}

#[test]
fn read_string_type_test() {
    let mut r = string_reader("abc\"");
    assert!(read_string_type(&mut r, '"') == Ok("abc".to_string()));
    let mut r = string_reader("abc\\\\\"");
    assert!(read_string_type(&mut r, '"') == Ok("abc\\".to_string()));
    let mut r = string_reader("abc");
    assert!(read_string_type(&mut r, '"') == Err(ReaderError::EOF));
}

#[test]
fn read_regex_test() {
    let mut r = string_reader("abc\"");
    assert!(read_regex(&mut r, '"') == Ok(Pattern("abc".to_string())));
    let mut r = string_reader("abc\\\"");
    assert!(read_regex(&mut r, '"') == Ok(Pattern("abc\\".to_string())));
    let mut r = string_reader("abc");
    assert!(read_regex(&mut r, '"') == Err(ReaderError::EOF));
}

#[test]
fn read_token_test() {
    let mut r = string_reader("bc ");
    assert!(read_token(&mut r, 'a').expect("Failed read_token") == "abc");
    let mut r2 = string_reader("bc{");
    assert!(read_token(&mut r2, 'a').expect("Failed read_token") == "abc");
    let mut r3 = string_reader("bc\\");
    assert!(read_token(&mut r3, 'a').expect("Failed read_token") == "abc");
}

#[test]
fn parse_symbol_test() {
    assert!(parse_symbol("abc".to_string()) == Ok((None, "abc".to_string())));
    assert!(parse_symbol(":a".to_string()) == Ok((None, ":a".to_string())));
    assert!(parse_symbol(":".to_string()) == Err(ReaderError::InvalidSymbol));
    assert!(parse_symbol(":a".to_string()) == Ok((None, ":a".to_string())));
}

#[test]
fn read_symbol_test() {
    let mut r = string_reader("bc ");
    let sym = read_symbol(&mut r, 'a').expect("Failed read_token");
    assert!(sym == Symbol::SimpleSymbol("abc".to_string()));
    let mut r2 = string_reader("s1/abc ");
    let sym2 = read_symbol(&mut r2, 'n').expect("Failed read_token");
    assert!(sym2 == Symbol::NamespacedSymbol("ns1".to_string(), "abc".to_string()));
    let mut r3 = string_reader(" ");
    let sym3 = read_symbol(&mut r3, '/').expect("Failed read_token");
    assert!(sym3 == Symbol::SimpleSymbol("/".to_string()));
}

#[test]
fn read_keyword_test() {
    let mut r = string_reader("abc ");
    let key = read_keyword(&mut r, ':').expect("Failed read_token");
    assert!(key == Keyword::SimpleKeyword("abc".to_string()));
    let mut r2 = string_reader("ns1/abc ");
    let key = read_keyword(&mut r2, ':').expect("Failed read_token");
    assert!(key == Keyword::NamespacedKeyword("ns1".to_string(), "abc".to_string()));
    let mut r3 = string_reader(" ");
    match read_keyword(&mut r3, ':') {
        Err(err) => assert!(err == ReaderError::InvalidToken),
        _ => panic!("Should have ReaderError::InvalidToken")
    }
    let mut r3 = string_reader(":a ");
    match read_keyword(&mut r3, ':') {
        Err(err) => assert!(err == ReaderError::InvalidKeyword),
        _ => panic!("Should have ReaderError::InvalidKeyword")
    }
}
