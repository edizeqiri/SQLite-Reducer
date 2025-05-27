#!/bin/bash

if [ $# -lt 1 ]; then
  echo "Usage: $0 <query> [oracle]"
  exit 1
fi

query=$1
oracle=$2

container="fuzzi"
db_path_old="/usr/bin/sqlite3-3.26.0"
db_path_new="/usr/bin/sqlite3-3.39.4"

run_sqlite() {
  local db_path=$1
  # Run the query inside docker and capture stdout+stderr
  local output
  output=$(docker exec -i "$container" "$db_path" -bail :memory: <<< "$query" 2>&1)
  local status=$?


  if [ $status -gt 128 ]; then
    local signal=$((status - 128))
    echo "CRASH(Segmentation fault (signal $signal))"
  else
    echo "$output"
  fi
}

out_old=$(run_sqlite "$db_path_old")
out_new=$(run_sqlite "$db_path_new")

output="${out_old}&${out_new}"

if [ -z "$oracle" ]; then
  echo "$output"
  #echo "$output;" >> /Users/saschatran/Desktop/Uni_gits/reducer/src/resources/test.csv
  exit 0
fi

echo $([[ "$output" == "$oracle" ]] && echo 1 || echo 0)
