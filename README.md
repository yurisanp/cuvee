# Cuvée

VMs de macOS no Linux, com aceleração gráfica, em cima do
[reims-vgpu](https://github.com/steelbrain/reims-vgpu).

O Cuvée instala o reims-vgpu (a GPU virtual, o QEMU dele, o `reims-ctl` e o
`reims-util`) em qualquer distro com um comando, prepara o host e confere se
está tudo pronto. Quem
cria e roda as VMs é o `reims-ctl`; o comando `cuvee` repassa para ele tudo o
que não é dele.

> O Cuvée é um projeto independente. Não tem relação com a Apple, com o
> projeto reims-vgpu nem com o OSX-REIMS.

## O que funciona

| | Situação |
|---|---|
| macOS 26 Tahoe | com aceleração gráfica (Metal) |
| macOS 10.13 a 15 | instala e liga, mas **sem aceleração** (a tela fica lenta) |
| GPUs do host | AMD (RADV), Intel (ANV) e NVIDIA, por Vulkan |
| Sessão | Wayland e X11 |

Acelerar o macOS 10.13 a 15 com uma GPU física passada para a VM (VFIO) está
nos planos.

## Instalação

Em qualquer distro (Ubuntu, Debian, Fedora, Arch, openSUSE e derivadas), como
seu usuário, sem `sudo`:

```sh
curl -fsSL https://yurisanp.github.io/cuvee/install.sh | sh
```

O instalador baixa o comando `cuvee` (um binário estático) da última release,
confere o SHA-256 e roda `cuvee install`, que:

1. instala os pacotes que faltam com o gerenciador da sua distro (apt, dnf,
   pacman ou zypper; pede `sudo`), incluindo o driver Vulkan da Mesa em GPUs
   AMD e Intel;
2. baixa o QEMU da Reims vGPU, o `reims-ctl` e o `reims-util` já compilados
   para `~/.local/share/cuvee/runtime`;
3. põe o `cuvee` e o app `cuvee-gui` em `~/.local/bin` (com atalho no menu);
4. prepara o host: módulo `udmabuf`, regra udev, grupo `kvm` e, em NVIDIA,
   `vm.max_map_count`;
5. roda `cuvee doctor`.

Saia e entre de novo na sessão depois da primeira instalação (por causa do
grupo `kvm`). Em GPU NVIDIA, instale antes o driver da sua distro
(`nvidia-utils` no Arch; no Ubuntu, `ubuntu-drivers install`; no Fedora, o
`akmod-nvidia` do RPM Fusion).

Opções: `... | sh -s -- --yes` responde sim ao gerenciador de pacotes;
`CUVEE_VERSION=v0.2.0` instala uma release específica. Depois:

```sh
cuvee update      # atualiza o cuvee, o runtime e o app
cuvee uninstall   # remove o que o install pôs na sua pasta pessoal (as VMs ficam)
```

Prefere não rodar um script da internet? Baixe `cuvee-x86_64-linux-musl` e
`SHA256SUMS` da [página de releases](https://github.com/yurisanp/cuvee/releases),
confira com `sha256sum -c --ignore-missing SHA256SUMS` e rode
`./cuvee-x86_64-linux-musl install`.

### Arch Linux, pelos PKGBUILDs

Ainda não está no AUR. A partir deste repositório:

```sh
git clone https://github.com/yurisanp/cuvee
cd cuvee/packaging/reims-vgpu-git && makepkg -si   # demora: compila o QEMU e a GPU virtual
cd ../cuvee-git && makepkg -si
cuvee setup-host && cuvee doctor
```

Instalado assim, o Cuvée é atualizado pelo pacman, e `cuvee install`/`update`
recusam mexer nele.

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

O instalador já o põe no menu de aplicativos. Para rodar a partir do código:
`cargo run --release -p cuvee-gui`. Os ajustes ficam em
`~/.config/cuvee/settings.toml`.

## Estrutura

| Caminho | O que é |
|---|---|
| `crates/cuvee-cli` | o comando `cuvee` (Rust, binário estático) |
| `install/install.sh` | o instalador `curl \| sh` |
| `packaging/runtime` | compila o QEMU, o reims-ctl e o reims-util das releases |
| `crates/cuvee-core` | núcleo: diagnóstico, catálogo de macOS, VMs, traduções |
| `crates/cuvee-gui` | o aplicativo |
| `host/` | regra udev e módulo carregado no boot (udmabuf) |
| `packaging/reims-vgpu-git` | PKGBUILD do reims-vgpu, QEMU, reims-ctl e reims-util |
| `packaging/cuvee-git` | PKGBUILD do Cuvée |

O QEMU da Reims vGPU fica em `~/.local/share/cuvee/runtime` (instalador) ou em
`/usr/lib/cuvee/qemu` (pacotes), separado do QEMU da distro.

## Licença

GPL-3.0-or-later. O reims-vgpu e o reims-util são GPL-2.0-or-later OR
LGPL-3.0-or-later; o QEMU é GPL-2.0.

macOS e Metal são marcas da Apple Inc. O macOS é baixado direto da Apple e não
é distribuído por este projeto.
