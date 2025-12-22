use std::{collections::HashMap, fs, hash::Hash, path, time::{Duration, SystemTime, UNIX_EPOCH}};
use sha256::digest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Commit {
    files: HashMap<String, String>,
    parent: String,
    timestamp: i64,
    message: String,
}

fn save_blob(content: String) -> String{
    let hash = digest(content);
    return hash;
}

fn save_commit(commit: Commit) {
    let json = serde_json::to_string(&commit).unwrap();
    let commit_hash = digest(&json); 
    fs::write(format!(".mygit/objects/{}", &commit_hash), json).unwrap();
    fs::write(".mygit/HEAD", &commit_hash).unwrap();
}

fn get_last_commit() -> String {
    let head = fs::read_to_string(".mygit/HEAD").unwrap();
    return head.trim().to_string();
}

fn log() {
    let mut current = get_last_commit();
    
    while !current.is_empty() {
        let commit_hash = current.clone();
        let commit_data = match fs::read_to_string(format!(".mygit/objects/{}", commit_hash)) {
            Ok(data) => data,
            Err(_) => break,
        };
        let commit: Commit = match serde_json::from_str(&commit_data) {
            Ok(commit) => commit,
            Err(_) => break,
        };
        
        println!("commit {}", commit_hash);
        println!("Message: {}", commit.message);
        println!("Timestamp: {}", commit.timestamp);
        println!("Files: {:?}", commit.files.keys());
        println!();
        
        current = commit.parent;
    }
}

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

fn diff_fn(commit_hash_1: String, commit_hash_2: String) {
    let commit_data_1 = fs::read_to_string(format!(".mygit/objects/{}", commit_hash_1)).unwrap();
    let commit_1 : Commit = serde_json::from_str(&commit_data_1).unwrap();
    let commit_data_2 = fs::read_to_string(format!(".mygit/objects/{}", commit_hash_2)).unwrap();
    let commit_2 : Commit = serde_json::from_str(&commit_data_2).unwrap();

    println!("Diff between {} and {}", commit_hash_1, commit_hash_2);
    println!("---");

    for filename in commit_1.files.keys() {
        if !commit_2.files.contains_key(filename) {
            println!("Removed: {:?} Hash {:?}", filename, commit_1.files.get(filename).unwrap());
        }
    }

    for filename in commit_2.files.keys() {
        if !commit_1.files.contains_key(filename) {
            println!("Added: {:?} Hash {:?}", filename, commit_2.files.get(filename).unwrap());
        }
    }

    for filename in commit_2.files.keys() {
        // Only check files that exist in BOTH commits
        if commit_1.files.contains_key(filename) {
            let hash1 = commit_1.files.get(filename).unwrap();
            let hash2 = commit_2.files.get(filename).unwrap();
            if hash1 != hash2 {
                println!("Modified: {:?} Hash {:?} -> Hash {:?}", filename, hash1, hash2);
            }
        }
    }
}

fn main() {
    fs::create_dir_all(".mygit/objects").unwrap();
    
    let mut files = HashMap::new();
    read_directory("test_project", &mut files);
    
    let commit = Commit {
        files,
        parent: get_last_commit(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        message: "Commit all test project files 2".to_string() 
    };
    
    // save_commit(commit);
    
    println!("\n--- Commit History ---");
    log();


    let current = get_last_commit();
    let commit_data = fs::read_to_string(format!(".mygit/objects/{}", current)).unwrap();
    let commit : Commit = serde_json::from_str(&commit_data).unwrap();
    diff_fn(current, commit.parent.clone());

}
