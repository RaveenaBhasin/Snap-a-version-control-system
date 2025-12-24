use std::{collections::HashMap, fmt::format, fs, time::{SystemTime, UNIX_EPOCH}};
use sha256::digest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum TreeEntry {
    File { name: String, blob_hash: String}, 
    Directory { name: String, tree_hash: String}
}

#[derive(Serialize, Deserialize, PartialEq)]
struct Tree {
    entries: Vec<TreeEntry>,
}
#[derive(Serialize, Deserialize, PartialEq)]
struct Commit {
    tree_hash: String,
    parent: String,
    timestamp: i64,
    message: String,
}

fn save_blob(content: String) -> String{
    let hash = digest(&content);
    fs::write(format!(".snap/objects/{}", hash), &content).unwrap();
    return hash;
}

fn save_tree(tree: Tree) -> String {
    let json = serde_json::to_string(&tree).unwrap();
    let tree_hash = digest(&json);
    fs::write(format!(".snap/objects/{}", tree_hash), json).unwrap();
    return tree_hash;   
}


fn save_commit(commit: Commit) {
    let json = serde_json::to_string(&commit).unwrap();
    let commit_hash = digest(&json);
    fs::write(format!(".snap/objects/{}", &commit_hash), json).unwrap();

    // Update branch ref if HEAD is pointing to a branch, otherwise update HEAD directly
    let head_content = fs::read_to_string(".snap/HEAD").unwrap();
    let head_content = head_content.trim();

    if head_content.starts_with("ref: ") {
        // HEAD is pointing to a branch, update the branch ref
        let branch_path = head_content.strip_prefix("ref: ").unwrap();
        fs::write(format!(".snap/{}", branch_path), &commit_hash).unwrap();
    } else {
        // HEAD is detached (pointing directly to a commit), update HEAD
        fs::write(".snap/HEAD", &commit_hash).unwrap();
    }
}

fn build_tree(dir: &str) -> String {
    let mut entries = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let blob_hash = save_blob(content);
            entries.push(TreeEntry::File { name: file_name, blob_hash});
        }
        else if path.is_dir() {
            let dir_path = path.to_str().unwrap().to_string();
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let tree_hash = build_tree(&dir_path);
            entries.push(TreeEntry::Directory { name: file_name, tree_hash })
        }
    }
    let tree = Tree {entries};
    save_tree(tree)
}

fn get_last_commit() -> String {
    let head = fs::read_to_string(".snap/HEAD").unwrap();
    let head = head.trim();

    // Check if HEAD is a symbolic ref (pointing to a branch)
    if head.starts_with("ref: ") {
        let branch_path = head.strip_prefix("ref: ").unwrap();
        if let Ok(commit_hash) = fs::read_to_string(format!(".snap/{}", branch_path)) {
            return commit_hash.trim().to_string();
        }
    }

    return head.to_string();
}

// fn log() {
//     let mut current = get_last_commit();
    
//     while !current.is_empty() {
//         let commit_hash = current.clone();
//         let commit_data = match fs::read_to_string(format!(".snap/objects/{}", commit_hash)) {
//             Ok(data) => data,
//             Err(_) => break,
//         };
//         let commit: Commit = match serde_json::from_str(&commit_data) {
//             Ok(commit) => commit,
//             Err(_) => break,
//         };
        
//         println!("commit {}", commit_hash);
//         println!("Message: {}", commit.message);
//         println!("Timestamp: {}", commit.timestamp);
//         println!("Files: {:?}", commit.files.keys());
//         println!();
        
//         current = commit.parent;
//     }
// }

fn read_directory(dir: &str, files: &mut HashMap<String, String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();
            let blob_hash = save_blob(content);
            let file_name = path.to_str().unwrap().to_string();
            files.insert(file_name, blob_hash);
        }
        else if path.is_dir() {
            read_directory(&path.to_str().unwrap(), files);
        }
    }
}

fn diff_fn(tree_hash_1: String, tree_hash_2: String) {
    compare_trees(tree_hash_1, tree_hash_2);
}

