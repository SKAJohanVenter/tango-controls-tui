FROM ubuntu:20.04 as build

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get clean && apt-get update && apt-get install --no-install-recommends -y curl libtango-dev build-essential
RUN apt-get install -y --reinstall ca-certificates

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Compile tango-controls-tui
ADD . /tango-controls-tui
WORKDIR /tango-controls-tui
RUN /root/.cargo/bin/cargo build --release
RUN mv /tango-controls-tui/target/release/tango-controls-tui /usr/local/bin/tango-controls-tui

FROM ubuntu:20.04
COPY --from=build /usr/local/bin /usr/local/bin
COPY --from=build /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu

WORKDIR /
CMD ["tango-controls-tui"]