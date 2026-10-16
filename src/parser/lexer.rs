use std::sync::mpsc::{channel, Receiver, Sender};

use crate::parser::aptnode::APTNode;
use crate::parser::token::Token;
use crate::pic::coordinatesystem::CoordinateSystem;
use crate::pic::data::gradient::GradientData;
use crate::pic::data::grayscale::GrayscaleData;
use crate::pic::data::hsv::HSVData;
use crate::pic::data::mono::MonoData;
use crate::pic::data::rgb::RGBData;
use crate::pic::color::Color;
use crate::pic::pic::Pic;

// Function pointer definition must be wrapped in a struct to be recursive
struct StateFunction(fn(&mut Lexer) -> Option<StateFunction>);

pub struct Lexer<'a> {
    input: &'a str,
    start: usize,
    pos: usize,
    width: usize,
    token_sender: Sender<Token<'a>>,
    current_line: usize,
}

impl<'a> Lexer<'a> {
    pub fn begin_lexing(s: &'a str, sender: Sender<Token<'a>>) {
        let mut lexer = Lexer::<'a> {
            input: s,
            start: 0,
            pos: 0,
            width: 0,
            token_sender: sender,
            current_line: 0,
        };
        lexer.run();
    }

    fn run(&mut self) {
        let mut state = Some(StateFunction(Lexer::determine_token));
        while let Some(next_state) = state {
            state = next_state.0(self)
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            self.width = 0;
            None
        } else {
            let c = self.input[self.pos..]
                .chars()
                .next()
                .expect("unexpected end of input");
            // Advance by the UTF-8 byte length so `pos` always lands on a char
            // boundary; otherwise slicing `self.input[self.pos..]` panics.
            self.width = c.len_utf8();
            if Lexer::is_linebreak(c) {
                self.current_line += 1;
            }
            self.pos += self.width;
            Some(c)
        }
    }

    fn backup(&mut self) {
        // Undo the last `next` by its real byte width (0 if `next` hit EOF, so
        // this never underflows).
        self.pos -= self.width;
    }

    fn ignore(&mut self) {
        self.start = self.pos;
    }

    fn emit(&mut self, token: Token<'a>) {
        // println!("token:{:?}", token);
        self.token_sender.send(token).expect("token send failure");
        self.start = self.pos;
    }

    fn accept(&mut self, valid: &str) -> bool {
        if let Some(n) = self.next() {
            if valid.contains(n) {
                true
            } else {
                self.backup();
                false
            }
        } else {
            self.backup();
            false
        }
    }

    fn accept_run(&mut self, valid: &str) {
        loop {
            let n = self.next();
            if !(n.is_some() && valid.contains(n.expect("unexpected character in token stream"))) {
                break;
            }
        }
        self.backup();
    }

    fn lex_operation(l: &mut Lexer) -> Option<StateFunction> {
        l.accept_run("+-/*%abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.");
        l.emit(Token::Operation(&l.input[l.start..l.pos], l.current_line));
        Some(StateFunction(Lexer::determine_token))
    }

    fn lex_number(l: &mut Lexer) -> Option<StateFunction> {
        l.accept("-");
        let digits = "0123456789";
        l.accept_run(digits);
        if l.accept(".") {
            l.accept_run(digits);
        }
        if &l.input[l.start..l.pos] == "-" {
            // special case - could indicate start of number, or subtract operation
            l.emit(Token::Operation(&l.input[l.start..l.pos], l.current_line));
        } else {
            l.emit(Token::Constant(&l.input[l.start..l.pos], l.current_line));
        }
        Some(StateFunction(Lexer::determine_token))
    }

    fn determine_token(l: &mut Lexer) -> Option<StateFunction> {
        loop {
            {
                let c = l.next()?;
                if Lexer::is_white_space(c) {
                    l.ignore();
                } else if c == '(' {
                    l.emit(Token::OpenParen(l.current_line));
                } else if c == ')' {
                    l.emit(Token::CloseParen(l.current_line));
                } else if Lexer::is_start_of_number(c) {
                    return Some(StateFunction(Lexer::lex_number));
                } else {
                    return Some(StateFunction(Lexer::lex_operation));
                }
            }
        }
    }

    fn is_start_of_number(c: char) -> bool {
        c.is_ascii_digit() || c == '-' || c == '.'
    }

    fn is_white_space(c: char) -> bool {
        c == ' ' || c == '\n' || c == '\t' || c == '\r'
    }

    fn is_linebreak(c: char) -> bool {
        c == '\n'
    }
}

