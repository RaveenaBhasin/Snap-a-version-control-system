# Snap a version control system 

## Commands

```bash
# Initialize repository
cargo run -- init

# Stage files from directory
cargo run -- add <directory>

# Create commit
cargo run -- commit <message>

# Show differences between commits
cargo run -- diff

# Show repository status
cargo run -- status <directory>

# View commit history
cargo run -- log
cargo run -- log --all

# Rollback to a specific commit
cargo run -- rollback <commit_hash> <directory>

# List branches
cargo run -- branch

# Create new branch
cargo run -- checkout <branch_name>

# Switch to branch
cargo run -- switch <branch_name> <directory>
```
