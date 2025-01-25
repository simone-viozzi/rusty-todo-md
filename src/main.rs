mod git_utils;

use git_utils::{open_repository, get_staged_files};

fn main() {
    // Attempt to open the Git repository
    match open_repository() {
        Ok(repo) => {
            println!("Successfully opened the repository!");

            // Attempt to retrieve staged files
            match get_staged_files(&repo) {
                Ok(files) => {
                    if files.is_empty() {
                        println!("No staged files found.");
                    } else {
                        println!("Staged files:");
                        for file in files {
                            println!("- {:?}", file);
                        }
                    }
                }
                Err(e) => eprintln!("Error retrieving staged files: {}", e),
            }
        }
        Err(e) => eprintln!("Error opening repository: {}", e),
    }
}
