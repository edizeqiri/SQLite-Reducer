#!/usr/bin/env bash

# Usage: ./showfile.sh /path/to/file

# Exit immediately on error, undefined variable, or pipe failure
set -euo pipefail

# Check for the required argument
if [ $# -lt 1 ]; then
  echo "Usage: $0 <file-path>"
  exit 1
fi

FILE_PATH="$1"

# Check that the path exists and is a regular file
if [ ! -e "$FILE_PATH" ]; then
  echo "Error: '$FILE_PATH' does not exist."
  exit 2
fi

if [ ! -f "$FILE_PATH" ]; then
  echo "Error: '$FILE_PATH' is not a regular file."
  exit 3
fi

# Check that the file is readable
if [ ! -r "$FILE_PATH" ]; then
  echo "Error: '$FILE_PATH' is not readable."
  exit 4
fi

# Finally, output the file contents
cat -- "$FILE_PATH"