fn compare_trees(tree_hash_1: String, tree_hash_2: String) {
    compare_trees_recursive(tree_hash_1, tree_hash_2, "");
}

fn show_file_diff(old_hash: &str, new_hash: &str, file_path: &str) {
    let old_content = match fs::read_to_string(format!(".snap/objects/{}", old_hash)) {
        Ok(content) => content,
        Err(_) => {
            println!("  Error: Could not read old version");
            return;
        }
    };

    let new_content = match fs::read_to_string(format!(".snap/objects/{}", new_hash)) {
        Ok(content) => content,
        Err(_) => {
            println!("  Error: Could not read new version");
            return;
        }
    };

    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    println!("  --- old/{}", file_path);
    println!("  +++ new/{}", file_path);

    let mut i = 0;
    let mut j = 0;

    while i < old_lines.len() || j < new_lines.len() {
        if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            // Lines are the same
            println!("   {}", old_lines[i]);
            i += 1;
            j += 1;
        } else if i < old_lines.len() && (j >= new_lines.len() || !new_lines[j..].contains(&old_lines[i])) {
            // Line was removed
            println!("  \x1b[31m - {}\x1b[0m", old_lines[i]);
            i += 1;
        } else if j < new_lines.len() {
            // Line was added
            println!("  \x1b[32m + {}\x1b[0m", new_lines[j]);
            j += 1;
        } else {
            break;
        }
    }
    println!();
}

fn compare_trees_recursive(tree_hash_1: String, tree_hash_2: String, path_prefix: &str) {
    let tree_data_1 = fs::read_to_string(format!(".snap/objects/{}", tree_hash_1)).unwrap();
    let tree_1 : Tree = serde_json::from_str(&tree_data_1).unwrap();
    let tree_data_2 = fs::read_to_string(format!(".snap/objects/{}", tree_hash_2)).unwrap();
    let tree_2 : Tree = serde_json::from_str(&tree_data_2).unwrap();

    let full_path = |name: &String| -> String {
        if path_prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", path_prefix, name)
        }
    };

    // get removed entries only in tree_1
    for entry1 in &tree_1.entries {
        let name1 = match entry1 {
            TreeEntry::File { name, blob_hash: _ } => name,
            TreeEntry::Directory { name, tree_hash: _ } => name,
        };
        let found = tree_2.entries.iter().any(|e| {
            let name2 = match e {
                TreeEntry::File { name, blob_hash: _ } => name,
                TreeEntry::Directory { name, tree_hash: _ } => name,
            };
            name1 == name2
        });
        if !found {
            println!("Removed: {}", full_path(name1));
        }
    }

    // get added entries only in tree_2
    for entry2 in &tree_2.entries {
        let name2 = match entry2 {
            TreeEntry::File { name, blob_hash: _ } => name,
            TreeEntry::Directory { name, tree_hash: _ } => name,
        };
        let found = tree_1.entries.iter().any(|e| {
            let name1 = match e {
                TreeEntry::File { name, blob_hash: _ } => name,
                TreeEntry::Directory { name, tree_hash: _ } => name,
            };
            name1 == name2
        });
        if !found {
            println!("Added: {}", full_path(name2));
        }
    }

    // get modified entries
    for entry2 in &tree_2.entries {
        let name2 = match entry2 {
            TreeEntry::File { name, blob_hash: _ } => name,
            TreeEntry::Directory { name, tree_hash: _ } => name,
        };
        
        // get matching entry in tree_1 by name
        if let Some(entry1) = tree_1.entries.iter().find(|e| {
            let name1 = match e {
                TreeEntry::File { name, blob_hash: _ } => name,
                TreeEntry::Directory { name, tree_hash: _ } => name,
            };
            name1 == name2
        }) {
            // Both entries exist, compare their hashes
            let hash1 = match entry1 {
                TreeEntry::File { name: _, blob_hash } => blob_hash,
                TreeEntry::Directory { name: _, tree_hash } => tree_hash,
            };
            let hash2 = match entry2 {
                TreeEntry::File { name: _, blob_hash } => blob_hash,
                TreeEntry::Directory { name: _, tree_hash } => tree_hash,
            };
            
            if hash1 != hash2 {
                match (entry1, entry2) {
                    // Both are files - file was modified
                    (TreeEntry::File { .. }, TreeEntry::File { .. }) => {
                        println!("Modified: {}", full_path(name2));
                        show_file_diff(hash1, hash2, &full_path(name2));
                    }
                    // Both are directories - recursively compare them
                    (TreeEntry::Directory { .. }, TreeEntry::Directory { .. }) => {
                        // recursively comparing the subdirectories to look for change
                        compare_trees_recursive(hash1.clone(), hash2.clone(), &full_path(name2));
                    }
                    // Type changed (file -> directory or vice versa)
                    _ => {
                        println!("Type changed: {} (was {:?}, now {:?})", full_path(name2), entry1, entry2);
                    }
                }
            }
        }
    }
}