pub fn lisp_to_pic(code: String, coord: CoordinateSystem) -> Result<Pic, String> {
    // Lex fully into an unbounded channel, then parse. Running the lexer to
    // completion first (rather than concurrently with the parser) avoids the
    // race where the parser drops the receiver on an early error while the
    // lexer is still emitting, which panicked in `Lexer::emit`.
    let (sender, receiver) = channel();
    Lexer::begin_lexing(&code, sender);
    parse_pic(&receiver, coord)
}

pub fn expect_open_paren(receiver: &Receiver<Token>) -> Result<(), String> {
    let open_paren = receiver.recv().map_err(|_| "Unexpected end of file")?;
    match open_paren {
        Token::OpenParen(_) => Ok(()),
        Token::Operation(v, line) | Token::Constant(v, line) => {
            Err(format!("Expected '(' on line {}, got a '{}'", line, v))
        }
        _ => {
            Err(format!(
                "Expected '(' on line {}",
                extract_line_number(&open_paren)
            ))
        }
    }
}

pub fn expect_close_paren(receiver: &Receiver<Token>) -> Result<(), String> {
    let close_paren = receiver.recv().map_err(|_| "Unexpected end of file")?;
    match close_paren {
        Token::CloseParen(_) => Ok(()),
        _ => {
            Err(format!(
                "Expected '(' on line {}",
                extract_line_number(&close_paren)
            ))
        }
    }
}

pub fn expect_operation(s: &str, receiver: &Receiver<Token>) -> Result<(), String> {
    let op = receiver.recv().map_err(|_| "Unexpected end of file")?;
    match op {
        Token::Operation(op_str, _) => {
            if op_str.to_lowercase() == s {
                Ok(())
            } else {
                Err(format!(
                    "Expected '{}' on line {}, found {}",
                    s,
                    extract_line_number(&op),
                    op_str
                ))
            }
        }
        _ => {
            Err(format!(
                "Expected '{}' on line {}, found {:?}",
                s,
                extract_line_number(&op),
                op
            ))
        }
    }
}

pub fn expect_operations(ops: Vec<&str>, receiver: &Receiver<Token>) -> Result<String, String> {
    let op = receiver.recv().map_err(|_| "Unexpected end of file")?;
    for s in ops {
        if let Token::Operation(op_str, _) = op {
            if op_str.to_lowercase() == s.to_lowercase() {
                return Ok(op_str.to_string());
            }
        }
    }
    Err(format!(
        "Unexpected token on line {}",
        extract_line_number(&op),
    ))
}

pub fn expect_constant(receiver: &Receiver<Token>) -> Result<f32, String> {
    let op = receiver.recv().map_err(|_| "Unexpected end of file")?;
    match op {
        Token::Constant(vstr, line_number) => {
            let v = vstr
                .parse::<f32>()
                .map_err(|_| format!("Unable to parse number {} on line {}", vstr, line_number))?;
            Ok(v)
        }
        _ => {
            Err(format!(
                "Expected constant on line {}, found {:?}",
                extract_line_number(&op),
                op
            ))
        }
    }
}

