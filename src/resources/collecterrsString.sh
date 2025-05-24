abs_path="/Users/saschatran/Desktop/Uni_gits/reducer"
cd ..
for i in {1..20}; do
  cargo run --package reducer --bin reducer -- --query $abs_path/queries/query$i/original_test.sql --test $abs_path/queries/test.sh
done