fn cmd_init() {
    fs::create_dir_all(".snap/objects").unwrap();
    fs::create_dir_all(".snap/refs/heads").unwrap();
    if !fs::metadata(".snap/HEAD").is_ok() {
        fs::write(".snap/HEAD", "ref: refs/heads/main").unwrap();
    }
    println!("Initialized empty repository");
}

fn cmd_add(directory: &str) {
    fs::create_dir_all(".snap/objects").unwrap();
    
    let mut staged_files: HashMap<String, String> = match fs::read_to_string(".snap/INDEX") {
        Ok(data) => serde_json::from_str(&data).unwrap_or(HashMap::new()),
        Err(_) => HashMap::new(),
    };
    
    read_directory_and_stage(directory, &mut staged_files);
    
    // Save updated staging area
    let json = serde_json::to_string(&staged_files).unwrap();
    fs::write(".snap/INDEX", json).unwrap();
    
    println!("Added files from {} to staging area", directory);
}

fn read_directory_and_stage(dir: &str, staged_files: &mut HashMap<String, String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.to_str().unwrap().contains(".snap") {
            continue;
        }
        
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();
            let blob_hash = save_blob(content);  // Save blob to objects/
            let file_path = path.to_str().unwrap().to_string();
            staged_files.insert(file_path, blob_hash);
        } else if path.is_dir() {
            read_directory_and_stage(path.to_str().unwrap(), staged_files);
        }
    }
}

fn cmd_commit(message: &str) {
    let staged_data = fs::read_to_string(".snap/INDEX").unwrap();
    let staged_files: HashMap<String, String> = serde_json::from_str(&staged_data).unwrap();
    
    let tree = Tree {
        entries: staged_files.into_iter().map(|(name, blob_hash)| {
            TreeEntry::File { name, blob_hash }
        }).collect()
    };
    
    let tree_hash = save_tree(tree);
    
    let commit = Commit {
        tree_hash: tree_hash,
        parent: get_last_commit(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        message: message.to_string() 
    };
    
    save_commit(commit);
    
    // Clear staging area after commit
    fs::write(".snap/INDEX", "{}").unwrap();
    
    println!("Commit created: {}", message);
}

fn cmd_diff() {
    let current_commit_hash = get_last_commit();
    
    // Check if there's a commit to compare
    if current_commit_hash.is_empty() {
        println!("No commits to compare");
        return;
    }
    
    let current_commit_data = fs::read_to_string(format!(".snap/objects/{}", current_commit_hash)).unwrap();
    let current_commit: Commit = serde_json::from_str(&current_commit_data).unwrap();
    let tree_hash_current = current_commit.tree_hash;
    
    // Check if there's a parent commit
    if current_commit.parent.is_empty() {
        println!("No parent commit to compare with");
        return;
    }
    
    let parent_commit_data = fs::read_to_string(format!(".snap/objects/{}", current_commit.parent)).unwrap();
    let parent_commit: Commit = serde_json::from_str(&parent_commit_data).unwrap();
    let tree_hash_parent = parent_commit.tree_hash;
    diff_fn(tree_hash_parent, tree_hash_current);
}

fn scan_working_directory(dir: &str, files: &mut HashMap<String, String>, skip_snap: bool) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        if skip_snap && path_str.contains(".snap") {
            continue;
        }

        if path.is_file() {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let blob_hash = digest(&content);
            // Normalize path by removing leading ./
            let normalized_path = path_str.strip_prefix("./").unwrap_or(path_str);
            files.insert(normalized_path.to_string(), blob_hash);
        } else if path.is_dir() {
            scan_working_directory(path_str, files, skip_snap);
        }
    }
}