pub fn parse_pic(
    receiver: &Receiver<Token>,
    coord_default: CoordinateSystem,
) -> Result<Pic, String> {
    let mut coord = coord_default;
    expect_open_paren(receiver)?;
    let pic_type = receiver.recv().map_err(|_| "Unexpected end of file")?;
    match pic_type {
        Token::Operation(s, line_number) => match &s.to_lowercase()[..] {
            "mono" => {
                if let Ok(coord_system) = expect_operations(
                    CoordinateSystem::list_all()
                        .iter()
                        .map(|x| x.as_str())
                        .collect(),
                    receiver,
                ) {
                    if let Ok(parsed) = coord_system.parse() {
                        coord = parsed;
                    }
                };
                Ok(Pic::Mono(MonoData {
                    c: APTNode::parse_apt_node(receiver)?,
                    coord,
                }))
            }
            "grayscale" => {
                if let Ok(coord_system) = expect_operations(
                    CoordinateSystem::list_all()
                        .iter()
                        .map(|x| x.as_str())
                        .collect(),
                    receiver,
                ) {
                    if let Ok(parsed) = coord_system.parse() {
                        coord = parsed;
                    }
                };
                Ok(Pic::Grayscale(GrayscaleData {
                    c: APTNode::parse_apt_node(receiver)?,
                    coord,
                }))
            }
            "rgb" => {
                if let Ok(coord_system) = expect_operations(
                    CoordinateSystem::list_all()
                        .iter()
                        .map(|x| x.as_str())
                        .collect(),
                    receiver,
                ) {
                    if let Ok(parsed) = coord_system.parse() {
                        coord = parsed;
                    }
                };
                Ok(Pic::RGB(RGBData {
                    r: APTNode::parse_apt_node(receiver)?,
                    g: APTNode::parse_apt_node(receiver)?,
                    b: APTNode::parse_apt_node(receiver)?,
                    coord,
                }))
            }
            "hsv" => {
                if let Ok(coord_system) = expect_operations(
                    CoordinateSystem::list_all()
                        .iter()
                        .map(|x| x.as_str())
                        .collect(),
                    receiver,
                ) {
                    if let Ok(parsed) = coord_system.parse() {
                        coord = parsed;
                    }
                };
                Ok(Pic::HSV(HSVData {
                    h: APTNode::parse_apt_node(receiver)?,
                    s: APTNode::parse_apt_node(receiver)?,
                    v: APTNode::parse_apt_node(receiver)?,
                    coord,
                }))
            }
            "gradient" => {
                if let Ok(coord_system) = expect_operations(
                    CoordinateSystem::list_all()
                        .iter()
                        .map(|x| x.as_str())
                        .collect(),
                    receiver,
                ) {
                    if let Ok(parsed) = coord_system.parse() {
                        coord = parsed;
                    }
                };
                let mut colors = Vec::new();
                expect_open_paren(receiver)?;
                expect_operation("colors", receiver)?;
                loop {
                    let _token = receiver.recv().map_err(|_| "Unexpected end of file")?;
                    match expect_operations(vec!["color", "stopcolor"], receiver) {
                        Err(e) => {
                            if e.starts_with("Unexpected token on line ") {
                                break;
                            } else {
                                // e.g. unexpected end of file mid-gradient.
                                return Err(e);
                            }
                        }
                        Ok(color_type) => {
                            let r = expect_constant(receiver)?;
                            let g = expect_constant(receiver)?;
                            let b = expect_constant(receiver)?;
                            if color_type.to_lowercase() == "color" {
                                colors.push((Color::new(r, g, b, 1.0), false));
                            } else {
                                colors.push((Color::new(r, g, b, 1.0), true));
                            }
                            expect_close_paren(receiver)?;
                        }
                    }
                }
                Ok(Pic::Gradient(GradientData {
                    colors,
                    index: APTNode::parse_apt_node(receiver)?,
                    coord,
                }))
            }
            _ => Err(format!("Unknown pic type {} at line {}", s, line_number)),
        },
        _ => Err("Invalid picture type".to_string()), //todo line number etc
    }
}

