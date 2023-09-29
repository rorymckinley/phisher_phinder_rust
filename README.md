## env files

`env.test.example` and `env.example` can be used as a template for the required ENV files.

## Running all the modules

`cat dodgy.eml | cargo run --bin pp-parser | cargo run --bin pp-url-enumerator | cargo run --bin pp-rdap | env $(cat .env | xargs) cargo run --bin pp-mailer`

## Running tests

Mailtrap account is required to run the all the tests, but a free account only allows 100 mails
per month :( - consider a feature to disable mail testing.

Start the mountebank container: `docker-compose up -d`
Run the tests: `env $(cat .env.test | xargs) cargo test --features test-mocks`

## Processing multiple files

cat file.mbox | cargo run --bin pp-source-parser | env $(cat .env | xargs) cargo run --bin pp-store-mail-source | cargo run --bin pp-source-splitter | ./analyser_wrapper.sh

## Reprocessing a message

env $(cat .env | xargs) cargo run --bin pp-fetch-run-details -- --pipe-message-source 2419 | cargo run --bin pp-source-parser | env $(cat .env | xargs) cargo run --bin pp-store-mail-source | cargo run --bin pp-source-splitter | ./analyser_wrapper.sh
