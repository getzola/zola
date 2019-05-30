FROM bitnami/minideb AS builder
RUN install_packages python-pip curl tar python-setuptools rsync binutils
RUN pip install dockerize
RUN mkdir -p /workdir
WORKDIR /workdir
ENV DOCKER_TAG v0.7.0
RUN curl -L https://github.com/getzola/zola/releases/download/$DOCKER_TAG/zola-$DOCKER_TAG-x86_64-unknown-linux-gnu.tar.gz | tar xz
RUN mv zola /usr/bin
RUN dockerize -n  -o /workdir  /usr/bin/zola


FROM scratch
COPY --from=builder /workdir .
ENTRYPOINT [ "/usr/bin/zola" ]
