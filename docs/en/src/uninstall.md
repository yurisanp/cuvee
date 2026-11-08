# Updating and uninstalling

## Updating

| Installed with | To update |
|---|---|
| one-command installer | `cuvee update` |
| `.deb` | download both new `.deb` files and run `sudo apt install ./cuvee-runtime_amd64.deb ./cuvee_amd64.deb` |
| `.rpm` (Fedora) | download both new `.rpm` files and run `sudo dnf install ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm` |
| `.rpm` (openSUSE) | the same with `sudo zypper install --allow-unsigned-rpm ...` |
| Arch (PKGBUILD) | `git pull` in the repository and `makepkg -si` in both packages |

Updating does not touch your VMs.

## Uninstalling

```sh
sudo apt remove cuvee cuvee-runtime      # Ubuntu and Debian
sudo dnf remove cuvee cuvee-runtime      # Fedora
sudo zypper remove cuvee cuvee-runtime   # openSUSE
sudo pacman -R cuvee-git reims-vgpu-git  # Arch
```

If you used the one-command installer, run `cuvee uninstall`.

VMs are **not** deleted: they stay in `~/.local/share/reims-ctl/vms`. To
delete them too:

```sh
rm -rf ~/.local/share/reims-ctl
```

What `cuvee setup-host` changed stays on the system: your user in the `kvm`
group (`sudo gpasswd -d $USER kvm` to leave it) and, on NVIDIA, the file
`/etc/sysctl.d/70-cuvee.conf`.
