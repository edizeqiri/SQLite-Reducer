export RUSTFLAGS="-A warnings"
export RUST_LOG=warn
export CARGO_TARGET_DIR=/target
cargo build --package reducer --bin reducer    # or add --release for a release build

mkdir -p /output/logs

for i in {1..20}; do
  (
    export RUST_LOG=warn

    export SQL_NUMBER=$i
    export TEST_CASE_LOCATION=/output/query$i.sql

    reducer \
      --query queries/query$i/original_test.sql \
      --test native.sh \
    2>&1 | tee /output/logs/job_${i}.log
  ) &

done

wait
echo "All jobs done. Logs in /output/logs/"
