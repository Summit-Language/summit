use super::token::Token;
use super::utils::process_escapes;

/// Converts Summit source code into a list of tokens.
///
/// # Parameters
/// - `source`: The Summit source code to tokenize
///
/// # Returns
/// A list of tokens or an error message
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if chars[i] == '"' {
            match tokenize_string(&chars, &mut i) {
                Ok(token) => tokens.push(token),
                Err(e) => return Err(e),
            }
            continue;
        }

        if chars[i].is_ascii_digit() {
            match tokenize_number(&chars, &mut i) {
                Ok(token) => tokens.push(token),
                Err(e) => return Err(e),
            }
            continue;
        }

        if chars[i].is_alphabetic() || chars[i] == '_' {
            tokens.push(tokenize_identifier(&chars, &mut i));
            continue;
        }

        if i + 1 < chars.len() {
            if let Some(token) = tokenize_two_char_operator(&chars, i) {
                tokens.push(token);
                i += 2;
                continue;
            }
        }

        match tokenize_single_char(&chars[i]) {
            Ok(token) => {
                tokens.push(token);
                i += 1;
            }
            Err(e) => return Err(e),
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

/// Tokenizes a string literal from the source code.
///
/// # Parameters
/// - `chars`: The source characters
/// - `i`: Mutable reference to the current position in the source
///
/// # Returns
/// A `Token::StringLiteral` or an error message
fn tokenize_string(chars: &[char], i: &mut usize) -> Result<Token, String> {
    *i += 1;
    let start = *i;

    while *i < chars.len() && chars[*i] != '"' {
        if chars[*i] == '\\' {
            *i += 2;
        } else {
            *i += 1;
        }
    }

    if *i >= chars.len() {
        return Err("Unterminated string literal".to_string());
    }

    let string: String = chars[start..*i].iter().collect();
    *i += 1;

    Ok(Token::StringLiteral(process_escapes(&string)))
}

/// Tokenizes a numeric literal from the source code.
///
/// # Parameters
/// - `chars`: The source characters
/// - `i`: Mutable reference to the current position in the source
///
/// # Returns
/// A `Token::IntLiteral` or an error message
fn tokenize_number(chars: &[char], i: &mut usize) -> Result<Token, String> {
    let start = *i;

    while *i < chars.len() && chars[*i].is_ascii_digit() {
        *i += 1;
    }

    let num_str: String = chars[start..*i].iter().collect();

    match num_str.parse::<u128>() {
        Ok(n) => Ok(Token::IntLiteral(n)),
        Err(_) => Err(format!(
            "Integer literal '{}' is too large (maximum value is {})",
            num_str,
            u128::MAX
        )),
    }
}

/// Tokenizes an identifier or keyword from the source code.
///
/// # Parameters
/// - `chars`: The source characters
/// - `i`: Mutable reference to the current position in the source
///
/// # Returns
/// A token (either identifier or keyword)
fn tokenize_identifier(chars: &[char], i: &mut usize) -> Token {
    let start = *i;

    while *i < chars.len() && (chars[*i].is_alphanumeric() || chars[*i] == '_') {
        *i += 1;
    }

    let word: String = chars[start..*i].iter().collect();

    match word.as_str() {
        "import" => Token::Import,
        "func" => Token::Func,
        "ret" => Token::Ret,
        "var" => Token::Var,
        "const" => Token::Const,
        "comptime" => Token::Comptime,
        "if" => Token::If,
        "else" => Token::Else,
        "while" => Token::While,
        "when" => Token::When,
        "expect" => Token::Expect,
        "is" => Token::Is,
        "fallthrough" => Token::Fallthrough,
        "next" => Token::Next,
        "stop" => Token::Stop,
        "for" => Token::For,
        "in" => Token::In,
        "to" => Token::To,
        "through" => Token::Through,
        "by" => Token::By,
        "where" => Token::Where,
        "not" => Token::Not,
        "and" => Token::And,
        "or" => Token::Or,
        "null" => Token::Null,
        "true" => Token::True,
        "false" => Token::False,
        "bool" => Token::Type("bool".to_string()),
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16"
        | "u32" | "u64" | "u128" | "void" | "str" => {
            Token::Type(word)
        }
        _ => Token::Identifier(word),
    }
}

/// Tokenizes a two character operator from the source code.
///
/// # Parameters
/// - `chars`: The source characters
/// - `i`: Current position in the source
///
/// # Returns
/// A token if it's a valid two-character operator, or `None`
fn tokenize_two_char_operator(chars: &[char], i: usize) -> Option<Token> {
    let two_char = format!("{}{}", chars[i], chars[i + 1]);

    match two_char.as_str() {
        "::" => Some(Token::DoubleColon),
        "==" => Some(Token::Equal),
        "!=" => Some(Token::NotEqual),
        "<=" => Some(Token::LessEqual),
        ">=" => Some(Token::GreaterEqual),
        "->" => Some(Token::Arrow),
        _ => None,
    }
}

/// Tokenizes a single character operator or delimiter.
///
/// # Parameters
/// - `ch`: The character to tokenize
///
/// # Returns
/// A token or an error message
fn tokenize_single_char(ch: &char) -> Result<Token, String> {
    match ch {
        '+' => Ok(Token::Plus),
        '-' => Ok(Token::Minus),
        '*' => Ok(Token::Star),
        '/' => Ok(Token::Slash),
        '%' => Ok(Token::Percent),
        '=' => Ok(Token::Assign),
        '<' => Ok(Token::LeftAngle),
        '>' => Ok(Token::RightAngle),
        ':' => Ok(Token::Colon),
        ';' => Ok(Token::Semicolon),
        ',' => Ok(Token::Comma),
        '(' => Ok(Token::LeftParen),
        ')' => Ok(Token::RightParen),
        '{' => Ok(Token::LeftBrace),
        '}' => Ok(Token::RightBrace),
        '[' => Ok(Token::LeftBracket),
        ']' => Ok(Token::RightBracket),
        '?' => Ok(Token::Question),
        _ => Err(format!("Unexpected character: {}", ch)),
    }
}