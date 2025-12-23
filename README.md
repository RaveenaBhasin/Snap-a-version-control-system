Read file contents in a directory

for each file create
    - hash its contents
    - create blob of each file (creating blob which has just the contents without the file name helps prevent de duplication of the same content)

create commit with file -> blob_hash

save commit


In current structure the one without trees there is no directory structure
Commits store everything in flat format. out of 10 files even if one is changed it will create a new commit with 10 file mappings