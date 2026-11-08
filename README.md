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

### Pacotes .deb e .rpm

Para quem prefere o gerenciador da distro. Cada release traz dois pacotes:
`cuvee` (o comando, o app e a preparação do host) e `cuvee-runtime` (o QEMU
da Reims vGPU em `/usr/lib/cuvee/qemu`, o `reims-ctl` e o `reims-util`).

```sh
# Ubuntu 22.04+ e Debian 12/13
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee-runtime_amd64.deb
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee_amd64.deb
sudo apt install ./cuvee-runtime_amd64.deb ./cuvee_amd64.deb

# Fedora (no openSUSE: sudo zypper install --allow-unsigned-rpm ...)
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee-runtime.x86_64.rpm
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee.x86_64.rpm
sudo dnf install ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm
```

Depois, `cuvee setup-host`, sair e entrar na sessão e `cuvee doctor`.

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

## Documentação e bugs

Guia completo em português e inglês: <https://yurisanp.github.io/cuvee/>
(instalação por distro, primeira VM, problemas comuns). Achou um bug? Rode
`cuvee report NOME_DA_VM` e anexe os arquivos que ele gera a uma
[issue](https://github.com/yurisanp/cuvee/issues/new).

## Estrutura

| Caminho | O que é |
|---|---|
| `crates/cuvee-cli` | o comando `cuvee` (Rust, binário estático) |
| `install/install.sh` | o instalador `curl \| sh` |
| `packaging/runtime` | compila o QEMU, o reims-ctl e o reims-util das releases |
| `crates/cuvee-core` | núcleo: diagnóstico, catálogo de macOS, VMs, traduções |
| `crates/cuvee-gui` | o aplicativo |
| `crates/cuvee-cli/src/cmd/passthrough.rs` | o subcomando `cuvee passthrough` |
| `docs/passthrough.md` | GPU passthrough: GPUs que servem e passo a passo |
| `host/` | regra udev e módulo carregado no boot (udmabuf) |
| `packaging/reims-vgpu-git` | PKGBUILD do reims-vgpu, QEMU, reims-ctl e reims-util |
| `packaging/cuvee-git` | PKGBUILD do Cuvée |
| `packaging/native` | pacotes `.deb` e `.rpm` (nfpm) e o teste de instalação em 7 distros |
| `docs/pt-BR`, `docs/en` | o site de documentação (mdBook) |

O QEMU da Reims vGPU fica em `~/.local/share/cuvee/runtime` (instalador) ou em
`/usr/lib/cuvee/qemu` (pacotes), separado do QEMU da distro.

## Licença

GPL-3.0-or-later. O reims-vgpu e o reims-util são GPL-2.0-or-later OR
LGPL-3.0-or-later; o QEMU é GPL-2.0.

macOS e Metal são marcas da Apple Inc. O macOS é baixado direto da Apple e não
é distribuído por este projeto.
