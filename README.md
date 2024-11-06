# RSK Simulation
This is an unofficial simulation for the [RSK / SCT / SSL Junior Robocup league](https://robot-soccer-kit.github.io/) written in Rust. It is still in development. It aims at providing both a visual interface and a fast headless simulation (maybe for AI?).

## Usage
You need to install git, rust and rust's targets depending on which mode you want to use. If you encounter a problem at any step please open an issue.

1. Clone the repository
`
git clone https://github.com/Arkitu/rsk-simulation.git
`

2. Move into the repository
`
cd rsk-simulation
`

3. Depending on the mode you want :
   - Native mode : `cargo run`
   - Server/client mode (supports multiple clients) : `cargo build --target wasm32-unknown-unknown --no-default-features --features http_client && cargo run --no-default-features --features http_server`
   - Server/client alternative mode (simulation, game controller and referee on server side. slower) : `cargo build --target wasm32-unknown-unknown --no-default-features --features alternative_http_client && cargo run --no-default-features --features alternative_http_server`

### Server mode
By default, the server is only working on `localhost` (127.0.0.1). You can change that by setting the ip you want with the environment variable `SERVER_ADDRESS`. You can also make it an https server with `HTTPS=true`

## Git structure
- master: The release branch where everything works
- dev: The branch where the next update is being coded

## Contribution
You can help by creating a branch at you name from the `dev` branch and starting coding.
