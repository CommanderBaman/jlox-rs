use strum::Display;
use thiserror::Error;

#[derive(Clone, Debug, Display)]
pub enum Token {
    // single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character token
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    #[strum(to_string = "Identifier('{0}')")]
    Identifier(String),
    #[strum(to_string = "String('{0}')")]
    String(String),
    #[strum(to_string = "Number({0})")]
    Number(f64),

    // keywords
    For,
    While,
    If,
    Else,
    Return,
    And,
    Or,
    This,
    True,
    False,
    Var,
    Nil,
    Fun,
    Class,
    Super,
    Print,

    // special
    EndOfFile,
}

#[derive(Error, Debug)]
#[error("tokenisation error at line {line_number} : {source}")]
pub struct TokenError {
    line_number: usize,
    source: TokenErrorSource,
}

#[derive(Error, Debug)]
pub enum TokenErrorSource {
    #[error("could not recognize token: {0}")]
    UnrecognizedToken(char),
    #[error("could not parse token as number: {0}")]
    UnparsableNumber(String),
    #[error("the given string was not terminated: \"{0}\"")]
    UnfinishedString(String),
}

fn keyword_to_token(keyword: &str) -> Option<Token> {
    match keyword {
        "for" => Some(Token::For),
        "while" => Some(Token::While),
        "if" => Some(Token::If),
        "else" => Some(Token::Else),
        "return" => Some(Token::Return),
        "and" => Some(Token::And),
        "or" => Some(Token::Or),
        "this" => Some(Token::This),
        "true" => Some(Token::True),
        "false" => Some(Token::False),
        "var" => Some(Token::Var),
        "nil" => Some(Token::Nil),
        "fun" => Some(Token::Fun),
        "class" => Some(Token::Class),
        "super" => Some(Token::Super),
        "print" => Some(Token::Print),
        _ => None, // Not a keyword, likely an identifier
    }
}

// NOTE: line ending at a token is confusing
// currently i just recognize it as a token and I do not
// know what complications arise from it
// normally it should be \n, maybe i should add an assert
pub fn scan_tokens(line: &str) -> Result<Vec<Token>, TokenError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = line.chars().peekable();
    let mut line_number: usize = 1;
    loop {
        let char = chars.next();
        if char.is_none() {
            break;
        }
        // safe to unwrap because of none check above
        let char = char.unwrap();
        match char {
            // single character tokens
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '{' => tokens.push(Token::LeftBrace),
            '}' => tokens.push(Token::RightBrace),
            ',' => tokens.push(Token::Comma),
            '.' => tokens.push(Token::Dot),
            '-' => tokens.push(Token::Minus),
            '+' => tokens.push(Token::Plus),
            ';' => tokens.push(Token::Semicolon),
            '*' => tokens.push(Token::Star),
            // two character tokens
            '!' => match chars.peek() {
                Some('=') => {
                    // consume the equal char
                    chars.next();
                    tokens.push(Token::BangEqual)
                }
                Some(_) => tokens.push(Token::Bang),
                None => tokens.push(Token::Bang),
            },
            '=' => match chars.peek() {
                Some('=') => {
                    // consume the equal char
                    chars.next();
                    tokens.push(Token::EqualEqual)
                }
                Some(_) => tokens.push(Token::Equal),
                None => tokens.push(Token::Equal),
            },
            '>' => match chars.peek() {
                Some('=') => {
                    // consume the equal char
                    chars.next();
                    tokens.push(Token::GreaterEqual)
                }
                Some(_) => tokens.push(Token::Greater),
                None => tokens.push(Token::Greater),
            },
            '<' => match chars.peek() {
                Some('=') => {
                    // consume the equal char
                    chars.next();
                    tokens.push(Token::LessEqual)
                }
                Some(_) => tokens.push(Token::Less),
                None => tokens.push(Token::Less),
            },
            // comments
            '/' => match chars.peek() {
                Some('/') => {
                    // consume the slash char
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('\n') => {
                                line_number += 1;
                                break;
                            }
                            Some(_) => continue,
                            None => break,
                        }
                    }
                }
                Some(_) => tokens.push(Token::Slash),
                None => tokens.push(Token::Slash),
            },
            // whitespace
            ' ' => continue,
            '\r' => continue,
            '\t' => continue,
            '\n' => {
                line_number += 1;
                continue;
            }
            // string literals
            '"' => {
                let mut buffer = String::new();
                loop {
                    let c = chars.next().ok_or_else(|| TokenError {
                        line_number,
                        source: TokenErrorSource::UnfinishedString(
                            buffer.trim().to_string(),
                        ),
                    })?;

                    if c == '"' {
                        break;
                    }
                    buffer.push(c);
                }
                tokens.push(Token::String(buffer));
            }
            // literals
            c => {
                // numbers
                if c.is_ascii_digit() {
                    let mut buffer = String::new();
                    buffer.push(c);
                    while let Some(&p) = chars.peek() {
                        if p == '.' || p.is_ascii_digit() {
                            chars.next();
                            buffer.push(p);
                            continue;
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(buffer.parse().map_err(
                        |_| TokenError {
                            line_number,
                            source: TokenErrorSource::UnparsableNumber(buffer),
                        },
                    )?));
                    continue;
                }
                if c.is_alphabetic() {
                    let mut buffer = String::new();
                    buffer.push(c);
                    // doing it a different way than above
                    // i believe this is harder to read but the suggestion
                    // said it was more idiomatic
                    while matches!(chars.peek(), Some(&p) if p.is_alphabetic())
                    {
                        // safe to unwrap because of check above
                        buffer.push(chars.next().unwrap());
                    }
                    // if one of the keywords, then add as an keyword, otherwise identifier
                    if let Some(token) = keyword_to_token(&buffer) {
                        tokens.push(token);
                    } else {
                        tokens.push(Token::Identifier(buffer));
                    }
                    continue;
                }
                return Err(TokenError {
                    line_number,
                    source: TokenErrorSource::UnrecognizedToken(c),
                });
            }
        }
    }
    Ok(tokens)
}
