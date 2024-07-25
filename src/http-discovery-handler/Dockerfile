FROM --platform=$BUILDPLATFORM tonistiigi/xx:master AS xx

FROM  --platform=$BUILDPLATFORM rust:1.73-slim-bookworm AS build
RUN rustup component add rustfmt
RUN apt-get update && apt-get install -y clang lld protobuf-compiler pkg-config mmdebstrap wget
COPY --from=xx / /
ARG TARGETPLATFORM

# See https://github.com/tonistiigi/xx/issues/108
RUN sed -i -E 's/xx-clang --setup-target-triple/XX_VENDOR=\$vendor ARM_TARGET_ARCH="" xx-clang --setup-target-triple/' $(which xx-cargo) && \
    sed -i -E 's/\$\(xx-info\)-/\$\(XX_VENDOR=\$vendor ARM_TARGET_ARCH="" xx-info\)-/g' $(which xx-cargo)

# Generate minimal runtime environment
RUN mmdebstrap --architectures=$(xx-info debian-arch) --include=libc6,libssl3,libudev1,libv4l-0,busybox --variant=extract bookworm /installroot
RUN mkdir -p /installroot/usr/local/bin /build/bin && for tool in sh uniq tail sort grep cut; do ln -s /bin/busybox /installroot/bin/$tool; done
RUN xx-apt-get install -y xx-c-essentials libssl-dev libudev-dev libv4l-dev pkg-config ca-certificates openssl

WORKDIR /dh
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && \
    # find ./src -name "*.rs" -type f -delete && \
    rm ./src/*.rs && \
    DEP_PATH=`echo "./target/release/deps/akri-http-discovery-handler*" | tr - _` && \
    rm $DEP_PATH

FROM scratch
ENV RUST_BACKTRACE=FULL
LABEL org.opencontainers.image.source https://github.com/project-akri/akri
COPY --from=build /installroot /
COPY --from=build /dh/target/release/akri-http-discovery-handler /akri-http-discovery-handler

ENTRYPOINT ["/akri-http-discovery-handler"]