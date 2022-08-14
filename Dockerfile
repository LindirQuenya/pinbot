
FROM rust:slim-bullseye AS builder

# Copy the needed files over.
COPY . /pinbot
# Create and move to a new directory for our project.
WORKDIR /pinbot

# Build the binary.
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM debian:bullseye-slim AS main

COPY --from=builder /pinbot/target/x86_64-unknown-linux-gnu/release/pinbot /pinbot

CMD /pinbot
