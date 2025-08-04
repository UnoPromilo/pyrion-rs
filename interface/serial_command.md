## Overview

A command consists of a command name followed optionally by a list of key=value arguments. All commands are UTF-8
encoded, line-oriented, and whitespace-delimited. All commands have to be less than 256 bytes long.

## Syntax Rules

### Command Name

- Required
- First token in the command string
- Must match: [a-zA-Z][a-zA-Z0-9_]

### Arguments

- Optional
- Commands may define required arguments
- Max 8 arguments per command
- Each argument must be in *key=value* or *key* format
- Keys must match: [a-zA-Z_][a-zA-Z0-9_]
