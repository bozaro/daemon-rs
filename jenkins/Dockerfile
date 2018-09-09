FROM rust:1.28-stretch

RUN apt-get update
RUN apt-get install -y git curl
RUN rustup target add i686-pc-windows-gnu
RUN rustup target add x86_64-pc-windows-gnu
RUN apt-get install -y gcc-mingw-w64

COPY cargo.config /.cargo/config
COPY . /root/test/
WORKDIR /root/test/

CMD ["/bin/bash"]
