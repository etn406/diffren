use std::path::PathBuf;

/// Path renaming with current and target paths
pub type Renaming = (PathBuf, PathBuf);

/// Path renaming validation
pub type Validation = Result<Renaming, ValidationError>;

/// Renaming validation error,
pub enum ValidationError {
    FileDoesntExist(Renaming),
    TargetAlreadyExists(Renaming),
    SeveralTargetsAreTheSame(Renaming, u32),
    InvalidTargetPath(Renaming),
    InvalidCurrentPath(Renaming),
    Unchanged(Renaming),
}

/// Verify that each renaming is possible, or add a detailed error.
/// Also counts the non-blocking errors (only a unchanged path is counted as a non-blocking error)
/// Returns a tuple containing the renamings, the changes count and then the errors count.
pub fn validate_renamings(renamings: Vec<Renaming>) -> (Vec<Validation>, u32, u32) {
    let mut errors = 0;
    let mut changes = 0;

    let renamings_copy = renamings.clone();

    let renamings: Vec<Validation> = renamings
        .into_iter()
        .map(|renaming| {
            let validation = validate_renaming(renaming, &renamings_copy);

            match validation {
                Ok(_) => changes += 1,
                Err(ValidationError::Unchanged(_)) => (),
                Err(_) => errors += 1,
            }

            validation
        })
        .collect();

    (renamings, changes, errors)
}

/// Verify the validity of a renaming.
fn validate_renaming(renaming: Renaming, renamings: &Vec<Renaming>) -> Validation {
    let (current, target) = &renaming;

    if current.eq(target) {
        Err(ValidationError::Unchanged(renaming))
    } else if !current.exists() {
        Err(ValidationError::FileDoesntExist(renaming))
    } else if current.to_str().unwrap_or("").is_empty() {
        Err(ValidationError::InvalidCurrentPath(renaming))
    } else if target.exists() {
        Err(ValidationError::TargetAlreadyExists(renaming))
    } else if target.to_str().unwrap_or("").is_empty() {
        Err(ValidationError::InvalidTargetPath(renaming))
    } else if let Err(count) = validate_target_uniqueness(target, &renamings) {
        Err(ValidationError::SeveralTargetsAreTheSame(renaming, count))
    } else {
        Ok(renaming)
    }
}

fn validate_target_uniqueness(target: &PathBuf, renamings: &Vec<Renaming>) -> Result<(), u32> {
    let count = renamings
        .iter()
        .filter(|(_, target_2)| target == target_2)
        .count() as u32;

    if count > 1 {
        Err(count)
    } else {
        Ok(())
    }
}

/// "Zip" the two files (current and target) contents together,
/// and fails if the two files don't have the same number of lines.
pub fn combine_paths_vecs(
    current: Vec<PathBuf>,
    target: Vec<PathBuf>,
) -> Result<Vec<Renaming>, String> {
    if current.len() != target.len() {
        return Err("The two files don't have the same number of lines.".to_string());
    }

    Ok(Vec::from_iter(current.into_iter().zip(target.into_iter())))
}

/// Filter changes and only keeps the ones with an Ok result.
pub fn keep_valid_renamings(renamings: Vec<Validation>) -> Vec<Renaming> {
    renamings
        .into_iter()
        .filter_map(|validation| validation.ok())
        .collect()
}
