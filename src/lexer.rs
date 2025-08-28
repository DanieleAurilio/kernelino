#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),

    //Types
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),

    //Keywords
    Local,
    Nil,
    Print,
    If,
    Then,
    Elseif,
    Else,
    End,
    For,
    Do,
    While,
    Repeat,
    Break,
    Until,
    Function,

    //Signs
    Assign,
    DoubleQuote,
    Plus,
    Minus,
    Divider,
    Modulo,
    Multiplier,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Gt,
    Eq,
    Lt,

    Not,
    NotEq,
    GtEq,
    LtEq,

    Eof,
}

pub struct Lexer {
    input: String,
    position: usize,
    seen_double_quote: bool,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            seen_double_quote: false,
        }
    }

    pub fn read_input(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.position < self.input.chars().count() {
            if self.is_white_space() {
                self.advance();
                continue;
            }

            if let Some(token_keyword) = self.read_keyword() {
                tokens.push(token_keyword);
                continue;
            }

            if let Some(token_sign) = self.evaluate_sign() {
                tokens.push(token_sign);
                continue;
            }

            if let Some(token_string) = self.read_string() {
                tokens.push(token_string);
                continue;
            }

            if let Some(token_number) = self.read_number() {
                tokens.push(token_number);
                continue;
            }
        }

        tokens.push(Token::Eof);
        println!("{:?}", tokens);
        return tokens;
    }

    fn is_white_space(&mut self) -> bool {
        if let Some(char) = self.get_current_char() {
            match char {
                ' ' | '\n' | '\t' | '\r' => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        } else {
            return false;
        }
    }

    fn read_string(&mut self) -> Option<Token> {
        let mut str: String = String::new();
        while let Some(char) = self.get_current_char() {
            if self.seen_double_quote == true && self.is_char_valid(char, true) {
                str.push(char);
                self.advance();
            } else {
                break;
            }
        }

        if str.len() > 0 {
            return Some(Token::String(str));
        } else {
            return None;
        }
    }

    fn evaluate_sign(&mut self) -> Option<Token> {
        if let Some(char) = self.get_current_char() {
            match char {
                '=' => {
                    if let Some(peek_char) = self.get_next_char() {
                        if peek_char == '=' {
                            self.advance();
                            return Some(Token::Eq);
                        }
                    } else {
                        self.advance();
                        return Some(Token::Assign);
                    }
                }
                '"' => {
                    self.set_seen_double_quote();
                    self.advance();
                    return Some(Token::DoubleQuote);
                }
                '+' => {
                    self.advance();
                    return Some(Token::Plus);
                }
                '-' => {
                    self.advance();
                    return Some(Token::Minus);
                }
                '%' => {
                    self.advance();
                    return Some(Token::Modulo);
                }
                '*' => {
                    self.advance();
                    return Some(Token::Multiplier);
                }
                '/' => {
                    self.advance();
                    return Some(Token::Divider);
                }
                '(' => {
                    self.advance();
                    return Some(Token::LParen);
                }
                ')' => {
                    self.advance();
                    return Some(Token::RParen);
                }
                '{' => {
                    self.advance();
                    return Some(Token::LBrace);
                }
                '}' => {
                    self.advance();
                    return Some(Token::RBrace);
                }
                '>' => {
                    if let Some(peek_char) = self.get_next_char() {
                        if peek_char == '=' {
                            self.advance();
                            return Some(Token::GtEq);
                        }
                    } else {
                        self.advance();
                        return Some(Token::Gt);
                    }
                }
                '<' => {
                    if let Some(peek_char) = self.get_next_char() {
                        if peek_char == '=' {
                            self.advance();
                            return Some(Token::LtEq);
                        }
                    } else {
                        self.advance();
                        return Some(Token::Lt);
                    }
                }
                '!' => {
                    if let Some(peek_char) = self.get_next_char() {
                        if peek_char == '=' {
                            self.advance();
                            return Some(Token::NotEq);
                        }
                    } else {
                        self.advance();
                        return Some(Token::Not);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn is_peek_char_digit(&self) -> bool {
        if let Some(char) = self.input.chars().nth(self.position + 1) {
            if char.is_digit(10) {
                return true;
            }
        }

        return true;
    }

    fn get_next_char(&self) -> Option<char> {
        if let Some(char) = self.input.chars().nth(self.position + 1) {
            return Some(char);
        } else {
            return None;
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        let mut number_str: String = String::new();
        while let Some(char) = self.get_current_char() {
            if char.is_ascii_digit() || (char == '.' && self.is_peek_char_digit()) {
                number_str.push(char);
                self.advance();
            } else {
                break;
            }
        }

        if number_str.len() > 0 {
            if number_str.contains('.') {
                if let Ok(float) = number_str.parse::<f64>() {
                    return Some(Token::Float(float));
                } else {
                    return None;
                }
            } else if let Ok(integer) = number_str.parse::<i64>() {
                return Some(Token::Integer(integer));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    fn read_keyword(&mut self) -> Option<Token> {
        let mut keyword = String::new();

        while let Some(char) = self.get_current_char() {
            if self.seen_double_quote == false && char.is_alphabetic() {
                keyword.push(char);
                self.advance();
            } else {
                break;
            }
        }

        if keyword.len() == 0 {
            return None;
        }

        match keyword.as_str() {
            "local" => return Some(Token::Local),
            "print" => return Some(Token::Print),
            "false" => return Some(Token::Boolean(false)),
            "true" => return Some(Token::Boolean(true)),
            "if" => return Some(Token::If),
            "then" => return Some(Token::Then),
            "elseif" => return Some(Token::Elseif),
            "else" => return Some(Token::Else),
            "end" => return Some(Token::End),
            "for" => return Some(Token::For),
            "do" => return Some(Token::Do),
            "while" => return Some(Token::While),
            "repeat" => return Some(Token::Repeat),
            "until" => return Some(Token::Until),
            "break" => return Some(Token::Break),
            "nil" => return Some(Token::Nil),
            "function" => return Some(Token::Function),
            _ => return Some(Token::Identifier(keyword)),
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn get_current_char(&mut self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn set_seen_double_quote(&mut self) {
        if self.seen_double_quote == true {
            self.seen_double_quote = false;
        } else {
            self.seen_double_quote = true;
        }
    }

    fn is_char_valid(&self, char: char, include_punctuation: bool) -> bool {
        if char.is_alphabetic()
            || char.is_ascii_alphabetic()
            || (include_punctuation == true && char != '"' && char.is_ascii_punctuation())
            || char == '_'
        {
            true
        } else {
            false
        }
    }
}
