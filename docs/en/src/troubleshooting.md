# Troubleshooting

Always start with `cuvee doctor`: it checks KVM, udmabuf, GPU, Vulkan, the
graphical session and the install, and says what to do for each `✗`. If your
problem is not here, [report a bug](report-bug.md).

## `/dev/kvm does not exist`

Virtualization is off or the KVM module did not load.

1. Turn virtualization on in the firmware (BIOS/UEFI): **Intel VT-x**
   (sometimes "Intel Virtualization Technology") or **AMD-V/SVM**.
2. Load the module: `sudo modprobe kvm_intel` (Intel) or
   `sudo modprobe kvm_amd` (AMD).

Is your Linux itself inside a VM? Then the outer hypervisor needs nested
virtualization enabled.

## No access to `/dev/kvm` or `/dev/udmabuf`

You are not in the `kvm` group, or you joined it and have not logged out yet.

```sh
cuvee setup-host
```

Then **log out and back in**. To check: `id -nG` must list `kvm`.

## `/dev/udmabuf does not exist`

```sh
sudo modprobe udmabuf
```

If it says `Module udmabuf not found`, your kernel was built without
`CONFIG_UDMABUF` (some custom kernels are). Use your distro's stock kernel:
Ubuntu, Debian, Fedora, openSUSE and Arch all ship the module.

## Vulkan does not work (`vulkaninfo failed`)

Install your GPU's Vulkan driver (tables in [Installation](install.md)) and
test with:

```sh
vulkaninfo --summary
```

The `Devices` list must show your real GPU, not only `llvmpipe` (software
rendering). On laptops with two GPUs both show up; `cuvee gpus` shows which
ones a VM can use.

## NVIDIA: VM slow or freezing at high resolutions

On NVIDIA, reims-vgpu needs a high `vm.max_map_count`. `cuvee setup-host`
already sets it (in `/etc/sysctl.d/70-cuvee.conf`). Check with
`sysctl vm.max_map_count`: it should be `1048576` or more. The `nvidia-open`
driver is recommended.

## macOS 26: installation finishes and the desktop never shows up

The `METAL_ERROR_MODE` step is missing. See
[Required step on macOS 26](first-vm.md#required-step-on-macos-26).

## macOS 10.13 to 15 is very slow

That is expected: reims-vgpu's graphics acceleration only exists on macOS 26.
Older versions run on a plain framebuffer, without Metal. For real use,
prefer macOS 26.

## Mouse and keyboard are stuck in the window

Press **Ctrl+Alt+Super** to release them.

## Window problems on Wayland

Cuvée works on Wayland and X11. If the VM window does not open, flickers or
does not take the keyboard on Wayland, try running it through XWayland:

```sh
GDK_BACKEND=x11 cuvee start mac
```

And tell us in the [bug report](report-bug.md) which desktop you use (GNOME,
KDE, Hyprland...).

## Low disk space

An installed macOS plus its installer takes 40 to 60 GB. VMs live in
`~/.local/share/reims-ctl/vms`. To free space, delete VMs you do not use with
`cuvee delete NAME` (it deletes its disk too).

## Dependency errors installing the .deb or .rpm

Messages like `Depends: libsomething but it is not installable` mean your
distro version is not one of the supported ones ([Installation](install.md)),
or a standard repository is disabled (on Ubuntu, `universe`:
`sudo add-apt-repository universe`). If your version is supported,
[report a bug](report-bug.md) with the full message.

## `cuvee update` refuses to update

If you installed the `.deb` or `.rpm`, your distro's package manager does the
updates. See [Updating and uninstalling](uninstall.md).

## See what the VM is doing

```sh
cuvee logs mac
```

Prints the end of the QEMU log. GPU errors usually show up there.
