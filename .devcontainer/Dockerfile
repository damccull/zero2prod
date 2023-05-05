# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.238.0/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends lld clang postgresql-client

USER 1000
# Install some cargo tools
RUN cargo install cargo-audit
RUN cargo install cargo-edit
RUN cargo install cargo-whatfeatures
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

# Install Act tool to run GH actions locally
RUN curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
