Read file contents in a directory

for each file create
    - hash its contents
    - create blob of each file (creating blob which has just the contents without the file name helps prevent de duplication of the same content)

create commit with file -> blob_hash

save commit