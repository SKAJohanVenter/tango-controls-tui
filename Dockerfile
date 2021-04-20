FROM gcc:latest

RUN apt-get update && apt-get install -y git curl cmake build-essential omniidl libomniorb4-dev libcos4-dev libomnithread4-dev libzmq3-dev python3

# tango-idl
WORKDIR /
RUN git clone  https://gitlab.com/tango-controls/tango-idl
WORKDIR /tango-idl
RUN mkdir build
WORKDIR /build
RUN cmake ..
RUN make install

ENV LD_LIBRARY_PATH=/usr/local/lib

# cppTango
WORKDIR /
RUN git clone  https://gitlab.com/tango-controls/cppTango
WORKDIR /cppTango
RUN mkdir build
WORKDIR /build
RUN cmake ..
RUN make -j$(nproc)
RUN make install

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Compile tango-controls-tui
RUN mkdir /tango-controls-tui
ADD . /tango-controls-tui
WORKDIR /tango-controls-tui
RUN /root/.cargo/bin/cargo build --release
RUN mv /tango-controls-tui/target/release/tango_controls_tui /usr/local/bin
CMD tango_controls_tui