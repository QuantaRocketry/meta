# Singularity

[![My Skills](https://skillicons.dev/icons?i=tauri,rust,ts,tailwind)](https://skillicons.dev)

A device manager designed for interacting with Quanta Rocketry's systems (and any other serial devices).

## Usage

This project uses a mix of a Nix flake and justfile.

```sh
nix develop
```

```sh
just install
just dev
```

## Installation

### Debian-Based

```sh
sudo apt install \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  librsvg2-dev \
  libsoup-3.0-dev \
  libayatana-appindicator3-dev \
  libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev
```

### Fedora

```sh
sudo dnf group install c-development
sudo dnf install \
  curl \
  wget \
  file \
  libxdo-devel \
  openssl-devel \
  librsvg2-devel \
  libsoup3-devel \
  libappindicator-gtk3-devel \
  webkit2gtk4.1-devel \
  javascriptcoregtk4.1-devel \
  systemd-devel
```

## Screenshots

![Cover Image](./public/cover.png)

![Map Image](./public/map.png)
