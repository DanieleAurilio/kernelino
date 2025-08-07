#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    Number(i64),
    String(String),

    //Keywords
    Local,

    Assign,
    DoubleQuote,
    Plus,
    Eof,
}

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    pub fn read_input(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.position < self.input.chars().count() {
            if self.is_white_space() {
                self.advance();
                continue;
            }

            if let Some(assign_token) = self.evaluate_sign() {
                tokens.push(assign_token);
                continue;
            }

            let token = self.is_keyword_or_identifier();
            tokens.push(token);
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

    fn is_keyword_or_identifier(&mut self) -> Token {
        let mut keyword = String::new();
        while let Some(char) = self.get_current_char() {
            if char.is_alphabetic() || char.is_ascii_alphabetic() || char == '_' {
                keyword.push(char);
                self.advance();
            } else {
                break;
            }
        }

        match keyword.as_str() {
            "local" => {
                return Token::Local;
            }
            _ => Token::Identifier(keyword),
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
                _ => {}   
            }
        }

        None
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn get_current_char(&mut self) -> Option<char> {
        self.input.chars().nth(self.position)
    }
}
