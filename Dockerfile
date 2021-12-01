FROM debian:bookworm-slim as build

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get clean && apt-get update && apt-get install --no-install-recommends -y git wget curl cmake build-essential git libcos4-dev libomniorb4-dev libomnithread4-dev libzmq3-dev omniidl python3 pkg-config
RUN apt-get install -y --reinstall ca-certificates

# latest cmake
RUN git clone --depth 1 --branch v3.22.0 https://github.com/Kitware/CMake cmake
RUN mkdir /cmake/build
RUN cd /cmake/build && ../bootstrap && make -j$(nproc) && make install

# libzmq
RUN git clone --depth 1 --branch v4.2.0 https://github.com/zeromq/libzmq
RUN mkdir /libzmq/build
RUN cd /libzmq/build && cmake -DENABLE_DRAFTS=OFF -DWITH_DOC=OFF -DZMQ_BUILD_TESTS=OFF .. && make -j$(nproc) && make install

# cppzmq
RUN git clone --depth 1 --branch v4.7.1 https://github.com/zeromq/cppzmq
RUN mkdir /cppzmq/build
RUN cd /cppzmq/build && cmake -DCPPZMQ_BUILD_TESTS=OFF -DCMAKE_INSTALL_PREFIX=/usr/local .. && make -j$(nproc) && make install

# Tango IDL
RUN git clone --depth 1 https://gitlab.com/tango-controls/tango-idl
RUN cd tango-idl
RUN mkdir /tango-idl/build
RUN cd /tango-idl/build && cmake .. && make install

# cppTango
RUN git clone --depth 1 https://gitlab.com/tango-controls/cppTango
RUN mkdir /cppTango/build
RUN cd /cppTango/build && cmake .. && make -j$(nproc) && make install

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Compile tango-controls-tui
ADD . /tango-controls-tui
WORKDIR /tango-controls-tui
RUN /root/.cargo/bin/cargo build --release
RUN mv /tango-controls-tui/target/release/tango-controls-tui /usr/local/bin/

FROM debian:bookworm-slim
ENV LD_LIBRARY_PATH=/usr/local/lib
COPY --from=build /usr/local/lib /usr/local/lib
COPY --from=build /usr/local/bin /usr/local/bin
COPY --from=build /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu

WORKDIR /
CMD ["tango-controls-tui"]