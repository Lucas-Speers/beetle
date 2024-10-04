use std::char;

use anyhow::Result;

#[derive(PartialEq, Clone)]
#[derive(Debug)]
pub enum Token {
    Import,
    Function,
    Identifier(String),
    NumLiteral(String),
    StringLiteral(String),
    Comment,
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
                    while let Some(_) = chars.next_if(|c| *c == '\n') {}
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
            _ => ()
        }
        // match identifiers
        let mut identifier = String::from(char);
        while let Some(char) = chars.next_if(|c| c.is_alphabetic()) {
            identifier.push(char);
        }
        tokens.push(Token::Identifier(identifier));
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