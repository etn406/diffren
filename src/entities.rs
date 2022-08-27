use std::path::PathBuf;

/// Path Change Result
pub type PathChangeResult = Result<(), PathChangeError>;

/// Path Change Error
pub enum PathChangeError {
    InputDoesntExist,
    OutputAlreadyExists,
    SeveralOutputsAreTheSame(u32),
    InvalidOutputPath,
    InvalidInputPath,
    Unchanged,
}

/// Path renaming with input and output
pub type PathChange = (PathBuf, PathBuf);
