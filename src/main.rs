use std::{collections::HashMap, fs, path::Path, time::{SystemTime, UNIX_EPOCH}};
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
    fs::write(".snap/HEAD", &commit_hash).unwrap();
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
    return head.trim().to_string();
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
                        println!("Modified: {} hash changed from {:?} to {:?}", full_path(name2), hash1, hash2);
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
    if !fs::metadata(".snap/HEAD").is_ok() {
        fs::write(".snap/HEAD", "").unwrap();
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [args]", args[0]);
        println!("Commands: init, add <directory>, commit <message>, diff");
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
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Commands: init, add <directory>, commit <message>, diff");
        }
    }
}
