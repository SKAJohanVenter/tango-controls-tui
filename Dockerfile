FROM ubuntu:latest as build

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get clean && apt-get update && apt-get install --no-install-recommends -y git curl cmake build-essential omniidl libomniorb4-dev libcos4-dev libomnithread4-dev libzmq3-dev python3
RUN apt-get install -y --reinstall ca-certificates

# tango-idl
RUN git clone  https://gitlab.com/tango-controls/tango-idl /tango-idl
RUN mkdir /tango-idl/build && cd /tango-idl/build && cmake .. && make install

# cppTango
RUN git clone  https://gitlab.com/tango-controls/cppTango /cppTango
RUN mkdir /cppTango/build
RUN cd /cppTango/build && cmake .. && make -j$(nproc) && make install

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Compile tango-controls-tui
ADD . /tango-controls-tui
WORKDIR /tango-controls-tui
RUN /root/.cargo/bin/cargo build --release
RUN mv /tango-controls-tui/target/release/tango-controls-tui /usr/local/bin/

FROM ubuntu:latest
ENV LD_LIBRARY_PATH=/usr/local/lib
COPY --from=build /usr/local/lib /usr/local/lib
COPY --from=build /usr/local/bin /usr/local/bin
COPY --from=build /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu

WORKDIR /
CMD ["tango-controls-tui"]