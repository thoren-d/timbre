FROM mcr.microsoft.com/vscode/devcontainers/rust:1

# [Optional] Uncomment this section to install additional packages.
RUN curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | sudo bash

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends cmake libsdl2-dev git-lfs clang