fn cmd_status(directory: &str) {
    // Read staged files
    let staged_files: HashMap<String, String> = match fs::read_to_string(".snap/INDEX") {
        Ok(data) => serde_json::from_str(&data).unwrap_or(HashMap::new()),
        Err(_) => HashMap::new(),
    };

    let mut working_files: HashMap<String, String> = HashMap::new();
    scan_working_directory(directory, &mut working_files, true);

    println!("On branch main\n");

    // changes to be committed
    let mut has_staged = false;
    for (file_path, _staged_hash) in &staged_files {
        if !has_staged {
            println!("Changes to be committed:");
            has_staged = true;
        }
        println!("  \x1b[32mnew file:   {}\x1b[0m", file_path);
    }
    if has_staged {
        println!();
    }

    // Show modified files changes not staged for commit
    let mut has_modified = false;
    for (file_path, working_hash) in &working_files {
        if let Some(staged_hash) = staged_files.get(file_path) {
            if working_hash != staged_hash {
                if !has_modified {
                    println!("Changes not staged for commit:");
                    has_modified = true;
                }
                println!("  \x1b[31mmodified:   {}\x1b[0m", file_path);
            }
        }
    }
    if has_modified {
        println!();
    }

    // Show untracked files
    let mut has_untracked = false;
    for (file_path, _) in &working_files {
        if !staged_files.contains_key(file_path) {
            if !has_untracked {
                println!("Untracked files:");
                has_untracked = true;
            }
            println!("  \x1b[31m{}\x1b[0m", file_path);
        }
    }
    if has_untracked {
        println!();
    }

    if !has_staged && !has_modified && !has_untracked {
        println!("nothing to commit, working tree clean");
    }
}

fn cmd_log() {
    let mut current = get_last_commit();

    if current.is_empty() {
        println!("No commits yet");
        return;
    }

    println!("Commit history (from current HEAD):\n");

    while !current.is_empty() {
        let commit_hash = current.clone();
        let commit_data = match fs::read_to_string(format!(".snap/objects/{}", commit_hash)) {
            Ok(data) => data,
            Err(_) => break,
        };
        let commit: Commit = match serde_json::from_str(&commit_data) {
            Ok(commit) => commit,
            Err(_) => break,
        };

        let is_head = commit_hash == get_last_commit();
        let marker = if is_head { " (HEAD)" } else { "" };

        println!("commit {}{}", commit_hash, marker);
        println!("Message: {}", commit.message);
        println!("Timestamp: {}", commit.timestamp);
        println!();

        current = commit.parent;
    }
}

