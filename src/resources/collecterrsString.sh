cd ..
for i in {1..20}; do
  cargo run --package reducer --bin reducer -- --query /Users/saschatran/Desktop/Uni_gits/reducer/queries/query$i/original_test.sql --test /Users/saschatran/Desktop/Uni_gits/reducer/queries/test.sh
done