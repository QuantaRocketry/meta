# Singularity

[![My Skills](https://skillicons.dev/icons?i=tauri,rust,ts,tailwind)](https://skillicons.dev)

A device manager designed for interacting with Quanta Rocketry's systems (and any other serial devices).

## Usage

Tasks are defined in `mise.toml`. Run `mise tasks` to list them.

```sh
mise run install
mise run dev
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
