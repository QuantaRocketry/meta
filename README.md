# Quanta meta

Monorepo for Quanta Rocketry's set of devices that operate together, their shared libraries, and the ground station.

| Path | Contents |
| --- | --- |
| `devices/` | One folder per device, each with its `firmware/` and `hardware/` |
| `libs/` | Shared library crates, including submodules |
| `apps/singularity/` | Tauri ground-station app (see its README) |

## Setup

Tools are managed with [mise](https://mise.jdx.dev).

System packages that `mise install` needs but can't provide, since `elf2flash`, `probe-rs-tools` and `espflash` build against libudev:

```bash
# Fedora
sudo dnf install systemd-devel

# Debian/Ubuntu
sudo apt install libudev-dev pkg-config
```

On Fedora Atomic, install it in a toolbox or with `rpm-ostree install systemd-devel` and reboot.

`openocd` and `gdb` are not available through mise. Install them with your system package manager if you debug over OpenOCD.

```bash
mise install
git submodule update --init --recursive
cargo check --workspace
```
