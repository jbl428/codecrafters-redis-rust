use crate::resp::RespToken;

trait CommandHandler {
    fn try_handle(&self, token: &RespToken) -> Option<RespToken>;
}

struct PingHandler;

impl CommandHandler for PingHandler {
    fn try_handle(&self, token: &RespToken) -> Option<RespToken> {
        match token {
            RespToken::BulkString(s) if s.to_uppercase() == "PING" => {
                Some(RespToken::BulkString("PONG".to_string()))
            }
            _ => None,
        }
    }
}

struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn try_handle(&self, token: &RespToken) -> Option<RespToken> {
        match token {
            RespToken::Array(tokens) => match (&tokens[0], &tokens[1]) {
                (RespToken::BulkString(command), RespToken::BulkString(arg))
                if command.to_uppercase() == "ECHO" =>
                    {
                        Some(RespToken::BulkString(arg.clone()))
                    }
                _ => None,
            },
            _ => None,
        }
    }
}

struct CommandDispatcher {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CommandDispatcher {
    fn new() -> Self {
        CommandDispatcher {
            handlers: vec![Box::new(PingHandler), Box::new(EchoHandler)],
        }
    }

    fn dispatch(&self, token: &RespToken) -> Option<RespToken> {
        for handler in &self.handlers {
            if let Some(response) = handler.try_handle(token) {
                return Some(response);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping() {
        let dispatcher = CommandDispatcher::new();
        let token = RespToken::BulkString("PinG".to_string());
        let response = dispatcher.dispatch(&token);

        assert_eq!(response, Some(RespToken::BulkString("PONG".to_string())));
    }

    #[test]
    fn test_echo() {
        let dispatcher = CommandDispatcher::new();
        let token = RespToken::Array(vec![
            RespToken::BulkString("ecHO".to_string()),
            RespToken::BulkString("Hello".to_string()),
        ]);
        let response = dispatcher.dispatch(&token);

        assert_eq!(response, Some(RespToken::BulkString("Hello".to_string())));
    }

    #[test]
    fn test_unknown_command() {
        let dispatcher = CommandDispatcher::new();
        let token = RespToken::BulkString("unknown".to_string());
        let response = dispatcher.dispatch(&token);

        assert_eq!(response, None);
    }
}
