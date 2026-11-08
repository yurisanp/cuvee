# Cuvée

VMs de macOS no Arch Linux, com aceleração gráfica, em cima do
[reims-vgpu](https://github.com/steelbrain/reims-vgpu).

O Cuvée empacota o reims-vgpu (a GPU virtual, o QEMU dele, o `reims-ctl` e o
`reims-util`) para o Arch, prepara o host e confere se está tudo pronto. Quem
cria e roda as VMs é o `reims-ctl`; o comando `cuvee` repassa para ele tudo o
que não é dele.

> O Cuvée é um projeto independente. Não tem relação com a Apple, com o
> projeto reims-vgpu nem com o OSX-REIMS.

## O que funciona

| | Situação |
|---|---|
| macOS 26 Tahoe | com aceleração gráfica (Metal) |
| macOS 10.13 a 15 | instala e liga **sem aceleração**; com uma segunda GPU passada para a VM, roda com Metal ([GPU passthrough](docs/passthrough.md), beta) |
| GPUs do host | AMD (RADV), Intel (ANV) e NVIDIA, por Vulkan |
| Sessão | Wayland e X11 |

Para acelerar o macOS 10.13 a 15, passe uma GPU física para a VM (VFIO):
precisa de uma segunda GPU, de preferência uma AMD Polaris (RX 570/580). O
passo a passo está em [docs/passthrough.md](docs/passthrough.md):

```sh
cuvee passthrough list --macos sequoia   # quais GPUs servem e o que falta
cuvee passthrough setup-host 0c:00       # vfio-pci, udev e limites (sudo)
cuvee passthrough attach mac15 0c:00     # a VM usa a GPU no próximo boot
```

O subcomando `cuvee passthrough` entra no `cuvee` junto com o CLI em Rust da
v0.2; no aplicativo, a escolha da GPU já está no passo Hardware.

## Instalação

Ainda não está no AUR. Por enquanto, a partir deste repositório:

```sh
git clone https://github.com/yurisanp/cuvee
cd cuvee/packaging/reims-vgpu-git && makepkg -si   # demora: compila o QEMU e a GPU virtual
cd ../cuvee-git && makepkg -si
```

Instale também o driver Vulkan da sua GPU: `vulkan-radeon` (AMD),
`vulkan-intel` (Intel) ou `nvidia-utils` (NVIDIA).

Depois, como seu usuário (sem `sudo`):

```sh
cuvee setup-host   # carrega o udmabuf e põe você no grupo kvm
cuvee doctor       # confere KVM, udmabuf, GPU, Vulkan e sessão
```

Saia e entre de novo na sessão depois do `setup-host`.

## Uso

```sh
cuvee                                   # menu interativo
cuvee create mac --macos tahoe --start  # cria e liga uma VM com o macOS 26
cuvee list                              # lista as VMs
```

Na primeira vez, a VM abre no macOS Recovery: no Utilitário de Disco, apague
o disco "QEMU HARDDISK Media" como APFS e escolha "Reinstalar macOS".

**Passo obrigatório no macOS 26:** sem ele, o desktop não aparece. Siga o item
3 do [README do reims-vgpu](https://github.com/steelbrain/reims-vgpu#x86-64-guest-on-linux-kvm)
(definir `METAL_ERROR_MODE` pelo Terminal do Recovery).

Para soltar o teclado e o mouse da janela da VM: **Ctrl+Alt+Super**.

## Aplicativo (cuvee-gui)

Interface em Rust ([iced](https://iced.rs)) para criar, ligar e acompanhar as
VMs, com diagnóstico do host, dicas por versão do macOS e um guia do que
esperar de cada uma. Em português e inglês; respeita a preferência de
"reduzir movimento".

```sh
cargo run --release -p cuvee-gui
```

Precisa do `reims-ctl` no PATH (o pacote `reims-vgpu-git` já o instala). Os
ajustes ficam em `~/.config/cuvee/settings.toml`.

## Estrutura

| Caminho | O que é |
|---|---|
| `bin/cuvee` | o comando `cuvee` |
| `crates/cuvee-core` | núcleo: diagnóstico, catálogo de macOS, VMs, traduções |
| `crates/cuvee-gui` | o aplicativo |
| `crates/cuvee-cli/src/cmd/passthrough.rs` | o subcomando `cuvee passthrough` |
| `docs/passthrough.md` | GPU passthrough: GPUs que servem e passo a passo |
| `host/` | regra udev e módulo carregado no boot (udmabuf) |
| `packaging/reims-vgpu-git` | PKGBUILD do reims-vgpu, QEMU, reims-ctl e reims-util |
| `packaging/cuvee-git` | PKGBUILD do Cuvée |

O QEMU do reims-vgpu fica em `/usr/lib/cuvee/qemu`, separado do QEMU do Arch.

## Licença

GPL-3.0-or-later. O reims-vgpu e o reims-util são GPL-2.0-or-later OR
LGPL-3.0-or-later; o QEMU é GPL-2.0.

macOS e Metal são marcas da Apple Inc. O macOS é baixado direto da Apple e não
é distribuído por este projeto.
