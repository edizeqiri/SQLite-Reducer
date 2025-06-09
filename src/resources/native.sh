#!/bin/bash
if [ $# -lt 1 ]; then
  echo "Usage: $0 <query> [oracle]"
  exit 2
fi

if [ -f .env ]; then
  # The `set -a` tells Bash: export all variables that are subsequently defined
  set -a
  source .env
  set +a
fi
curr_query_num=$SQL_NUMBER
oracle_txt_path="queries/query$curr_query_num/oracle.txt"

read -r given_oracle < "$oracle_txt_path"

query=$1
oracle=$2

container="fuzzi"
db_path_old="/usr/bin/sqlite3-3.26.0"
db_path_new="/usr/bin/sqlite3-3.39.4"

run_sqlite() {
  local db_path=$1
  # Run the query inside and capture stdout+stderr
  local output
  output=$("$db_path" < "$query" 2>&1) # sqlite3 < /output/query.sql

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
#echo $output

if [[ "$oracle" == "" ]]; then
  echo "$output"
  #echo "$output;" >> /Users/saschatran/Desktop/Uni_gits/reducer/src/resources/test.csv
  exit 0
fi

# if oracle contains disk image malformed then reduction successful
if [[ "$output" == *disk\ image\ is\ malformed* ]]; then
 # echo "BASH $output"
  exit 0
fi


#echo "BASH $output"
#echo "OUT BASH $output ENDBASH"


if [[ "$given_oracle" == *DIFF* ]]; then
  parse_old=${out_old#Error: }

  # remove weird prefixes
  parse_new=${out_new#Parse error[[:space:]]}
  parse_new=${parse_new#Runtime error[[:space:]]}

  #echo "BASH $parse_old&$parse_new ENDBASH"
  # exit 1 when out_old == out_new, exit 0 otherwise

  if [[ "$parse_old"  == *"near line"* && \
      "$parse_new" == *"near line"* ]]; then
  #echo "Both old and new contain 'near line', aborting."
  exit 1
  fi

  if [[ "$parse_old" != "$parse_new" ]]; then
    #echo "QUERY $curr_query_num: $parse_old&$parse_new" >> /output/query.txt
    exit 0
  else
    exit 1
  fi

fi

# exit 0 when output == oracle, exit 1 otherwise
if [[ "$output" == "$oracle" ]]; then
  exit 0
else
  echo "QUERY $curr_query_num: $output" >> /output/query.txt
  exit 1
fi


