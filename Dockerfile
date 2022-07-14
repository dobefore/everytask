FROM rust:latest as builder
WORKDIR /usr/src/everydaytask
# copy from host to container
COPY . .
RUN cargo build --release  && cp ./target/release/task . && cargo clean

FROM debian:stable-slim as runner
# #RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/everydaytask/task /usr/local/bin/task
RUN chmod +x /usr/local/bin/task 
# # WORKDIR /app means, when you log into the shell of container，
# # you will be in the /app directory of the container by default.
# WORKDIR /app
# # https://linuxhint.com/dockerfile_volumes/
# # persist data with a named volume https://docs.docker.com/get-started/05_persisting_data/
# VOLUME /app
# COPY --from=builder /usr/src/everydaytask/scripts/ankisyncd.toml /app/ankisyncd.toml
CMD ["task"]
