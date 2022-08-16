use sha1::{Digest, Sha1};

/// # struct Piece
/// Represents a piece of the torrent file:
///     - idx -> piece index
///     - tl_piece_bytes -> the total amount of bytes of the piece
///     - dl_piece_bytes -> the amount of bytes already downloaded of the piece
///     - rq_piece_bytes -> the amount of bytes already requested to a peer
///     - expected_hash -> the hash we should end up with after completing download
///     - data -> the piece data (blocks)
#[derive(Debug)]
//#[derive(Clone)]
pub struct Piece {
    idx: u32,
    tl_piece_bytes: u32,
    dl_piece_bytes: u32,
    rq_piece_bytes: u32,
    expected_hash: Vec<u8>,
    data: Vec<u8>,
}

impl Piece {
    pub fn new(idx: u32, tl_piece_bytes: u32, expected_hash: Vec<u8>) -> Piece {
        Piece {
            idx,
            tl_piece_bytes,
            dl_piece_bytes: 0,
            rq_piece_bytes: 0,
            expected_hash,
            data: vec![],
        }
    }

    /// Adds a new block to the piece data
    pub fn add_block(&mut self, mut block: Vec<u8>) {
        self.data.append(&mut block)
    }

    /// Return the size of the next block
    pub fn next_block_length(&self) -> u32 {
        let block_length = 1 << 14;
        let left = self.tl_piece_bytes - self.rq_piece_bytes;
        if left < block_length {
            return left;
        }
        block_length
    }

    /// Checks if the downloaded piece is valid. To do this, it compares the hash of downloaded piece
    /// with the hash of the original piece that is in the torrent file
    pub fn piece_is_valid(&mut self) -> bool {
        //Get hash of downloaded piece
        let mut hasher = Sha1::new();
        hasher.update(self.get_data());
        let piece_hash = hasher.finalize();

        //Compare two hashes
        self.expected_hash == piece_hash.to_vec()
    }

    // Returns the piece index
    pub fn get_idx(&self) -> u32 {
        self.idx
    }

    // Returns the amount downloaded so far in bytes
    pub fn get_dl(&self) -> u32 {
        self.dl_piece_bytes
    }

    // Returns the total piece size in bytes
    pub fn get_tl(&self) -> u32 {
        self.tl_piece_bytes
    }

    // Returns the amount requested to peers so far in bytes
    pub fn get_rq(&self) -> u32 {
        self.rq_piece_bytes
    }

    /// Increases the amount downloaded
    pub fn add_to_dl(&mut self, bytes: u32) {
        self.dl_piece_bytes += bytes;
    }

    /// Increases the amount requested
    pub fn add_to_rq(&mut self, bytes: u32) {
        self.rq_piece_bytes += bytes;
    }

    /// Returns the piece data
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Returns the hash the piece should have to be valid
    pub fn get_hash(&mut self) -> Vec<u8> {
        self.expected_hash.clone()
    }

    // Deletes the data we had so far
    pub fn empty_data(&mut self) {
        self.data = vec![];
    }

    // Deletes all of the piece info
    pub fn reset_info(&mut self) {
        self.dl_piece_bytes = 0;
        self.rq_piece_bytes = 0;
        self.data = vec![];
    }
}
