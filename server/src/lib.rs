//! Simple Chat Protocol Library
//!
//! This crate defines the message protocol used for communication between
//! the chat server and clients. It provides a serializable `ClientMessage`
//! enum that represents all possible client-to-server operations.
//!
//! # Message Format
//! All messages are serialized to JSON format and transmitted over TCP
//! connections with newline delimiters.
//!
//! # Usage
//! Both the server and client binaries depend on this library to ensure
//! consistent message format across the system.
//!
//! # Examples
//! ```
//! use server::ClientMessage;
//! use serde_json;
//!
//! let connect_msg = ClientMessage::Connect {
//!     username: "alice".to_string(),
//! };
//! let json = serde_json::to_string(&connect_msg).unwrap();
//! assert_eq!(json, r#"{"Connect":{"username":"alice"}}"#);
//! ```

use serde::{Deserialize, Serialize};

/// Message types that clients can send to the chat server.
///
/// All messages are serialized to JSON format for transmission over TCP connections.
/// The server expects each message to be sent as a JSON string followed by a newline.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ClientMessage {
    /// Request to connect to the chat server with a specified username.
    ///
    /// # Fields
    /// - `username`: Unique identifier for the client in the chat room.
    ///
    /// # Server Behavior
    /// - If the username is already taken, the connection should be rejected.
    /// - If successful, the client is added to the list of active users.
    Connect { username: String },

    /// Request to send a message to the chat room.
    ///
    /// # Fields
    /// - `username`: The sender's username (must match the connected username).
    /// - `message`: The text content to broadcast to all other connected users.
    ///
    /// # Server Behavior
    /// - Verifies the username matches the connection.
    /// - Broadcasts the message to all other connected clients.
    /// - Does not send the message back to the original sender.
    SendMessage { username: String, message: String },

    /// Request to leave the chat server and disconnect.
    ///
    /// # Fields
    /// - `username`: The username of the client leaving.
    ///
    /// # Server Behavior
    /// - Removes the user from the list of active users.
    /// - Stops sending messages to this client.
    /// - Cleans up associated resources.
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
