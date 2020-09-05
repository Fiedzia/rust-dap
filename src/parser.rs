#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Short(char),
    Long(&'a str),
    Value(&'a str),
    PositionalSeparator,
    Error(String, Position),
    End
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    argument: usize,
    char_in_argument:u32,
}

impl Position {
    pub fn new(argument: usize) -> Position {
        Position {
            argument,
            char_in_argument: 0
        }
    }
    pub fn new_detailed(argument: usize, char_in_argument: u32) -> Position {
        Position {
            argument,
            char_in_argument,
        }
    }
}

pub struct TokenStream<'a> {
    args: &'a [&'a str],
    position: Position,
    state: State,
}

#[derive(Debug,PartialEq)]
enum State {
    Iterating,
    EndToken,
    Error,
    Done,
}


impl <'a>TokenStream<'a> {

    pub fn new(args: &'a [&'a str]) -> TokenStream<'a> {
        TokenStream {
            args: args,
            position: Position::new(0),
            state: if args.len() == 0 { State::EndToken } else { State::Iterating }
        }
    }

    pub fn next_argument(&mut self) {
        if self.position.argument + 1 < self.args.len() {
            self.position.argument += 1;
            self.position.char_in_argument = 0;
        } else {
            self.state = State::EndToken;
        }
    }
}

impl <'a>Iterator for TokenStream<'a> {
    type Item=Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {

        if self.state == State::Error || self.state == State::Done {
            None
        } else if self.state == State::EndToken {
            self.state = State::Done;
            Some(Token::End)
        } else {
            let arg = self.args[self.position.argument];

            if arg == "--" {
                self.next_argument();
                Some(Token::PositionalSeparator)
            } else if arg.starts_with("--") {
                self.next_argument();
                Some(Token::Long(&arg[2..]))
            } else if arg.starts_with('-') {

                let char_len = arg.chars().count();
                if char_len == 1 {
                    self.state = State::Error;
                    return Some(Token::Error("invalid value".to_string(), self.position.clone()))
                } else {
                    self.position.char_in_argument += 1;
                    let chr = arg.chars().nth(self.position.char_in_argument as usize).unwrap();
                    if chr == '-' || chr == ' ' {
                        self.state = State::Error;
                        return Some(Token::Error("invalid character".to_string(), self.position.clone()))
                    }
                    let result = Some(Token::Short(chr));
                    if self.position.char_in_argument as usize >= char_len - 1 {
                        self.next_argument();
                    }
                    result
                }
            } else {
                self.next_argument();
                Some(Token::Value(arg))
            }
        }
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
   args: &'a [&'a str], 
}

impl <'a>Parser<'a> {
    pub fn new(args: &'a [&'a str]) -> Parser {
        Parser {
            args: args
        }
    }

    pub fn iter(&self) -> TokenStream {
        TokenStream::new(self.args)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Token,
        Parser,
        Position
    };
    #[test]
    fn test_parser() {
        let parser = Parser::new(&[]);
        let tokens: Vec<Token> = parser.iter().collect();
        assert_eq!(tokens, vec![Token::End]);

        let parser = Parser::new(&["-v"]);
        let t: Vec<Token> = parser.iter().collect();
        assert_eq!(t, vec![Token::Short('v'), Token::End]);

        let parser = Parser::new(&["-s", "--long"]);
        let t: Vec<Token> = parser.iter().collect();
        assert_eq!(t, vec![Token::Short('s'), Token::Long("long"), Token::End]);

        let parser = Parser::new(&["-s", "--long", "value"]);
        let t: Vec<Token> = parser.iter().collect();
        assert_eq!(t, vec![Token::Short('s'), Token::Long("long"), Token::Value("value"), Token::End]);

        //test error handling
        let parser = Parser::new(&["-v "]);
        let t: Vec<Token> = parser.iter().collect();
        assert_eq!(t, vec![Token::Short('v'), Token::Error("invalid character".to_string(), Position::new_detailed(0, 2))]);

        let parser = Parser::new(&["- "]);
        let t: Vec<Token> = parser.iter().collect();
        assert_eq!(t, vec![Token::Error("invalid character".to_string(), Position::new_detailed(0, 1))]);
    }
}
