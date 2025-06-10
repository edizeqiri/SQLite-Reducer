# Reducer

## Build
### Docker
We have provided a docker image in the root of this directory which can be built simply with

```bash
docker build -t reducer .
```
### Native

To build the native binary you need latest rust. Then you can build it with

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

If the env `QUICK_RUN` is set, then the bruteforce will not reduce and it will go much quicker.

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

Current directory: /home/test
[2025-06-10T13:01:07Z WARN  reducer::utils] 65,1,4738,4,214.829366
[2025-06-10T13:01:07Z WARN  reducer::utils] [ANALYSIS] ".mode quote CREATE TABLE;" [END ANALYSIS]

```

The `/output` folder has all the outputs and queries of the reduction.

We have also provided a test-script.sh in the queries directory for every query with the right SQL_NUMBER present.