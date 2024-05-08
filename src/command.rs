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
