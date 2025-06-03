cd ../..
export RUST_LOG=info;export CARGO_TARGET_DIR=/target
for i in {1..20}; do
  cargo run --package reducer --bin reducer -- --query queries/query$i/original_test.sql --test src/resources/native.sh
done

