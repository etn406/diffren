use std::path::PathBuf;

/// Path renaming with current and target paths
pub type Renaming = (PathBuf, PathBuf);

/// Path renaming validation
pub type Validation = Result<(), ValidationError>;

/// Renaming validation error,
pub enum ValidationError {
    FileDoesntExist,
    TargetAlreadyExists,
    SeveralTargetsAreTheSame(u32),
    InvalidTargetPath,
    InvalidCurrentPath,
    Unchanged,
}

/// Verify that each renaming is possible, or add a detailed error.
/// Also counts the non-blocking errors (only a unchanged path is counted as a non-blocking error)
/// Returns a tuple containing the renamings, the changes count and then the errors count.
pub fn validate_renamings(renamings: Vec<Renaming>) -> (Vec<(Renaming, Validation)>, u32, u32) {
    let mut errors = 0;
    let mut changes = 0;

    let renamings: Vec<(Renaming, Validation)> = renamings
        .iter()
        .map(|change| {
            let result = validate_renaming(&change, &renamings);

            match result {
                Ok(_) => changes += 1,
                Err(ValidationError::Unchanged) => (),
                Err(_) => errors += 1,
            }

            ((change.0.clone(), change.1.clone()), result)
        })
        .collect();

    (renamings, changes, errors)
}

/// Verify the validity of a renaming.
fn validate_renaming((current, target): &Renaming, renamings: &Vec<Renaming>) -> Validation {
    if current.eq(target) {
        Err(ValidationError::Unchanged)
    } else if !current.exists() {
        Err(ValidationError::FileDoesntExist)
    } else if current.to_str().unwrap_or("").is_empty() {
        Err(ValidationError::InvalidCurrentPath)
    } else if target.exists() {
        Err(ValidationError::TargetAlreadyExists)
    } else if target.to_str().unwrap_or("").is_empty() {
        Err(ValidationError::InvalidTargetPath)
    } else if let Err(count) = check_uniqueness_of_target_path(target, &renamings) {
        Err(ValidationError::SeveralTargetsAreTheSame(count))
    } else {
        Ok(())
    }
}

/// Verify that the `target` path is unique in the renamings,
/// otherwise returns the number of occurences.
fn check_uniqueness_of_target_path(target: &PathBuf, renamings: &Vec<Renaming>) -> Result<(), u32> {
    let mut count = 0;

    for (_, target_2) in renamings {
        if target.eq(target_2) {
            count += 1;
        }
    }

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
        return Err("The two files do not have the same number of lines.".to_string());
    }

    Ok(Vec::from_iter(current.into_iter().zip(target.into_iter())))
}

/// Filter changes and only keeps the ones with an Ok result.
pub fn keep_valid_renamings(renamings: Vec<(Renaming, Validation)>) -> Vec<Renaming> {
    renamings
        .into_iter()
        .filter_map(|(change, result)| match result {
            Ok(_) => Some(change),
            _ => None,
        })
        .collect()
}
