FROM ubuntu:latest
WORKDIR /
EXPOSE 1334
COPY /target/debug/rsk-simulation /rsk-simulation
COPY /target/wasm32-unknown-unknown/debug/rsk-simulation.wasm /target/wasm32-unknown-unknown/debug/rsk-simulation.wasm
COPY /www /www
CMD ./rsk-simulation
