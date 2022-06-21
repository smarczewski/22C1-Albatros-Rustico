pub struct MsgDecoder {}

impl MsgDecoder {
    //takes ownership of the message. Requires to be used in conjunction
    //with the clone() function.
    pub fn decypher_message(message: String) -> (String, bool, String, String) {
        let mut message_it = message.split('|');
        let message_type = message_it.next().unwrap();
        let sender_type = message_it.next().unwrap();
        let message_content = message_it.next().unwrap();
        if message_type == "END" {
            (
                sender_type.to_string(),
                false,
                message_content.to_string(),
                message_type.to_string(),
            )
        } else {
            (
                sender_type.to_string(),
                true,
                message_content.to_string(),
                message_type.to_string(),
            )
        }
    }

    pub fn get_log_level(message: String) -> String {
        let mut message_it = message.split('|');
        let message_type = message_it.next().unwrap();
        if message_type == "INCOMPLETE" {
            "ERROR".to_string()
        } else {
            "INFO".to_string()
        }
    }
}
