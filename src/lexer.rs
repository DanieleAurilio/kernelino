#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    //Types
    Integer(i64),
    Float(f64),
    String(String),

    //Keywords
    Local,
    Print,

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

            if let Some(assign_token) = self.evaluate_sign() {
                if self.skip_sign_in_double_qoute(assign_token.clone()) == false {
                    tokens.push(assign_token);
                    continue;
                }
            }

            if let Some(token_keyword_string) = self.is_keyword_or_string() {
                tokens.push(token_keyword_string);
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

    fn is_keyword_or_string(&mut self) -> Option<Token> {
        let mut keyword: String = String::new();
        while let Some(char) = self.get_current_char() {
            if char.is_alphabetic() || char.is_ascii_alphabetic() || char == '_' || (self.seen_double_quote == true && char.is_ascii_punctuation())  {
                keyword.push(char);
                self.advance();
            } else {
                break;
            }
        }

        if keyword.len() > 0 {
            match keyword.as_str() {
                "local" => {
                    return Some(Token::Local);
                }
                "print" => return Some(Token::Print),
                _ => Some(Token::String(keyword)),
            }
        } else {
            return None;
        }
    }

    fn evaluate_sign(&mut self) -> Option<Token> {
        if let Some(char) = self.get_current_char() {
            match char {
                '=' => {
                    self.advance();
                    return Some(Token::Assign);
                }
                '"' => {
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
                    if self.seen_double_quote == false {
                        self.advance();
                        return Some(Token::LParen);
                    } else {
                        return None;
                    }
                }
                ')' => {
                    if self.seen_double_quote == false {
                        self.advance();
                        return Some(Token::RParen);
                    } else {
                        return None;
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

    fn advance(&mut self) {
        self.position += 1;
    }

    fn get_current_char(&mut self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn skip_sign_in_double_qoute(&mut self, token: Token) -> bool {
        if token == Token::DoubleQuote {
            if self.seen_double_quote == true {
                self.seen_double_quote = false;
            } else {
                self.seen_double_quote = true;
            }
        }

        return self.seen_double_quote;
    }
}