pub fn extract_line_number(token: &Token) -> usize {
    match token {
        Token::OpenParen(ln) | Token::CloseParen(ln) => *ln,
        Token::Constant(_, ln) | Token::Operation(_, ln) => *ln,
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;

    pub fn mock_lexer<'a>(code: &'a str, sender: Sender<Token<'a>>) -> Lexer<'a> {
        Lexer {
            input: code,
            start: 0,
            pos: 0,
            width: 1,
            token_sender: sender,
            current_line: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    const CODE: &str = r#"( RGB
    ( Sqrt ( Sin ( Abs Y ) ) )
    ( Atan ( Atan2 ( + X ( / ( Ridge Y -0.30377412 Y ) -0.4523425 ) ) ( + ( Turbulence 0.95225644 ( Tan Y ) Y ) -0.46079302 ) ) )
    ( Cell1 ( Ridge ( Ridge Y -0.83537865 -0.50440097 ) ( Atan2 Y X ) ( Sin 0.20003605 ) ) ( Sqrt ( Cell1 ( FBM Y X 0.8879242 ) 0.23509383 -0.4539826 ) ) ( Atan2 ( * X ( Ridge 0.6816149 X Y ) ) ( Cell1 ( Sin ( Turbulence X -0.25605845 Y ) ) -0.30595016 Y ) ) ) )
"#;

    #[test]
    fn test_lexer_next_linebreak() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        let expected = [(0, 0, 1, 1, 0),
            (1, 0, 2, 1, 0),
            (2, 0, 3, 1, 0),
            (3, 0, 4, 1, 0),
            (4, 0, 5, 1, 0),
            (5, 0, 6, 1, 1)];
        for (i, exp) in expected.iter().enumerate() {
            lexer.next();
            assert_eq!((i, lexer.start), (exp.0, exp.1));
            assert_eq!((i, lexer.pos), (exp.0, exp.2));
            assert_eq!((i, lexer.width), (exp.0, exp.3));
            assert_eq!((i, lexer.current_line), (exp.0, exp.4));
        }
    }
    #[test]
    fn test_lexer_next_end() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        lexer.pos = CODE.len();
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_lexer_backup() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        lexer.next();
        lexer.backup();
        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.current_line, 0);
        assert_eq!(lexer.start, 0);
        assert_eq!(lexer.width, 1);
    }

    #[test]
    fn test_lexer_ignore() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        lexer.next();
        lexer.ignore();
        assert_eq!(lexer.pos, 1);
        assert_eq!(lexer.current_line, 0);
        assert_eq!(lexer.start, 1);
        assert_eq!(lexer.width, 1);
    }

    #[test]
    fn test_lexer_emit() {
        let (sender, receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        lexer.next();
        lexer.emit(Token::OpenParen(66));
        assert_eq!(lexer.pos, 1);
        assert_eq!(lexer.current_line, 0);
        assert_eq!(lexer.start, 1);
        assert_eq!(lexer.width, 1);
        let msg = receiver.recv().unwrap();
        assert_eq!(msg, Token::OpenParen(66));
    }

    #[test]
    fn test_lexer_accept_start_ok() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        assert!(lexer.accept("("));
    }

    #[test]
    fn test_lexer_accept_start_not_ok() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        assert!(!lexer.accept(")"));
    }

    #[test]
    fn test_lexer_accept_spaced_ok() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        lexer.next();
        assert!(lexer.accept(" \t"));
    }

    #[test]
    fn test_lexer_accept_spaced_not_ok() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        lexer.next();
        assert!(!lexer.accept("abcdef"));
    }

    #[test]
    fn test_lexer_accept_end() {
        let (sender, _receiver) = channel::<Token>();
        let mut lexer = mock::mock_lexer(CODE, sender);
        for _i in 0..CODE.len() {
            lexer.next();
        }
        assert!(!lexer.accept(")"));
    }

    #[test]
    fn test_is_start_of_number() {
        assert!(!Lexer::is_start_of_number('a'));
        assert!(Lexer::is_start_of_number('0'));
        assert!(Lexer::is_start_of_number('9'));
    }

    #[test]
    fn test_is_white_space() {
        assert!(!Lexer::is_white_space('a'));
        assert!(Lexer::is_white_space(' '));
        assert!(Lexer::is_white_space('\t'));
        assert!(Lexer::is_white_space('\n'));
        assert!(Lexer::is_white_space('\r'));
    }

    #[test]
    fn test_is_linebreak() {
        assert!(!Lexer::is_linebreak('a'));
        assert!(!Lexer::is_linebreak(' '));
        assert!(!Lexer::is_linebreak('\t'));
        assert!(Lexer::is_linebreak('\n'));
        assert!(!Lexer::is_linebreak('\r'));
    }

    // todo: refactor into a separate module e.g. parser::token
    #[test]
    fn test_extract_line_number() {
        assert_eq!(extract_line_number(&Token::OpenParen(6)), 6);
        assert_eq!(extract_line_number(&Token::CloseParen(6)), 6);
        assert_eq!(extract_line_number(&Token::Operation("blablabla", 6)), 6);
        assert_eq!(extract_line_number(&Token::Constant("blablabla", 6)), 6);
    }
}
