use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsePuzError {
	#[error("this does not seem to be a .puz file - could not find beginning of puz data")]
	NotAPuz,
	#[error(
		"assumed version format is '0.0' (and a probably-null byte) - found these bytes instead: 0x{0:02x}{1:02x}{2:02x}{3:02x}"
	)]
	UnexpectedVersionFormat(u8, u8, u8, u8),
	#[error("unknown puzzle type: 0x{0:04x}")]
	UnknownPuzzleType(u16),
	#[error("unknown solution type: 0x{0:04x}")]
	UnknownSolutionType(u16),
	#[error("the puz file seems malformed or corrupted, could not find expected data")]
	Malformed(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum PuzzleType {
	Normal,
	Diagramless,
}
impl TryFrom<u16> for PuzzleType {
	type Error = ParsePuzError;
	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			0x0001 => Ok(Self::Normal),
			0x0401 => Ok(Self::Diagramless),
			other => Err(ParsePuzError::UnknownPuzzleType(other)),
		}
	}
}

#[derive(Debug)]
pub enum SolutionType {
	Normal,
	Scrambled,
	Missing,
}
impl TryFrom<u16> for SolutionType {
	type Error = ParsePuzError;
	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			0x0000 => Ok(Self::Normal),
			0x0002 => Ok(Self::Missing),
			0x0004 => Ok(Self::Scrambled),
			other => Err(ParsePuzError::UnknownSolutionType(other)),
		}
	}
}

#[derive(Debug)]
pub struct Crc16Checksum(u16);

impl From<u16> for Crc16Checksum {
	fn from(value: u16) -> Self {
		Crc16Checksum(value)
	}
}

#[derive(Debug)]
pub struct PuzVersion {
	/// first number of version tuple
	pub major: u8,
	/// second number of version tuple
	pub minor: u8,
	/// The last byte of the version was reported to sometimes contain other
	/// data instead of a 0x00 byte, e.g. a 'c'
	pub extension: Option<char>,
}
impl TryFrom<[u8; 4]> for PuzVersion {
	type Error = ParsePuzError;

	fn try_from([major, dot, minor, ext]: [u8; 4]) -> Result<Self, Self::Error> {
		if major.is_ascii_digit() && dot == b'.' && minor.is_ascii_digit() {
			Ok(Self {
				major: major - b'0', // from ascii
				minor: minor - b'0', // from ascii
				extension: if ext == 0 { None } else { Some(ext as char) },
			})
		} else {
			Err(ParsePuzError::UnexpectedVersionFormat(
				major, dot, minor, ext,
			))
		}
	}
}

/// Data of unknown use, likely just garbage
#[derive(Debug)]
pub struct PuzGarbage {
	/// There can be additional / unused data at the start of a puz file.
	/// If some is found, it will be saved here, so it can be re-added when
	/// saving the file (in case it has importance).
	pub preamble: Option<Vec<u8>>,

	/// 2 bytes of unknown use.
	/// Sometimes seems to be uninitialized data / random bits of strings
	pub unknown_header_data_1: [u8; 2],

	/// 12 bytes of unknown use.
	/// Sometimes seems to be uninitialized data / random bits of strings
	pub unknown_header_data_2: [u8; 12],
}

#[derive(Debug)]
pub struct PuzFile {
	pub garbage: PuzGarbage,

	/// overall file checksum
	pub checksum: Crc16Checksum,

	/// checksum of metadata fields
	pub checksum_board_configuration: Crc16Checksum,

	pub masked_checksums: [u8; 8],

	pub version: PuzVersion,

	/// Checksum of scrambled solution, (if scrambled)
	/// todo: put in data type of puzzle state
	pub checksum_scrambled: Option<Crc16Checksum>,

	/// Width of the diagram in squares
	pub width: u8,

	// Height of the diagram in squares
	pub height: u8,

	// Number of clues
	pub clue_count: u16,

	// Puzzle Type
	pub puzzle_type: PuzzleType,

	// Solution Type
	pub solution_type: SolutionType,
}

/// NUL-terminated constant string indicating start of file
const FILE_MAGIC: &[u8; 12] = b"ACROSS&DOWN\0";

fn get_puz_start_offset(puz_bytes: &[u8]) -> Result<usize, ParsePuzError> {
	for i in 0_usize.. {
		let sorry_sir_is_this_magic = puz_bytes
			.get((i + 2)..(i + 14))
			.ok_or(ParsePuzError::NotAPuz)?;

		if sorry_sir_is_this_magic == FILE_MAGIC {
			return Ok(i);
		}
	}

	unreachable!();
}

pub fn parse_a_puz(puz_bytes: &[u8]) -> Result<PuzFile, ParsePuzError> {
	let start_offset = get_puz_start_offset(puz_bytes)?;

	let preamble = if start_offset > 0 {
		Some(Vec::from(&puz_bytes[0..start_offset]))
	} else {
		None
	};

	let mut reader = Cursor::new(&puz_bytes[start_offset..]);

	let checksum: Crc16Checksum = reader.read_u16::<LittleEndian>()?.into();

	reader.seek(SeekFrom::Current(12))?;

	let checksum_board_configuration: Crc16Checksum = reader.read_u16::<LittleEndian>()?.into();

	let mut masked_checksums = [0_u8; 8];
	reader.read_exact(&mut masked_checksums)?;

	let mut version_bytes = [0_u8; 4];
	reader.read_exact(&mut version_bytes)?;
	let version = version_bytes.try_into()?;

	let mut unknown_header_data_1 = [0_u8; 2];
	reader.read_exact(&mut unknown_header_data_1)?;

	let checksum_scrambled_raw = reader.read_u16::<LittleEndian>()?;
	let checksum_scrambled = if checksum_scrambled_raw == 0 {
		None
	} else {
		Some(checksum_scrambled_raw.into())
	};

	let mut unknown_header_data_2 = [0_u8; 12];
	reader.read_exact(&mut unknown_header_data_2)?;

	let width = reader.read_u8()?;
	let height = reader.read_u8()?;
	let clue_count = reader.read_u16::<LittleEndian>()?;

	let puzzle_type = reader.read_u16::<LittleEndian>()?.try_into()?;
	let solution_type = reader.read_u16::<LittleEndian>()?.try_into()?;

	// for strings the reader interface seems less helpful
	let rest = &puz_bytes[(reader.position() as usize)..];

	Ok(PuzFile {
		garbage: PuzGarbage {
			preamble,
			unknown_header_data_1,
			unknown_header_data_2,
		},
		checksum,
		checksum_board_configuration,
		masked_checksums,
		version,
		checksum_scrambled,
		width,
		height,
		clue_count,
		puzzle_type,
		solution_type,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_parses_unicode_puz() {
		// This will break compilation if you don't have an example file there, sorry!
		// I will provide some in the future.
		let unicode_puzzle = include_bytes!("../fixtures/test-no-solution.puz");

		let parsed = parse_a_puz(unicode_puzzle).expect("Parsing Failed");

		dbg!(parsed);
	}
}
