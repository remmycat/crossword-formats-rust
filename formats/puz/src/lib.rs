use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsePuzError {
	#[error("error variants wip")]
	ToDo,
}

#[derive(Debug)]
pub enum PuzzleType {
	Normal,
	Diagramless,
}

#[derive(Debug)]
pub enum SolutionState {
	Unlocked,
	Scrambled,
}

#[derive(Debug)]
pub struct PuzFileHeader {
	// pub preamble: Vec<u8>,
	// pub checksum_overall: u16,
	// pub checksum_cib: u16,
	// pub masked_checksums: [u8; 8],
	// pub version: String,
	// pub unknown_data_1: [u8; 2],
	// pub scrambled_checksum: u16,
	// pub unknown_data_2: [u8; 12],
	// pub width: u8,
	// pub height: u8,
	// pub clue_count: u16,
	// pub puzzle_type: PuzzleType,
	// pub solution_state: SolutionState,
}

#[derive(Debug)]
pub struct PuzFile {
	pub header: PuzFileHeader,
}

pub fn parse_a_puz(_puz_bytes: &[u8]) -> Result<PuzFile, ParsePuzError> {
	Ok(PuzFile {
		header: PuzFileHeader {},
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_parses_unicode_puz() {
		// This will break compilation if you don't have an example file there, sorry!
		// I will provide some in the future.
		let unicode_puzzle = include_bytes!("../fixtures/unicode.puz");

		let parsed = parse_a_puz(unicode_puzzle).expect("Parsing Failed");

		dbg!(parsed);
	}
}
