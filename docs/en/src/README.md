# Cuvée

*[Versão em português](../index.html)*

Cuvée runs **macOS in virtual machines on Linux**, with graphics
acceleration, on top of [reims-vgpu](https://github.com/steelbrain/reims-vgpu):
a virtual GPU for QEMU/KVM that carries macOS's Metal to your card's Vulkan
driver.

It installs QEMU with reims-vgpu, `reims-ctl` (which creates and runs the
VMs) and `reims-util`, prepares the host and checks that everything is ready.
It comes with the `cuvee` command and the **Cuvée** desktop app
(`cuvee-gui`).

## What to expect

| | Status |
|---|---|
| macOS 26 Tahoe | graphics acceleration (Metal) |
| macOS 10.13 to 15 | installs and boots, but **without acceleration**: the screen is slow |
| Host GPU | AMD, Intel and NVIDIA, through their Vulkan driver |
| Session | Wayland and X11 |
| Distros | Ubuntu 22.04 or newer, Debian 12 and 13, Fedora, openSUSE Tumbleweed, Arch |

Accelerating macOS 10.13 to 15 by handing a physical GPU to the VM is planned.

## What you need

- An x86-64 CPU with virtualization (Intel VT-x or AMD-V) enabled in the
  firmware (BIOS/UEFI).
- A GPU with a working Vulkan driver on Linux.
- 8 GB of RAM or more (a VM takes 4 GB by default) and about **64 GB free**
  on disk for the installer and the system.
- Internet access: macOS is downloaded straight from Apple's servers.

## Where to start

1. [Install Cuvée](install.md) the way your distro does it.
2. [Create your first VM](first-vm.md).
3. If something goes wrong, see [Troubleshooting](troubleshooting.md) or
   [report a bug](report-bug.md).

---

Cuvée is free software (GPL-3.0-or-later) and an independent project: it is
not affiliated with Apple or with the reims-vgpu project. macOS and Metal are
trademarks of Apple Inc. Cuvée does not distribute macOS.