fn cmd_log_all() {
    println!("All commits in repository:\n");

    let mut commits = Vec::new();

    // Read all objects and filter for commits
    if let Ok(entries) = fs::read_dir(".snap/objects") {
        for entry in entries {
            if let Ok(entry) = entry {
                let hash = entry.file_name().to_string_lossy().to_string();
                if let Ok(data) = fs::read_to_string(format!(".snap/objects/{}", hash)) {
                    if let Ok(commit) = serde_json::from_str::<Commit>(&data) {
                        commits.push((hash, commit));
                    }
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    commits.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));

    let current_head = get_last_commit();

    for (hash, commit) in commits {
        let is_head = hash == current_head;
        let marker = if is_head { " (HEAD)" } else { "" };

        println!("commit {}{}", hash, marker);
        println!("Message: {}", commit.message);
        println!("Timestamp: {}", commit.timestamp);
        println!();
    }
}

fn collect_tree_files(tree_hash: &str, base_path: &str, files: &mut std::collections::HashSet<String>) {
    let tree_data = match fs::read_to_string(format!(".snap/objects/{}", tree_hash)) {
        Ok(data) => data,
        Err(_) => return,
    };
    let tree: Tree = match serde_json::from_str(&tree_data) {
        Ok(t) => t,
        Err(_) => return,
    };

    for entry in tree.entries {
        match entry {
            TreeEntry::File { name, blob_hash: _ } => {
                let file_path = if base_path.is_empty() {
                    name
                } else {
                    format!("{}/{}", base_path, name)
                };
                files.insert(file_path);
            }
            TreeEntry::Directory { name, tree_hash } => {
                let dir_path = if base_path.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", base_path, name)
                };
                collect_tree_files(&tree_hash, &dir_path, files);
            }
        }
    }
}

fn collect_work_directory_files(dir: &str, files: &mut std::collections::HashSet<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        if path_str.contains(".snap") {
            continue;
        }

        if path.is_file() {
            files.insert(path_str.to_string());
        } else if path.is_dir() {
            collect_work_directory_files(path_str, files);
        }
    }
}

fn restore_tree(tree_hash: &str, base_path: &str) {
    let tree_data = fs::read_to_string(format!(".snap/objects/{}", tree_hash)).unwrap();
    let tree: Tree = serde_json::from_str(&tree_data).unwrap();

    for entry in tree.entries {
        match entry {
            TreeEntry::File { name, blob_hash } => {
                // Read blob content
                let content = fs::read_to_string(format!(".snap/objects/{}", blob_hash)).unwrap();

                // Construct full path
                let file_path = if base_path.is_empty() {
                    name
                } else {
                    format!("{}/{}", base_path, name)
                };

                // Create parent directories if needed
                if let Some(parent) = std::path::Path::new(&file_path).parent() {
                    fs::create_dir_all(parent).ok();
                }

                // Write file to disk
                fs::write(&file_path, content).unwrap();
                println!("Restored: {}", file_path);
            }
            TreeEntry::Directory { name, tree_hash } => {
                // Construct subdirectory path
                let dir_path = if base_path.is_empty() {
                    name
                } else {
                    format!("{}/{}", base_path, name)
                };

                // Create directory
                fs::create_dir_all(&dir_path).ok();

                // Recursively restore subdirectory
                restore_tree(&tree_hash, &dir_path);
            }
        }
    }
}

fn cmd_rollback(commit_hash: &str, directory: &str) {
    let commit_data = match fs::read_to_string(format!(".snap/objects/{}", commit_hash)) {
        Ok(data) => data,
        Err(_) => {
            println!("Error: Commit {} not found", commit_hash);
            return;
        }
    };
    let commit: Commit = match serde_json::from_str(&commit_data) {
        Ok(c) => c,
        Err(_) => {
            println!("Error: Invalid commit data");
            return;
        }
    };

    println!("Rolling back to commit: {}", commit.message);
    println!("This will affect files in: {}\n", directory);

    // files existing in target commit
    let mut target_files = std::collections::HashSet::new();
    collect_tree_files(&commit.tree_hash, "", &mut target_files);

    // current files in the directory
    let mut current_files = std::collections::HashSet::new();
    collect_work_directory_files(directory, &mut current_files);

    // Delete files that exist currently but not in target commit
    for file in &current_files {
        // Extract the relative path within the directory
        let relative_path = file.strip_prefix(&format!("{}/", directory))
            .or_else(|| file.strip_prefix(directory))
            .unwrap_or(file);

        if !target_files.contains(relative_path) {
            match fs::remove_file(file) {
                Ok(_) => println!("Deleted: {}", file),
                Err(e) => println!("Warning: Failed to delete {}: {}", file, e),
            }
        }
    }

    // Restore all files from target commit
    restore_tree(&commit.tree_hash, "");

    fs::write(".snap/HEAD", commit_hash).unwrap();

    println!("\nRollback complete! HEAD is now at {}", &commit_hash[..12]);
}

