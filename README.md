## Running tests

Start the mountebank container: `docker-compose up -d`
Run the tests: `RUST_TEST_THREADS=1 cargo test --features test-mocks`
