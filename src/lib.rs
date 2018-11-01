//! # File Processor library
//! Assortment of functions to process and check
//! necessary files at runtime

// External crates
#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
extern crate serde_derive;
#[cfg(feature = "json")]
extern crate serde_json;

#[macro_use]
extern crate load_file;

use std::path::{PathBuf};



/// Find all files provided in `filenames`, run them through the provided function `process`
/// and then load the files provided by the function as a byte vector (binary format)
/// 
/// # Variables
///   `directory` - The directory from which to start the search
///   `filenames` - A vector of all the filenames to be searched
///   `process` - Function that takes a PathBuf to a file and returns a PathBuf to the new file created (`fn(&PathBuf)->PathBuf`)
///   `ignore_fail` - A boolean indicating if incorrect or corrupt paths should be errored on
///       If it is unset(`false`) a `file_processor::Error` will be returned
/// 
/// # Return type
///   `Result<&[u8], file_processor::Error>`
/// 
/// # Errors
///   `DirectoryDoesNotExist(std::path::PathBuf)` - The provided parent directory does not exist
///   `CouldNotOpenEntry` - There was an error while examining a directory entry (`std::fs::DirEntry`)
///   `NullDirectory` - The provided directory is null
///   `InvalidUnicodeData` - A file has invalid characters in its extension
///   `MissingFiles(Vec<usize>)` - Indicates which files from the requested ones (`filenames`) have not been found
pub fn find_and_then_and_load(directory: PathBuf, filenames: Vec<String>, process: fn(&PathBuf)->PathBuf, ignore_fail: bool) -> Result<Vec<&'static [u8]>, Error> {
	let mut count = 0;

	let mut indexes: Vec<usize> = (0..filenames.len()).collect();

	let mut binaries: Vec<_> = Vec::new();

	match directory.to_str() {
		Some(dir) => {
			if !directory.exists() {
				return Err(Error::DirectoryDoesNotExist(directory.clone()));
			}

			for entry in std::fs::read_dir(dir).unwrap() {
				match entry {
					Ok(e) => {
						match e.file_name().to_str() {
							Some(name) => {
								for (i, s) in filenames.iter().enumerate() {
									if s == &String::from(name) {
										binaries.push(load_bytes!(process(&e.path()).to_str().expect("Return path is incorrect")));
										count += 1;
										indexes.remove(i);
										break;
									}
								}
							},

							None => if !ignore_fail { return Err(Error::InvalidUnicodeData) }
						}

					},
					_ => if !ignore_fail { return Err(Error::CouldNotOpenEntry) },
				}
			}

			if count == filenames.len() {
				Ok(binaries)
			} else {
				Err(Error::MissingFiles(indexes.clone()))
			}
		},
		None => Err(Error::NullDirectory),
	}
}


/// Find all files with the extensions provided in `extensions` and run them through the provided function `process`
/// 
/// # Variables
///   `directory` - The directory from which to start the search
///   `extensions` - A vector of all the extensions to be filtered
///   `process` - Function that takes a PathBuf to a file `fn(&PathBuf)`
///   `ignore_fail` - A boolean indicating if incorrect or corrupt paths should be errored on
///       If it is unset(`false`) a `file_processor::Error` will be returned
/// 
/// # Return type
///   `Result<(), file_processor::Error>`
/// 
/// # Errors
///   `DirectoryDoesNotExist(std::path::PathBuf)` - The provided parent directory does not exist
///   `CouldNotOpenEntry` - There was an error while examining a directory entry (`std::fs::DirEntry`)
///   `NullDirectory` - The provided directory is null
pub fn find_by_extension_and_then(directory: PathBuf, extensions: Vec<String>, process: fn(&PathBuf), ignore_fail: bool) -> Result<(), Error> {
	match directory.to_str() {
		Some(dir) => {
			if !directory.exists() {
				return Err(Error::DirectoryDoesNotExist(directory));
			}

			for entry in std::fs::read_dir(dir).unwrap() {
				match entry {
					Ok(e) => {
						match e.path().extension() {
							Some(extension) => {
								match extension.to_str() {
									Some(ext) => {
										if extensions.contains(&String::from(ext)) {
											process(&e.path());
										}
									},
									None => continue,
								}
							},
							None => continue,
						}
					},
					_ => if !ignore_fail { return Err(Error::CouldNotOpenEntry) },
				}
			}

			Ok(())
		},
		None => Err(Error::NullDirectory),
	}
}


