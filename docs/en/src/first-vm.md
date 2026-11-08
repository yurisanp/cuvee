# First VM

## With the app

Open **Cuvée** from the app menu (or run `cuvee-gui`). In **New machine**,
pick the macOS version, memory, cores and disk; the review step shows the
exact command it will run. Then start the VM from the **Dashboard**.

## From the terminal

```sh
cuvee create mac --macos tahoe --start
```

This creates a VM named `mac` with macOS 26 Tahoe and starts it. Other
version names: `cuvee help create` lists them all. To see your VMs:
`cuvee list`.

## Install macOS

The first time, the VM opens in **macOS Recovery**:

1. Open **Disk Utility**, select the **QEMU HARDDISK Media** disk, click
   **Erase** and choose the **APFS** format.
2. Close Disk Utility and choose **Reinstall macOS**.
3. Wait: the installer downloads about 15 GB from Apple and restarts a few
   times.

## Required step on macOS 26

Without it, the desktop does not show up after installation. Follow item 3
of the [reims-vgpu README](https://github.com/steelbrain/reims-vgpu#x86-64-guest-on-linux-kvm):
from the Recovery's Terminal, set `METAL_ERROR_MODE` as it describes.

## Tips

- **Release keyboard and mouse** from the VM window: **Ctrl+Alt+Super**.
- **Shut down**: from the Apple menu inside macOS, or `cuvee stop mac`.
- **Change memory, cores or resolution**: `cuvee set mac ...` while the VM
  is off (`cuvee help set` shows the options).
- VMs live in `~/.local/share/reims-ctl/vms`.

## The Mac's identity (SMBIOS)

Each VM comes with a Mac of its own: model, serial number, MLB, UUID and ROM.
There is no need to run GenSMBIOS or macserial: reims-util makes the values
when the VM is created, and the app shows the identity in the "New machine"
review, with a **Make another** button.

- See a VM's identity: `cuvee smbios show mac`.
- Give it a new one (VM shut down): `cuvee smbios new mac`. macOS then sees
  another Mac, so iCloud and iMessage ask you to sign in again.
- Make an identity with no VM: `cuvee smbios generate --macos sequoia`.

To check a serial at <https://checkcoverage.apple.com>, "unable to check
coverage" is what it should get: it belongs to no Mac Apple sold.
