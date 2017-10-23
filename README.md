# stars2nix
Produce Nix expressions from GitHub star metadata.

# Running
    mkdir stars; touch cursor.sh
    source cursor.sh && GITHUB_API_TOKEN="..." cargo script stars2nix | tee -a cursor.sh

# Output
A Nix expression for each starred repository is written to `stars/<owner>/<repo>.nix`.
Each expression is a set with keys `repoName`, `repoOwner`, `repoUrl`, `mainUrl`, `created`, `starred`, and `description`.
File paths are printed to standard output as they are written, prefixed with `#`.
When the script completes, the result cursor (if any) is printed to standard out like `export START_CURSOR="..."`.

