use anyhow::Result;

#[derive(PartialEq, Clone)]
#[derive(Debug)]
pub enum Token {
    Func,
    Import,
    Let,
    If,
    Else,
    SetEqual,
    IsEqual,
    Semicolon,
    Comma,
    Period,
    Colon,
    NameSpace,
    LessThan,
    GreaterThan,
    LeftParentheses,
    RightParentheses,
    LeftBracket,
    RightBracket,
    LeftCurlyBracket,
    RightCurlyBracket,
    Identifier(String),
    NumLiteral(String),
    StringLiteral(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {

    let mut chars = input.chars().peekable();
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(char) = chars.next() {
        // match whitespace
        if char.is_whitespace() {
            continue;
        }
        // match numbers
        if char.is_ascii_digit() {
            let mut lit_number = String::from(char);
            while let Some(char) = chars.next_if(|c| c.is_ascii_digit()) {
                lit_number.push(char);
            }
            tokens.push(Token::NumLiteral(lit_number));
            continue;
        }
        // match sybols
        match char {
            '/' => {
                if let Some('/') = chars.peek() {
                    while let Some(_) = chars.next_if(|c| *c != '\n') {}
                } else {
                    // TODO division operator
                }
                continue;
            }
            '"' => { // TODO: string escaping
                let mut string_literal = String::new();
                while let Some(char) = chars.next() {
                    if char == '"' {
                        break;
                    }
                    string_literal.push(char);
                }
                tokens.push(Token::StringLiteral(string_literal));
                continue;
            }
            '(' => {
                tokens.push(Token::LeftParentheses);
                continue;
            }
            ')' => {
                tokens.push(Token::RightParentheses);
                continue;
            }
            '{' => {
                tokens.push(Token::LeftCurlyBracket);
                continue;
            }
            '}' => {
                tokens.push(Token::RightCurlyBracket);
                continue;
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                continue;
            }
            ']' => {
                tokens.push(Token::RightBracket);
                continue;
            }
            ';' => {
                tokens.push(Token::Semicolon);
                continue;
            }
            '=' => {
                if let Some('=') = chars.peek() {
                    tokens.push(Token::IsEqual);
                    chars.next();
                    continue;
                }
                tokens.push(Token::SetEqual);
                continue;
            }
            ':' => {
                if let Some(':') = chars.peek() {
                    tokens.push(Token::NameSpace);
                    chars.next();
                    continue;
                }
                tokens.push(Token::Colon);
                continue;
            }
            '<' => {
                tokens.push(Token::LessThan);
                continue;
            }
            '>' => {
                tokens.push(Token::GreaterThan);
                continue;
            }
            ',' => {
                tokens.push(Token::Comma);
                continue;
            }
            '.' => {
                tokens.push(Token::Period);
                continue;
            }
            _ => ()
        }
        // match identifiers/keywords
        let mut identifier = String::from(char);
        while let Some(char) = chars.next_if(|c| c.is_alphabetic()) {
            identifier.push(char);
        }
        tokens.push(
            match identifier.as_str() {
                "func" => Token::Func,
                "import" => Token::Import,
                "let" => Token::Let,
                "if" => Token::If,
                "else" => Token::Else,
                _ => Token::Identifier(identifier)
            }
        );
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    #[test]
    fn number_literal() -> Result<()> {
        let num_literal = "42a";

        let tokens = tokenize(&num_literal)?;
        println!("tokens computed: {:?}", tokens);

        let mut tokens = tokens.iter();

        if let Some(Token::NumLiteral(x)) = tokens.next() {
            assert_eq!(x, "42")
        } else {
            return Err(anyhow!("Did not detect num literal"));
        }

        Ok(())
    }
}