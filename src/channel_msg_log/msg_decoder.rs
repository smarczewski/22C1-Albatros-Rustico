pub struct MsgDecoder {}

impl MsgDecoder {
    //takes ownership of the message. Requires to be used in conjunction
    //with the clone() function.
    pub fn decode_message(message: String) -> (String, i32) {
        let mut message_it = message.split('|');
        let message_type = message_it.next().unwrap();
        let message_content = message_it.next().unwrap();
        let times_to_be_added = if message_type == "NEW" {
            2
        } else if message_type == "FINISH" {
            -1
        } else {
            0
        };
        (message_content.to_string(), times_to_be_added)
    }
}
