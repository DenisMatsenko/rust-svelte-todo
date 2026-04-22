FROM rustlang/rust:nightly

RUN apt-get update && apt-get install -y mold && rm -rf /var/lib/apt/lists/*
RUN rustup component add rustc-codegen-cranelift

WORKDIR /app

RUN cargo install cargo-watch --locked

ENV CARGO_TARGET_DIR=/app/target
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

CMD ["cargo", "watch", "-w", "src", "-w", "migrations", "-x", "run"]
