# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.238.0/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends lld clang postgresql-client neovim

USER 1000
# Install cargo-binstall
#RUN mkdir -p /usr/local/cargo/bin
RUN wget -qO - "https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz" | tar zxvf - -C /usr/local/cargo/bin
# Install some cargo tools
RUN cargo binstall -y cargo-deny
RUN cargo binstall -y cargo-edit
RUN cargo binstall -y cargo-whatfeatures
RUN cargo binstall -y cargo-nextest

# Can't use binstall due to unique requirements
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

# Install Act tool to run GH actions locally
RUN curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
