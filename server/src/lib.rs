use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ClientMessage {
    Connect { username: String },
    SendMessage { username: String, message: String },
    Leave { username: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_message_serialization() {
        let msg = ClientMessage::Connect {
            username: "alice".to_string(),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let expected = r#"{"Connect":{"username":"alice"}}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_connect_message_deserialization() {
        let data = r#"{"Connect":{"username":"alice"}}"#;
        let msg: ClientMessage = serde_json::from_str(data).unwrap();
        let expected = ClientMessage::Connect {
            username: "alice".to_string(),
        };
        assert_eq!(msg, expected);
    }

    #[test]
    fn test_send_message_serialization() {
        let msg = ClientMessage::SendMessage {
            username: "alice".to_string(),
            message: "hello world".to_string(),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let expected = r#"{"SendMessage":{"username":"alice","message":"hello world"}}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_leave_message_serialization() {
        let msg = ClientMessage::Leave {
            username: "alice".to_string(),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let expected = r#"{"Leave":{"username":"alice"}}"#;
        assert_eq!(serialized, expected);
    }
}