fn create_branch(branch_name: String) {
    let latest_commit = get_last_commit();
    if fs::exists(format!(".snap/refs/heads/{}", branch_name)).unwrap(){
        println!("Branch already exists");
        return;
    }
    fs::write(format!(".snap/refs/heads/{}", branch_name), &latest_commit).unwrap();
    println!("Branch {} created at commit {}", branch_name, &latest_commit[..12]);
}

fn switch_branch(branch_name: String, directory: &str) {
    let commit_hash = fs::read_to_string(format!(".snap/refs/heads/{}", branch_name)).unwrap();
    let commit_data = fs::read_to_string(format!(".snap/objects/{}", commit_hash)).unwrap();
    let commit: Commit = serde_json::from_str(&commit_data).unwrap();

    clear_working_directory(directory);

    restore_tree(&commit.tree_hash, "");

    fs::write(".snap/HEAD", format!("ref: refs/heads/{}", branch_name)).unwrap();

    println!("Switched to branch '{}'", branch_name);
    println!("HEAD is now at {}", &commit_hash[..12]);
}

fn list_branches() {
    let head_content = fs::read_to_string(".snap/HEAD").unwrap_or_default();
    let current_branch = if head_content.starts_with("ref: refs/heads/") {
        head_content.trim().strip_prefix("ref: refs/heads/").unwrap_or("")
    } else {
        ""
    };

    println!("Branches:");

    if let Ok(heads_dir) = fs::read_dir(".snap/refs/heads") {
        let mut found_branches = false;
        for entry in heads_dir {
            if let Ok(entry) = entry {
                let branch_name = entry.file_name().to_str().unwrap().to_string();
                let marker = if branch_name == current_branch { " *" } else { "  " };
                println!("{} {}", marker, branch_name);
                found_branches = true;
            }
        }
        if !found_branches {
            println!("  (no branches yet - create one with 'branch <name>')");
        }
    }
}

fn clear_working_directory(dir: &str) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        if path_str.contains(".snap") {
            continue;
        }

        if path.is_file() {
            fs::remove_file(path).ok();
        } else if path.is_dir() {
            clear_working_directory(path_str);
            fs::remove_dir(path).ok(); // Remove empty directory
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [args]", args[0]);
        println!("Commands: init, add <directory>, commit <message>, diff, status");
        return;
    }

    match args[1].as_str() {
        "init" => cmd_init(),
        "add" => {
            if args.len() < 3 {
                println!("Usage: {} add <directory>", args[0]);
                return;
            }
            cmd_add(&args[2]);
        }
        "commit" => {
            if args.len() < 3 {
                println!("Usage: {} commit <message>", args[0]);
                return;
            }
            cmd_commit(&args[2]);
        }
        "diff" => cmd_diff(),
        "status" => {
            if args.len() < 3 {
                println!("Usage: {} status <directory>", args[0]);
                return;
            }
            cmd_status(&args[2]);
        }
        "log" => {
            if args.len() > 2 && args[2] == "--all" {
                cmd_log_all();
            } else {
                cmd_log();
            }
        }
        "rollback" => {
            if args.len() < 4 {
                println!("Usage: {} rollback <commit_hash> <directory>", args[0]);
                println!("Example: {} rollback abc123... test_project", args[0]);
                return;
            }
            cmd_rollback(&args[2], &args[3]);
        }
        "branch" => {
            list_branches();
        }
        "checkout" => {
            create_branch(args[2].clone());
        }
        "switch" => {
            if args.len() < 4 {
                println!("Usage: {} switch <branch_name> <directory>", args[0]);
                println!("Example: {} switch feature-x test_project", args[0]);
                return;
            }
            switch_branch(args[2].clone(), &args[3]);
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Commands: init, add <directory>, commit <message>, diff, status, log, rollback <commit_hash> <directory>, branch [name], switch <branch> <directory>");
        }
    }
}
