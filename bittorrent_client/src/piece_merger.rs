use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
pub struct PieceMerger;

impl PieceMerger {
    pub fn merge_pieces(filename: &str, dl_dir: &str, num_pieces: u32) -> Result<(), String> {
        let merged_file_path = format!("{}/{}", dl_dir, filename);
        if Path::new(&merged_file_path).exists() {
            return Ok(());
        }

        if File::create(&merged_file_path).is_ok() {
            if let Ok(mut open_merged_file) =
                OpenOptions::new().append(true).open(&merged_file_path)
            {
                for i in 0..num_pieces {
                    let piece_filename = format!("{}_piece_{}", &merged_file_path, i);
                    if let Ok(piece_read) = fs::read(&piece_filename) {
                        if open_merged_file.write(&piece_read).is_err() {
                            match fs::remove_file(&merged_file_path){
                                Ok(_a) => return Err("Error: when combining files".to_string()),
                                Err(_error) => return Err("Error: could not combine files. The halfway combined file could not be deleted".to_string())
                            };
                        }
                    }
                    if fs::remove_file(piece_filename).is_err() {
                        return Err("File deletion failed".to_string());
                    }
                }
                return Ok(());
            }
        }
        Err("Error: Cannot merge the pieces".to_string())
    }
}
