# Phisher Phinder Rust

PhisherPhinderRust (PPR) is a utility intended to help identify infrastructure used to send or
support phishing/scam emails.

It does not identify phishing/scam emails - there are already a number of tools that do this as
well as can be reasonably hoped. PPR relies on these tools to identify the emails at which point it
can extract details from the raw email source.

These extracted details are used to identify the providers/owners of infrastructure and then notify
them that their infrastructure is being used to send or support scam emails. 

The initial principle is to use free tool/ data sources as much as possible so that anybody can
use PPR to process scam emails and notifiy the providers. Once this has reached an acceptable
level I would like to add optional support to commercial services (e.g. a passive DNS provider).

## Caveat Emptor

As of Oct 2023, I am not sure if I am happy enough to have PPR sending mails to providers, although
it is technically capable. However, I do believe it is useful if you wish to identify some of
the infrastructure behind an email.

The code quality is well below what I would prefer, as PPR started as a weekend project to scratch an
itch while allowing me to retain some basic Rust skills. I am hoping to improve the code quality over
time but I have strived for as much test coverage as makes sense.

Documentation is non-existent (it seems to always lose when deciding to spend time on functionality
vs code quality vs documentation). If PPR does seem like it will be useful, please open an issue and
I will be happy to provide you with the necessary support to get you up and running.

Note: I use Linux for development so PPR may not work on OS X and will definitely not work on
Windows. This is a practical choice and not a religious one :), so I am happy to assist with what
is needed to get PPR running on other platforms.

## Functionality

### Importing mail source(s)

PPR can import a file containing the message source of a single email or multiple emails. For a
file containing multiple message sources, the only format currently supported is the mbox format
produced by the Google takeout service.

```
cat /path/to/source/file | env $(cat .env | xargs) cargo run --bin ppr
```

Imported message sources are stored in the sqlite database specified in the ENV variables
(`PP_DBATH`).

### Reprocessing an existing message source

PPR can reprocess a message source that already exists in the DB. You may want to do this if the
message source was processed by an earlier version of PPR that had an bug or lacked a particular
feature.

Counterintuitively, you need to provide the id of a run linked to the message source rather than
the id of the message source.

```
env $(cat .env | xargs) cargo run --bin ppr --reprocess-run <RUN_ID>
```

## env files

`env.test.example` and `env.example` can be used as a template for the required ENV files.

## Processing a single email source file (DEPRECATED)

`cat dodgy.eml | cargo run --bin pp-source-parser | env $(cat .env | xargs) cargo run --bin pp-store-mail-source | cargo run --bin pp-source-splitter | ./analyser_wrapper.sh`

## Processing multiple files (DEPRECATED)

`cat file.mbox | cargo run --bin pp-source-parser | env $(cat .env | xargs) cargo run --bin pp-store-mail-source | cargo run --bin pp-source-splitter | ./analyser_wrapper.sh`

## Reprocessing a message (DEPRECATED)

`env $(cat .env | xargs) cargo run --bin pp-fetch-run-details -- --pipe-message-source 2419 | cargo run --bin pp-source-parser | env $(cat .env | xargs) cargo run --bin pp-store-mail-source | cargo run --bin pp-source-splitter | ./analyser_wrapper.sh`

## Running tests

Start the mountebank container:

`docker-compose up -d`

To run all the tests (see below) a [Mailtrap](https://mailtrap.io/) account is required.

### Running without mail-sending tests

Given the current deprioritisation of mail sending as a feature, the default test run will **not**
run the mail sending tests. This reduces the amount of traffic to the Mailtrap account, as the 
free account only allows 100 mails per month.

`env $(cat .env.test | xargs) cargo test --features test-mocks`

### Running with mail-sending tests

`env $(cat .env.test | xargs) cargo test --features test-mocks -- --include-ignored`
