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
                    if open_merged_file
                        .write(&fs::read(&piece_filename).unwrap())
                        .is_err()
                    {
                        match fs::remove_file(&merged_file_path){
							Ok(_a) => return Err("Error: when combining files".to_string()),
							Err(_error) => return Err("Error: could not combine files. The halfway combined file could not be deleted".to_string())
						};
                    } else {
                        fs::remove_file(piece_filename).expect("File deletion failed");
                    }
                }
            } else {
                return Err(
                    "Error: could not open the file in which to write the pieces/chunks of files"
                        .to_string(),
                );
            }
        } else {
            return Err("Error: File could not be created".to_string());
        }
        Ok(())
    }
}
