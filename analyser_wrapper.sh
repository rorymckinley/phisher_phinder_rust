#!/bin/bash

while read -r  line; do
  echo $line | env $(cat .env | xargs) cargo run --bin pp-parser | env $(cat .env | xargs) cargo run --bin pp-url-enumerator | env $(cat .env | xargs) cargo run --bin pp-rdap | env $(cat .env | xargs) cargo run --bin pp-reporter | env $(cat .env | xargs) cargo run --bin pp-store-run-details
done
