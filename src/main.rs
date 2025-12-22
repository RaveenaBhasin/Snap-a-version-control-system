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
    fs::write(format!(".mygit/commits/{}", &commit_hash), json).unwrap();
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
        let commit_data = match fs::read_to_string(format!(".mygit/commits/{}", commit_hash)) {
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

fn main() {
    let input = fs::read_to_string("hello.txt").unwrap();
    //read file name
    let path_name = path::Path::new("hello.txt").file_name().unwrap().to_str().unwrap();
    println!("path name {:?}", path_name);
    let blob_hash = save_blob(input);

    let mut files = HashMap::new();
    files.insert(path_name.to_string(), blob_hash);

    let commit = Commit {
        files,
        parent: get_last_commit(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        message: "second commit".to_string() 
    };

    // save_commit(commit);

    println!("\n--- Commit History ---");
    log();
}
