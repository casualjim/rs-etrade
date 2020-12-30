# Rust etrade

This repo contains a library to interact with the etrade API.
It also contains a cli command to interact with the etrade API.

In the library project there is an auth session store which lives in memory, so it will be empty when you quit the application.

For the CLI project there is a session store implementation which uses the secret service on linux to keep the state for the auth session securely. This makes it so that you can run the command several times during a day without the need to log in every time.  If all goes well you should only need to log on once every 24 hours.
