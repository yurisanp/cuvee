# Atualizar e desinstalar

## Atualizar

| Instalado por | Para atualizar |
|---|---|
| instalador de um comando | `cuvee update` |
| `.deb` | baixe os dois `.deb` novos e rode `sudo apt install ./cuvee-runtime_amd64.deb ./cuvee_amd64.deb` |
| `.rpm` (Fedora) | baixe os dois `.rpm` novos e rode `sudo dnf install ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm` |
| `.rpm` (openSUSE) | o mesmo com `sudo zypper install --allow-unsigned-rpm ...` |
| Arch (PKGBUILD) | `git pull` no repositório e `makepkg -si` nos dois pacotes |

Atualizar não mexe nas suas VMs.

## Desinstalar

```sh
sudo apt remove cuvee cuvee-runtime      # Ubuntu e Debian
sudo dnf remove cuvee cuvee-runtime      # Fedora
sudo zypper remove cuvee cuvee-runtime   # openSUSE
sudo pacman -R cuvee-git reims-vgpu-git  # Arch
```

Se você usou o instalador de um comando, rode `cuvee uninstall`.

As VMs **não** são apagadas: elas continuam em
`~/.local/share/reims-ctl/vms`. Para apagá-las também:

```sh
rm -rf ~/.local/share/reims-ctl
```

O que o `cuvee setup-host` mudou e fica no sistema: o seu usuário no grupo
`kvm` (`sudo gpasswd -d $USER kvm` para sair) e, em NVIDIA, o arquivo
`/etc/sysctl.d/70-cuvee.conf`.
