# --------------------------------
# building in this image
# --------------------------------
FROM ekidd/rust-musl-builder:latest as builder

# copying files
ADD --chown=rust:rust . ./

# Build with mounted cache
RUN cargo build --release --verbose

# --------------------------------
# copy the binary
# --------------------------------
FROM alpine:latest
EXPOSE 4723

# adding certs 
RUN apk --no-cache add ca-certificates

# copying the binary
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/actix-discord-demo \
    .
# copying other necessities
COPY ./static .
COPY .env ./

CMD ["./actix-discord-demo"]
