use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_while1};
use nom::character::complete::{crlf, digit1};
use nom::combinator::map_res;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum RespToken {
    SimpleString(String),
    BulkString(String),
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

fn tokenize(s: &str) -> IResult<&str, RespToken> {
    alt((
        parse_simple_string,
        parse_bulk_string,
        parse_integer,
        parse_array,
    ))(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_string_success() {
        let input = "+OK\r\n";
        let expected = RespToken::SimpleString("OK".to_string());
        let result = parse_simple_string(input);

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
        let result = parse_bulk_string(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_bulk_string failed"),
        }
    }

    #[test]
    fn integer_success() {
        let input = ":1000\r\n";
        let expected = RespToken::Integer(1000);
        let result = parse_integer(input);

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
        let result = parse_array(input);

        match result {
            Ok((s, token)) => {
                assert_eq!(s, "");
                assert_eq!(token, expected);
            }
            Err(_) => panic!("parse_array failed"),
        }
    }
}
