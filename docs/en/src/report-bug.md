# Reporting a bug

First, check whether the problem is in [Troubleshooting](troubleshooting.md)
or in the [open issues](https://github.com/yurisanp/cuvee/issues).

## 1. Make the report

```sh
cuvee report mac
```

Replace `mac` with the name of the VM that has the problem, or run just
`cuvee report` if the problem is not about a VM. It writes two files to your
Downloads folder (or your home folder):

- `cuvee-report-DATE.txt`: distro, kernel, CPU, GPUs and drivers, graphical
  session, installed versions, the result of each `cuvee doctor` check,
  `vulkaninfo --summary` and the list of VMs.
- a `.zip` from `reims-ctl report`, with the VM's logs and settings.

The text leaves out personal data: your home folder's path becomes `~`, your
user name becomes `<user>` and the computer's name becomes `<host>`. Of the
kernel command line, only the IOMMU and VFIO parameters go in. Even so, open
the file and look before you send it.

## 2. Open an issue

At [github.com/yurisanp/cuvee/issues/new](https://github.com/yurisanp/cuvee/issues/new),
tell us:

- what you did, step by step;
- what you expected to happen;
- what happened (the exact error message, or a screenshot).

Drag the `.txt` and the `.zip` into the issue box. English or Portuguese are
both fine.
