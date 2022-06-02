use std::{fs};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
pub struct PieceMerger{

}

impl Default for PieceMerger{
	fn default() -> Self{
		Self::new()
	}
}

impl PieceMerger{

	pub fn new()->Self{
		PieceMerger{}
	}

	//Finds the pieces related to a filename
	//format of the pieces must be: filename_pieceNumber
	pub fn find_pieces(&self, filename: String, num_pieces: i32) -> Result<String,String>{
		let mut pieces_found = 0;
		for i in 1..num_pieces+1{
			let piece_filename = format!("{}_{}",filename, i);
			for entry in fs::read_dir(".").unwrap(){
            	let dir = entry.unwrap();
            	let filename_found = dir.path().clone();
            	let filename_found = filename_found.to_str()
               		                               .unwrap()
                	                               .split("./")
               		                               .nth(1)
                                               	   .unwrap();
            	if piece_filename == filename_found{
                	pieces_found +=1;
            	}
        	}	
		}
		if pieces_found == num_pieces{
        	Ok("Found all pieces for the file".to_string())
        } else{
        	Err("Missing pieces for the file".to_string())
        }
	}

	pub fn merge_pieces(&self, filename:String, num_pieces: i32) -> Result<String, String>{
		if File::create(&filename).is_ok(){
			if let Ok(mut open_merged_file) = OpenOptions::new().append(true).open(&filename){
				for i in 1..num_pieces+1{
					let piece_filename = format!("{}_{}",filename,i);
					if open_merged_file.write(&fs::read(piece_filename).unwrap()).is_err(){
						match fs::remove_file(filename){
							Ok(_a) => return Err("Error: when combining files".to_string()),
							Err(_error) => return Err("Error: could not combine files. The halfway combined file could not be deleted".to_string())
						};
					} else{

					}
				}
			} else{
				return Err("Error: could not open the file in which to write the pieces/chunks of files".to_string())
			}
		} else{
			return Err("Error: File could not be created".to_string())
		}
		Ok("Combined file successfully created".to_string())
	}

	pub fn delete_pieces(&self, filename: String, num_pieces: i32) {
		for i in 1..num_pieces+1{
			let piece = format!{"{}_{}",filename,i};
			fs::remove_file(piece).expect("File deletion failed");
		}
	}

}




#[cfg(test)]
mod tests{

	use super::*;

	#[test]
	fn test_piece_merger_succesfully_created(){
		let _merger = PieceMerger::new();
	}

	#[test]
	fn test_piece_merger_finds_all_piece_from_a_given_file(){
		let file_name = "filename".to_string();
		let number_of_pieces = 1;
		let merger = PieceMerger::new();
		assert!(Result::is_ok(&merger.find_pieces(file_name, number_of_pieces)));
	}


	#[test]
	fn test_piece_merger_merges_n_files_successfully(){
		let file_name = "test_image_chunk".to_string();
		let number_of_pieces = 3;
		let merger = PieceMerger::new();
		if merger.find_pieces(file_name.clone(),number_of_pieces).is_ok(){
			merger.merge_pieces(file_name, number_of_pieces);
		} else {
			println!("No funciono");
		}
	}

}