#!/bin/bash

# Usage: ./showfile.sh /path/to/file


# Check for the required argument
if [ $# -lt 2 ]; then
  echo "Usage: $0 <file-path>"
  exit 1
fi

query=$1

request_oracle=$2

# Docker stuff
container="fuzzi"
db_path_old="/usr/bin/sqlite3-3.26.0"
db_path_new="/usr/bin/sqlite3-3.39.4"

# Output
out_old=$(docker exec -i "$container" "$db_path_old" -bail :memory: <<< $query 2>&1)
out_new=$(docker exec -i "$container" "$db_path_new" -bail :memory: <<< $query 2>&1)

if [ "$request_oracle" -eq "1" ]; then
  echo "$out_old,$out_new"
  exit 0
fi

echo "1"