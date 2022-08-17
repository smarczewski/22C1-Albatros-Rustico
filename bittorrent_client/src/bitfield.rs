use crate::constants::BYTE_FILLED_W_ONES;

/// # struct Piece Bitfield
/// It's a bitfield where each bit represent a piece.
///     - 1 -> piece has been already downloaded
///     - 0 -> piece has not been downloaded yet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieceBitfield {
    bitfield: Vec<u8>,
    n_pieces: u32,
}

impl PieceBitfield {
    /// Creates an empty Piece Bitfield
    pub fn new(n_pieces: u32) -> PieceBitfield {
        let bitfield = vec![0u8; (n_pieces as f32 / 8.0).ceil() as usize];
        PieceBitfield { bitfield, n_pieces }
    }

    /// Receives a Vec<u8> and creates a Piece Bitfield using it.
    pub fn new_from_vec(bitfield: Vec<u8>, n_pieces: u32) -> PieceBitfield {
        PieceBitfield { bitfield, n_pieces }
    }

    /// Receives a piece index and mark this one with a 1 in the peer's vector pieces
    pub fn add_a_piece(&mut self, piece_idx: u32) {
        let n_shift = 7 - (piece_idx % 8);
        let mask: u8 = 1 << n_shift;
        let idx: usize = (piece_idx / 8) as usize;
        if idx < self.bitfield.len() {
            self.bitfield[idx] |= mask;
        }
    }

    /// Receives a vector with pieces and adds them to the current bitfield.
    pub fn add_multiple_pieces(&mut self, pieces: Vec<u8>) {
        for (i, item) in pieces.iter().enumerate().take(self.bitfield.len()) {
            self.bitfield[i] |= item;
        }
    }

    /// Check if the current bitfield has a specific piece.
    pub fn has_piece(&self, piece_idx: u32) -> bool {
        let n_shift = 7 - (piece_idx % 8);
        let mut mask: u8 = 1 << n_shift;
        let idx: usize = (piece_idx / 8) as usize;
        if idx < self.bitfield.len() {
            mask &= self.bitfield[idx];
            if mask != 0 {
                return true;
            }
        }
        false
    }

    /// Returns a bitfield with all pieces marked as 1
    pub fn get_completed_bitfield(n_pieces: u32) -> PieceBitfield {
        let mut bitfield = vec![];
        let bitfield_len = ((n_pieces as f32 / 8.0).ceil()) as usize;
        if n_pieces > 8 {
            for _i in 0..bitfield_len - 1 {
                bitfield.push(BYTE_FILLED_W_ONES);
            }
        }

        let mut last_byte = 0;
        if n_pieces % 8 == 0 {
            last_byte = BYTE_FILLED_W_ONES;
        } else {
            for j in 0..(n_pieces % 8) {
                last_byte |= 1 << (7 - j);
            }
        }
        bitfield.push(last_byte);

        PieceBitfield::new_from_vec(bitfield, n_pieces)
    }

    /// Check if the bitfield has all the pieces.
    pub fn has_all_pieces(&self) -> bool {
        self.bitfield == PieceBitfield::get_completed_bitfield(self.n_pieces).bitfield
    }

    /// Receives another bitfield and checks if there is any match between the two ones.
    pub fn there_is_match(&self, comp_bf: &PieceBitfield) -> bool {
        for i in 0..self.bitfield.len() {
            let curr_result = comp_bf.bitfield[i] & self.bitfield[i];
            if curr_result != 0 {
                return true;
            }
        }
        false
    }

    /// Returns the complement.
    /// All bytes in 0 are changed to 1 and vice versa
    pub fn get_complement(&self) -> PieceBitfield {
        let mut c_bitfield = self.clone();
        for i in 0..c_bitfield.bitfield.len() {
            c_bitfield.bitfield[i] = !c_bitfield.bitfield[i];
        }
        c_bitfield
    }

    pub fn get_vec(&self) -> Vec<u8> {
        self.bitfield.clone()
    }

    pub fn number_of_downloaded_pieces(&self) -> u32 {
        let mut piece_counter: u32 = 0;
        for piece_idx in 0..self.n_pieces {
            if self.has_piece(piece_idx) {
                piece_counter += 1;
            }
        }
        piece_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn adding_one_piece_in_bitfield() {
        let mut bitfield = PieceBitfield::new(39);
        let expected_bitfield = vec![128, 0, 0, 0, 0];
        bitfield.add_a_piece(0);
        assert_eq!(bitfield.bitfield, expected_bitfield);
    }

    #[test]
    fn adding_multiple_pieces_in_bitfield() {
        let mut bitfield = PieceBitfield::new(39);
        let expected_bitfield = vec![65, 128, 0, 0, 0];
        bitfield.add_a_piece(1);
        bitfield.add_a_piece(7);
        bitfield.add_a_piece(8);
        assert_eq!(bitfield.bitfield, expected_bitfield);
    }

    #[test]
    fn adding_invalid_piece_in_bitfield() {
        let mut bitfield = PieceBitfield::new(40);
        let expected_bitfield = vec![0, 0, 0, 0, 0];
        bitfield.add_a_piece(100);
        assert_eq!(bitfield.bitfield, expected_bitfield);
    }

    #[test]
    fn bitfield_has_piece() {
        let mut bitfield = PieceBitfield::new(20);
        bitfield.add_a_piece(10);
        assert!(bitfield.has_piece(10));
    }

    #[test]
    fn bitfield_has_not_piece() {
        let bitfield = PieceBitfield::new(20);
        assert!(!bitfield.has_piece(10));
    }

    #[test]
    fn completed_bitfield() {
        let bitfield = PieceBitfield::get_completed_bitfield(5);
        assert_eq!(bitfield.bitfield, vec![0xF8]);
    }

    #[test]
    fn bitfield_has_all_pieces() {
        let bitfield = PieceBitfield::get_completed_bitfield(5);
        assert!(bitfield.has_all_pieces());
    }

    #[test]
    fn bitfield_there_is_match() {
        let mut wanted_pieces = PieceBitfield::new(10);
        let mut peer_pieces = PieceBitfield::new(10);

        wanted_pieces.add_a_piece(1);
        wanted_pieces.add_a_piece(7);
        peer_pieces.add_a_piece(7);

        assert!(peer_pieces.there_is_match(&wanted_pieces));
    }

    #[test]
    fn bitfield_get_complement() {
        let mut bitfield = PieceBitfield::new(10);
        bitfield.add_a_piece(10);

        assert_eq!(bitfield.get_complement().bitfield, vec![0xFF, 0xDF]);
    }

    #[test]
    fn bitfield_count_pieces() {
        let bitfield1 = PieceBitfield::new(100);
        assert_eq!(bitfield1.number_of_downloaded_pieces(), 0);

        let mut bitfield2 = PieceBitfield::new(100);
        bitfield2.add_a_piece(10);
        assert_eq!(bitfield2.number_of_downloaded_pieces(), 1);

        let mut bitfield3 = PieceBitfield::new(100);
        for i in 0..10 {
            bitfield3.add_a_piece(i);
        }
        assert_eq!(bitfield3.number_of_downloaded_pieces(), 10);

        let completed_bf_1 = PieceBitfield::get_completed_bitfield(100);
        assert_eq!(completed_bf_1.number_of_downloaded_pieces(), 100);

        let completed_bf_2 = PieceBitfield::get_completed_bitfield(80);
        assert_eq!(completed_bf_2.number_of_downloaded_pieces(), 80);
    }
}
