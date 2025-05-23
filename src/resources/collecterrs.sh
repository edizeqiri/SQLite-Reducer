#!/usr/bin/env bash

for i in {1..20}; do
  FILE_PATH="/Users/saschatran/Desktop/Uni_gits/reducer/queries/query$i/original_test.sql"
  
  # Docker stuff
  container="fuzzi"
  db_path_old="/usr/bin/sqlite3-3.26.0"
  db_path_new="/usr/bin/sqlite3-3.39.4"
  
  # Output
  out_old=$(
    docker exec -i "$container" "$db_path_old" -bail :memory: \
      < "$FILE_PATH" 2>&1
  )
  
  out_new=$(
    docker exec -i "$container" "$db_path_new" -bail :memory: \
      < "$FILE_PATH" 2>&1
  )
  
  echo "$out_old,$out_new"
  # save output
  echo "$out_old,$out_new;" >> expected_output.csv
done