#!/bin/bash
set -e
set -o pipefail

# Install our deps.
sudo apt update -y && sudo apt install -y \
	ca-certificates \
	clang \
	cmake \
	curl \
	g++ \
	gcc \
	gcc-mingw-w64-i686 \
	gcc-mingw-w64 \
	libmpc-dev \
	libmpfr-dev \
	libgmp-dev \
	libssl-dev \
	libxml2-dev \
	mingw-w64 \
	wget \
	zlib1g-dev

# We need this for the version.
cargo install toml-cli

# Install cross.
cargo install cross

# Build osxcross.
#git clone https://github.com/jessfraz/osxcross
#cd osxcross
#wget -nc https://dl.oxide.computer/mac/sdk/MacOSX12.1.sdk.tar.xz
#mv MacOSX12.1.sdk.tar.xz tarballs/
#UNATTENDED=yes ./build.sh

#cd ../

# Update the cargo config for macos.
#echo "[target.x86_64-apple-darwin]" >> $HOME/.cargo/config
#echo "linker = \"x86_64-apple-darwin21.2-clang\"" >> $HOME/.cargo/config
#echo "ar = \"x86_64-apple-darwin21.2-ar\"" >> $HOME/.cargo/config
#echo >> $HOME/.cargo/config
#echo "[target.aarch64-apple-darwin]" >> $HOME/.cargo/config
#echo "linker = \"arm64-apple-darwin21.2-clang\"" >> $HOME/.cargo/config
#echo "ar = \"arm64-apple-darwin21.2-ar\"" >> $HOME/.cargo/config
#echo >> $HOME/.cargo/config
