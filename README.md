# Reducer

## Build

To build the binary you need latest rust. Then you can build it with

```bash
cargo build --release
```
The binary will then be in ` target/release/reducer`

## Run

We have implemented the interface as described in the project description:

```bash
reducer --query [File with query to be reduced] --test [Test Script to approve reduction]
```
The result of the reduction will be printed as a WARN log and saved to the file in `TEST_CASE_LOCATION`.

If the env `QUICK_RUN` is set, then the bruteforce will not reduce and it will go 100x quicker.

## Test Script

> If you use your own test script then this is not important for you.

If you want to use our test script, it is called `native.sh` and is also copied in the `Dockerfile`.
To run all the 20 queries at once with our test script, please use `full_run.sh`, as our test script needs the env `SQL_NUMBER` to be present.

Example with docker:

```bash
docker build -t reducer .
docker run -it --rm reducer /bin/bash -c "bash full_run.sh"
```

If you only want to run one query (query1 for example):

```bash
docker build -t reducer .
docker run -it --rm reducer /bin/bash -c "export SQL_NUMBER=17; export TEST_CASE_LOCATION=/output/query17.sql; reducer --query queries/query17/original_test.sql --test native.sh"
```

