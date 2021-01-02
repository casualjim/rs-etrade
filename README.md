# Rust etrade

Wraps the etrade API and implements the required oauth1 flow.

## State storage

The default feature for the crate includes a thread safe in-memory store for the oauth tokens.
There is an optional feature `secretservice` which will the keychain on linux to store the token information.
