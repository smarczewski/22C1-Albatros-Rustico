pub struct MsgCoder {}

impl MsgCoder {
    pub fn generate_new_connection_message(message: String) -> String {
        format!("NEW|{}", message)
    }
    pub fn generate_download_complete_message(message: String) -> String {
        format!("COMPLETE|{}", message)
    }

    pub fn generate_download_not_complete_message(message: String) -> String {
        format!("INCOMPLETE|{}", message)
    }
    pub fn generate_file_completely_downloaded_message(message: String) -> String {
        format!("FINISH|{}", message)
    }
}
