use RespToken::*;

use crate::resp::RespToken;
use crate::store::Store;

pub struct CommandContext {
    pub token: RespToken,
    pub store: Store,
}

trait CommandHandler {
    fn try_handle(&self, context: &CommandContext) -> Option<RespToken>;
}

struct PingHandler;

impl PingHandler {
    fn is_ping(&self, token: &RespToken) -> bool {
        match token {
            SimpleString(s) if s.to_uppercase() == "PING" => true,
            BulkString(s) if s.to_uppercase() == "PING" => true,
            Array(elements) if elements.len() == 1 => self.is_ping(&elements[0]),
            _ => false,
        }
    }
}

impl CommandHandler for PingHandler {
    fn try_handle(&self, context: &CommandContext) -> Option<RespToken> {
        if self.is_ping(&context.token) {
            return Some(BulkString("PONG".to_string()));
        }
        None
    }
}

struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn try_handle(&self, context: &CommandContext) -> Option<RespToken> {
        if let Array(elements) = &context.token {
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

struct GetHandler;

impl CommandHandler for GetHandler {
    fn try_handle(&self, context: &CommandContext) -> Option<RespToken> {
        if let Array(elements) = &context.token {
            if elements.len() != 2 {
                return None;
            }

            if let (BulkString(command), BulkString(key)) = (&elements[0], &elements[1]) {
                if command.to_uppercase() != "GET" {
                    return None;
                }
                if let Some(value) = context.store.get(key) {
                    return Some(BulkString(value));
                }
                return Some(NullBulkString);
            }
        }
        None
    }
}

struct SetHandler;

impl CommandHandler for SetHandler {
    fn try_handle(&self, context: &CommandContext) -> Option<RespToken> {
        if let Array(elements) = &context.token {
            if elements.len() != 3 {
                return None;
            }

            if let (BulkString(command), BulkString(key), BulkString(value)) =
                (&elements[0], &elements[1], &elements[2])
            {
                if command.to_uppercase() != "SET" {
                    return None;
                }

                context.store.insert(key.to_string(), value.to_string());

                return Some(SimpleString("OK".to_string()));
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
            handlers: vec![
                Box::new(PingHandler),
                Box::new(EchoHandler),
                Box::new(GetHandler),
                Box::new(SetHandler),
            ],
        }
    }

    pub fn dispatch(&self, context: &CommandContext) -> RespToken {
        for handler in &self.handlers {
            if let Some(response) = handler.try_handle(context) {
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
        let context = CommandContext {
            token,
            store: Store::new(),
        };
        let response = dispatcher.dispatch(&context);

        assert_eq!(response, BulkString("PONG".to_string()));
    }

    #[test]
    fn test_echo() {
        let dispatcher = CommandDispatcher::new();
        let token = Array(vec![
            BulkString("ecHO".to_string()),
            BulkString("Hello".to_string()),
        ]);
        let context = CommandContext {
            token,
            store: Store::new(),
        };
        let response = dispatcher.dispatch(&context);

        assert_eq!(response, BulkString("Hello".to_string()));
    }

    #[test]
    fn test_unknown_command() {
        let dispatcher = CommandDispatcher::new();
        let token = BulkString("unknown".to_string());
        let context = CommandContext {
            token,
            store: Store::new(),
        };
        let response = dispatcher.dispatch(&context);

        assert_eq!(response, SimpleError("unknown command".to_string()));
    }

    #[test]
    fn test_get() {
        let dispatcher = CommandDispatcher::new();
        let token = Array(vec![
            BulkString("GET".to_string()),
            BulkString("key".to_string()),
        ]);
        let store = Store::new();
        store.insert("key".to_string(), "value".to_string());
        let context = CommandContext { token, store };
        let response = dispatcher.dispatch(&context);

        assert_eq!(response, BulkString("value".to_string()));
    }

    #[test]
    fn test_set() {
        let dispatcher = CommandDispatcher::new();
        let token = Array(vec![
            BulkString("SET".to_string()),
            BulkString("key".to_string()),
            BulkString("value".to_string()),
        ]);
        let store = Store::new();
        let context = CommandContext {
            token,
            store: store.clone(),
        };
        let response = dispatcher.dispatch(&context);

        assert_eq!(response, SimpleString("OK".to_string()));
        assert_eq!(store.get("key"), Some("value".to_string()));
    }
}
