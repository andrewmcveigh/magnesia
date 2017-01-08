use std::vec::*;
use std::str::Chars;
use std::iter::Peekable;
use types::*;
// use std::result::*;

#[derive(Debug)]
enum ReaderError {
    EOF,
    CannotUnreadChar,
    InvalidToken,
    InvalidSymbol
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

fn reader(s : Peekable<Chars>, buf : Vec<char>) -> Reader {
    Reader { s: s, buf: buf }
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

fn read_while(r : &mut Reader, p : &Fn(char) -> bool) -> ReaderResult<String> {
    let mut s = "".to_string();
    while let Ok(c) = read_char(r) {
        if p(c) {
            unread_char(r, c);
            return Ok(s)
        } else {
            s.push(c)
        }
    }
    Err(ReaderError::EOF)
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

fn read_token(r : &mut Reader, initch : char) -> ReaderResult<String> {
    read_while(r, &|c| is_macro_terminating(c) || is_whitespace(c))
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

// (defn- read-keyword
//   [reader initch opts pending-forms]
//   (let [ch (read-char reader)]
//     (if-not (whitespace? ch)
//       (let [token (read-token reader ch)
//             s (parse-symbol token)]
//         (if s
//           (let [^String ns (s 0)
//                 ^String name (s 1)]
//             (if (identical? \: (nth token 0))
//               (if ns
//                 (let [ns (resolve-ns (symbol (subs ns 1)))]
//                   (if ns
//                     (keyword (str ns) name)
//                     (reader-error reader "Invalid token: :" token)))
//                 (keyword (str *ns*) (subs name 1)))
//               (keyword ns name)))
//           (reader-error reader "Invalid token: :" token)))
// (reader-error reader "Invalid token: :"))))

fn read_keyword(r : &mut Reader) -> ReaderResult<Symbol> {
    match read_char(r) {
        Ok(c) if is_whitespace(c) => Err(ReaderError::InvalidToken),
        Ok(c) => let token = read_token(
    match read_token(r).and_then(parse_symbol) {
        Ok((None,     name)) => Ok(Symbol::SimpleSymbol(name)),
        Ok((Some(ns), name)) => Ok(Symbol::NamespacedSymbol(ns, name)),
        Err(e)               => Err(e)
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
    assert!(read_while(&mut r, f).expect("Failed read_while") == "abc");
    let mut r2 = string_reader("abc");
    match read_while(&mut r2, f) {
        Ok(_) => panic!("Shouldn't have succeeded read_while"),
        Err(ReaderError::EOF) => (),
        _ => panic!("Should have Err(ReaderError::EOF)")
    }
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
