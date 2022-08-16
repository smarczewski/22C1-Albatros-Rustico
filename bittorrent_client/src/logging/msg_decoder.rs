pub struct MsgDecoder {}

impl MsgDecoder {
    //takes ownership of the message. Requires to be used in conjunction
    //with the clone() function.
    pub fn decypher_message(message: String) -> Result<(String, bool, String, String), String> {
        let mut message_it = message.split('|');
        let message_type = message_it.next();
        let sender_type = message_it.next();
        let message_content = message_it.next();
        if let (Some(m_type), Some(s_type), Some(m_content)) =
            (message_type, sender_type, message_content)
        {
            if m_type == "END" {
                return Ok((
                    s_type.to_string(),
                    false,
                    m_content.to_string(),
                    m_type.to_string(),
                ));
            } else {
                return Ok((
                    s_type.to_string(),
                    true,
                    m_content.to_string(),
                    m_type.to_string(),
                ));
            }
        }
        Err("Error: Cannot decypher message".to_string())
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
