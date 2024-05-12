use RespToken::*;

use crate::resp::RespToken;

trait CommandHandler {
    fn try_handle(&self, token: &RespToken) -> Option<RespToken>;
}

struct PingHandler;

impl PingHandler {
    fn is_ping(&self, token: &RespToken) -> bool {
        match token {
            SimpleString(s) if s.to_uppercase() == "PING" => true,
            BulkString(s) if s.to_uppercase() == "PING" => true,
            Array(elements) if elements.len() == 1 => {
                self.is_ping(&elements[0])
            }
            _ => false,
        }
    }
}

impl CommandHandler for PingHandler {
    fn try_handle(&self, token: &RespToken) -> Option<RespToken> {
        if self.is_ping(token) {
            return Some(BulkString("PONG".to_string()));
        }
        None
    }
}

struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn try_handle(&self, token: &RespToken) -> Option<RespToken> {
        if let Array(elements) = token {
            if elements.len() != 2 {
                return None;
            }

            if let (BulkString(command), BulkString(argument)) = (&elements[0], &elements[1]) {
                if command.to_uppercase() == "ECHO" {
                    return Some(BulkString(argument.to_string()));
                }
            }
        }
        None
    }
}

pub struct CommandDispatcher {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CommandDispatcher {
    pub fn new() -> Self {
        CommandDispatcher {
            handlers: vec![Box::new(PingHandler), Box::new(EchoHandler)],
        }
    }

    pub fn dispatch(&self, token: &RespToken) -> RespToken {
        for handler in &self.handlers {
            if let Some(response) = handler.try_handle(token) {
                return response;
            }
        }
        SimpleError("unknown command".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping() {
        let dispatcher = CommandDispatcher::new();
        let token = BulkString("PinG".to_string());
        let response = dispatcher.dispatch(&token);

        assert_eq!(response, BulkString("PONG".to_string()));
    }

    #[test]
    fn test_echo() {
        let dispatcher = CommandDispatcher::new();
        let token = Array(vec![
            BulkString("ecHO".to_string()),
            BulkString("Hello".to_string()),
        ]);
        let response = dispatcher.dispatch(&token);

        assert_eq!(response, BulkString("Hello".to_string()));
    }

    #[test]
    fn test_unknown_command() {
        let dispatcher = CommandDispatcher::new();
        let token = BulkString("unknown".to_string());
        let response = dispatcher.dispatch(&token);

        assert_eq!(
            response,
            SimpleError("unknown command".to_string())
        );
    }
}
