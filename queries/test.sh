#!/bin/bash

# Check for the required argument
if [ $# -lt 2 ]; then
  echo "Expected exactly 2 arguments, got $#"
  exit 1
fi

query=$1

# 2nd argument optional, default: ""
oracle=$2

# Docker stuff
container="fuzzi"
db_path_old="/usr/bin/sqlite3-3.26.0"
db_path_new="/usr/bin/sqlite3-3.39.4"

# Output
out_old=$(docker exec -i "$container" "$db_path_old" -bail :memory: <<< $query 2>&1)
out_new=$(docker exec -i "$container" "$db_path_new" -bail :memory: <<< $query 2>&1)

output="$out_old,$out_new"

# no oracle given => return oracle
if [ -z "$oracle" ]; then
  echo "$output"
  exit 0
fi

echo $([[ "$output" == "$oracle" ]] && echo 1 || echo 0)