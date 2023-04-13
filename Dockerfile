FROM rust:latest AS build1

WORKDIR /kernel

# Copy the project to the docker container 
COPY . /kernel

# Build our /kernel (inside the container)
RUN rustup toolchain install nightly
RUN rustup component add rust-src --toolchain nightly-aarch64-unknown-linux-gnu
RUN rustup default nightly
RUN cargo build
# done in the rust container

# changed from debian:buster-slim due to dosfstools 4.2
FROM alpine:edge as build2

# Shell dependencies for the makefile
RUN apk -U upgrade
RUN apk add make binutils binutils-x86_64 parted dosfstools nasm
RUN rm -rf /var/cache/apk/*

WORKDIR /kernel_make

# Copy everything from the rust container to our new container. We only really need
# the makefile, scripts and the image in /target though
COPY --from=build1 /kernel /kernel_make

# make
RUN chmod +x *
RUN objcopy --info; objcopy -I elf64-x86-64 -O binary --binary-architecture=i386:x86-64
RUN make mbr.bin vbr.bin 
RUN ./makeimg_half.sh



