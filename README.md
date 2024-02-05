# scurl
A curl-like utility for making HTTP requests to Safenet servers. It will automatically negotiate keys with the server, as long as it adheres to the Safenet API (which of right now is the inclusion of an `/conn/http` HTTP endpoint).

## Installation
- Pre-built Binaries: [Cargo Actions](https://github.com/mateocabanal/scurl/actions)
- Cargo: `cargo install --git https://github.com/mateocabanal/scurl`

## Usage
`scurl <OPTS> <URL>`
#### Options
- `-m` or `--method`: `get` or `post` (case-insensitve)
- `d` or `--data`: If method is POST, the given string will be the POST body
