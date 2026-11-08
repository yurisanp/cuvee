# Installation

Pick **one** of the ways below. They all install the same things: the
`cuvee` command, the `cuvee-gui` app, QEMU with reims-vgpu (in
`/usr/lib/cuvee/qemu`, apart from your distro's QEMU), `reims-ctl` and
`reims-util`.

After installing, follow [Prepare the host](#prepare-the-host): that step is
the same for everyone.

## One-command installer (any distro)

```sh
curl -fsSL https://yurisanp.github.io/cuvee/install.sh | sh
```

The installer finds out your distro, installs the dependencies with its
package manager (it asks for your `sudo` password), downloads the prebuilt
Cuvée and checks the SHA-256 sums. Want to read the script first? Download it
and open it:

```sh
curl -fsSLO https://yurisanp.github.io/cuvee/install.sh
less install.sh
sh install.sh
```

## Ubuntu and Debian (.deb)

Works on Ubuntu 22.04, 24.04 or newer and on Debian 12 and 13.

```sh
cd /tmp
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee-runtime_amd64.deb
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee_amd64.deb
sudo apt install ./cuvee-runtime_amd64.deb ./cuvee_amd64.deb
```

Use `apt install ./file.deb` (with the `./`), not `dpkg -i`: apt fetches the
dependencies for you.

Vulkan driver:

| GPU | Package |
|---|---|
| AMD or Intel | `sudo apt install mesa-vulkan-drivers` |
| NVIDIA | the proprietary driver (`sudo ubuntu-drivers install` on Ubuntu; `nvidia-driver` from non-free on Debian) |

## Fedora (.rpm)

```sh
cd /tmp
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee-runtime.x86_64.rpm
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee.x86_64.rpm
sudo dnf install ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm
```

Vulkan driver: `sudo dnf install mesa-vulkan-drivers` (AMD and Intel). For
NVIDIA, `akmod-nvidia` from [RPM Fusion](https://rpmfusion.org/Howto/NVIDIA).

## openSUSE Tumbleweed (.rpm)

Download both `.rpm` files as for Fedora and install them with:

```sh
sudo zypper install --allow-unsigned-rpm ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm
```

Vulkan driver: `libvulkan_radeon` (AMD) or `libvulkan_intel` (Intel). For
NVIDIA, NVIDIA's official repository for openSUSE.

## Arch Linux

For now, from the repository (the AUR package comes later):

```sh
git clone https://github.com/yurisanp/cuvee
cd cuvee/packaging/reims-vgpu-git && makepkg -si   # builds QEMU: 20 to 40 minutes
cd ../cuvee-git && makepkg -si
```

Vulkan driver: `vulkan-radeon` (AMD), `vulkan-intel` (Intel) or
`nvidia-utils` (NVIDIA).

## Check the signature (optional)

Every release has a `SHA256SUMS` file signed with
[minisign](https://jedisct1.github.io/minisign/). The public key is
`cuvee.pub` at the root of the repository.

```sh
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/SHA256SUMS
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/SHA256SUMS.minisig
curl -fLO https://raw.githubusercontent.com/yurisanp/cuvee/main/cuvee.pub
minisign -Vm SHA256SUMS -p cuvee.pub
sha256sum -c --ignore-missing SHA256SUMS
```

The last two lines must finish without errors: the first says
`Signature and comment signature verified`, the second prints `OK` for each
file you downloaded.

## Prepare the host

As your normal user, **without** `sudo` (the command asks for the password
when it needs it):

```sh
cuvee setup-host
```

It loads the `udmabuf` module, adds the rule that lets the `kvm` group use
`/dev/udmabuf`, raises `vm.max_map_count` on NVIDIA GPUs and adds you to the
`kvm` group.

**Log out and back in** (or reboot) so the group takes effect. Then:

```sh
cuvee doctor
```

All green? Go to the [first VM](first-vm.md). Any `✗`? The doctor says what
to do; the common cases are in [Troubleshooting](troubleshooting.md).