/// Find all files provided in `filenames` and run them through the provided function `process`
/// 
/// # Variables
///   `directory` - The directory from which to start the search
///   `filenames` - A vector of all the filenames to be searched
///   `process` - Function that takes a PathBuf to a file
///   `ignore_fail` - A boolean indicating if incorrect or corrupt paths should be errored on
///       If it is unset(`false`) a `file_processor::Error` will be returned
/// 
/// # Return type
///   `Result<(), file_processor::Error>`
/// 
/// # Errors
///   `DirectoryDoesNotExist(std::path::PathBuf)` - The provided parent directory does not exist
///   `CouldNotOpenEntry` - There was an error while examining a directory entry (`std::fs::DirEntry`)
///   `NullDirectory` - The provided directory is null
///   `InvalidUnicodeData` - A file has invalid characters in its name
///   `MissingFiles(Vec<usize>)` - Indicates which files from the requested ones (`filenames`) have not been found
pub fn find_and_then(directory: PathBuf, filenames: Vec<String>, process: fn(&PathBuf), ignore_fail: bool) -> Result<(), Error> {
	let mut count = 0;

	let mut indexes: Vec<usize> = (0..filenames.len()).collect();

	match directory.to_str() {
		Some(dir) => {
			if !directory.exists() {
				return Err(Error::DirectoryDoesNotExist(directory.clone()));
			}

			for entry in std::fs::read_dir(dir).unwrap() {
				match entry {
					Ok(e) => {
						match e.file_name().to_str() {
							Some(name) => {
								for (i, s) in filenames.iter().enumerate() {
									if s == &String::from(name) {
										process(&e.path());
										count += 1;
										indexes.remove(i);
										break;
									}
								}
							},

							None => if !ignore_fail { return Err(Error::InvalidUnicodeData) }
						}

					},
					_ => if !ignore_fail { return Err(Error::CouldNotOpenEntry) },
				}
			}

			if count == filenames.len() {
				Ok(())
			} else {
				Err(Error::MissingFiles(indexes.clone()))
			}
		},
		None => Err(Error::NullDirectory),
	}
}


/// A structure representing a filename and its last modification date
/// 
/// It is used to keep record of state changes
#[derive(Debug)]
pub struct FileModify {
	filename: std::path::PathBuf,
	date: std::time::SystemTime,
}

impl FileModify {
	pub fn new(filename: std::path::PathBuf, date: std::time::SystemTime) -> FileModify {
		FileModify{
			filename,
			date,
		}
	}
}

/// An enum listing all possibles file types to save `FileModify`'s into
#[derive(Debug, Copy, Clone)]
pub enum SaveFileFormat {
	JSON,
	Bincode,
	CBOR,
	YAML,
	TOML,
	MessagePack,
	RON,
}


/// Error types
///   `DirectoryDoesNotExist(std::path::PathBuf)` - The directory does not exist
///   `CouldNotOpenEntry` - There was an error while examining a directory entry (`std::fs::DirEntry`)
///   `NullDirectory` - The directory is null
///   `InvalidUnicodeData` - A file has invalid characters in its name
///   `MissingFiles(Vec<usize>)` - Indicates files index which could not be found
#[derive(Debug, Clone)]
pub enum Error {
	InvalidUnicodeData,
	NullDirectory,
	CouldNotOpenEntry,
	DirectoryDoesNotExist(PathBuf),
	MissingFiles(Vec<usize>),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Error::InvalidUnicodeData => write!(f, "Entry name contains invalid Unicode data", ),
			Error::NullDirectory => write!(f, "Null directory does not exist"),
			Error::CouldNotOpenEntry => write!(f, "Could not open entry"),
			Error::DirectoryDoesNotExist(dir) => write!(f, "Directory does not exist:\n{:?}", dir),
			Error::MissingFiles(indexes) => write!(f, "Could not find all files\nMissing files: {:?}", indexes),
		}
	}
}