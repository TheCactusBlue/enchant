Runs a bash command in a persistent shell session, in a secure way.

- Use bash commands with this tool for moving/renaming files, creating directories, or deleting files.
- If the command will create new directories or files, verify that the parent directory exists using other tools.

NEVER use certain commands - these will automatically be rejected by Bash, as there are better tools available to you:

- `pwd` (the working directory is already provided)
- `ls` (use LS tool instead)
- `cat` (use Write instead)
- `sed -i`, `awk` (use Edit instead)
- `grep`, `rg` (use Grep instead)
