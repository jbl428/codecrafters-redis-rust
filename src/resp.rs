use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_while1};
use nom::character::complete::{crlf, digit1};
use nom::combinator::map_res;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum RespToken {
    SimpleString(String),
    SimpleError(String),
    BulkString(String),
    NullBulkString,
    Integer(i64),
    Array(Vec<RespToken>),
}

fn parse_simple_string(s: &str) -> IResult<&str, RespToken> {
    let (s, _) = tag("+")(s)?;
    let (s, value) = take_while1(|c| c != '\r' && c != '\n')(s)?;
    let (s, _) = crlf(s)?;

    Ok((s, RespToken::SimpleString(value.to_string())))
}

fn parse_bulk_string(s: &str) -> IResult<&str, RespToken> {
    let (s, _) = tag("$")(s)?;
    let (s, len): (&str, usize) = map_res(digit1, str::parse)(s)?;
    let (s, _) = crlf(s)?;
    let (s, value) = take(len)(s)?;
    let (s, _) = crlf(s)?;

    Ok((s, RespToken::BulkString(value.to_string())))
}

fn parse_null_bulk_string(s: &str) -> IResult<&str, RespToken> {
    let (s, _) = tag("$-1\r\n")(s)?;

    Ok((s, RespToken::NullBulkString))
}

fn parse_integer(s: &str) -> IResult<&str, RespToken> {
    let (s, _) = tag(":")(s)?;
    let (s, value): (&str, i64) = map_res(digit1, str::parse)(s)?;
    let (s, _) = crlf(s)?;

    Ok((s, RespToken::Integer(value)))
}

fn parse_array(s: &str) -> IResult<&str, RespToken> {
    let (s, _) = tag("*")(s)?;
    let (s, len): (&str, usize) = map_res(digit1, str::parse)(s)?;
    let (s, _) = crlf(s)?;

    let mut tokens = Vec::with_capacity(len);
    let mut s = s;
    for _ in 0..len {
        let (s_, token) = tokenize(s)?;
        s = s_;
        tokens.push(token);
    }

    Ok((s, RespToken::Array(tokens)))
}

pub fn tokenize(s: &str) -> IResult<&str, RespToken> {
    alt((
        parse_simple_string,
        parse_bulk_string,
        parse_null_bulk_string,
        parse_integer,
        parse_array,
    ))(s)
}

impl Display for RespToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespToken::SimpleString(s) => write!(f, "+{}\r\n", s),
            RespToken::SimpleError(s) => write!(f, "-{}\r\n", s),
            RespToken::BulkString(s) => write!(f, "${}\r\n{}\r\n", s.len(), s),
            RespToken::NullBulkString => write!(f, "$-1\r\n"),
            RespToken::Integer(i) => write!(f, ":{}\r\n", i),
            RespToken::Array(tokens) => {
                let mut s = String::new();
                s.push_str(&format!("*{}\r\n", tokens.len()));
                for token in tokens {
                    s.push_str(&format!("{}", token));
                }
                write!(f, "{}", s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_string_success() {
        let input = "+OK\r\n";
        let expected = RespToken::SimpleString("OK".to_string());
        let result = tokenize(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_simple_string failed"),
        }
    }

    #[test]
    fn bulk_string_success() {
        let input = "$6\r\nfoobar\r\n";
        let expected = RespToken::BulkString("foobar".to_string());
        let result = tokenize(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_bulk_string failed"),
        }
    }

    #[test]
    fn null_bulk_string_success() {
        let input = "$-1\r\n";
        let expected = RespToken::NullBulkString;
        let result = tokenize(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_null_bulk_string failed"),
        }
    }

    #[test]
    fn integer_success() {
        let input = ":1000\r\n";
        let expected = RespToken::Integer(1000);
        let result = tokenize(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_integer failed"),
        }
    }

    #[test]
    fn array_success() {
        let input = "*2\r\n+OK\r\n:1000\r\n";
        let expected = RespToken::Array(vec![
            RespToken::SimpleString("OK".to_string()),
            RespToken::Integer(1000),
        ]);
        let result = tokenize(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_array failed"),
        }
    }

    #[test]
    fn display_simple_string() {
        let token = RespToken::SimpleString("OK".to_string());
        let expected = "+OK\r\n";
        let result = token.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn display_bulk_string() {
        let token = RespToken::BulkString("foobar".to_string());
        let expected = "$6\r\nfoobar\r\n";
        let result = token.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn display_null_bulk_string() {
        let token = RespToken::NullBulkString;
        let expected = "$-1\r\n";
        let result = token.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn display_integer() {
        let token = RespToken::Integer(1000);
        let expected = ":1000\r\n";
        let result = token.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn display_array() {
        let token = RespToken::Array(vec![
            RespToken::SimpleString("OK".to_string()),
            RespToken::Integer(1000),
        ]);
        let expected = "*2\r\n+OK\r\n:1000\r\n";
        let result = token.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn display_nested_array() {
        let token = RespToken::Array(vec![
            RespToken::Array(vec![
                RespToken::SimpleString("OK".to_string()),
                RespToken::Integer(1000),
            ]),
            RespToken::Array(vec![
                RespToken::SimpleString("ERR".to_string()),
                RespToken::Integer(2000),
            ]),
        ]);
        let expected = "*2\r\n*2\r\n+OK\r\n:1000\r\n*2\r\n+ERR\r\n:2000\r\n";
        let result = token.to_string();

        assert_eq!(result, expected);
    }
}
