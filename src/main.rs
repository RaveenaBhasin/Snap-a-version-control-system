use std::{collections::HashMap, fs, hash::Hash, path, time::{Duration, SystemTime, UNIX_EPOCH}};
use sha256::digest;
use serde::Serialize;

#[derive(Serialize)]
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

    save_commit(commit);
}
