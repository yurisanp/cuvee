# Cuvée

*[English version](en/index.html)*

O Cuvée roda **macOS em máquinas virtuais no Linux**, com aceleração gráfica,
em cima do [reims-vgpu](https://github.com/steelbrain/reims-vgpu): uma GPU
virtual para QEMU/KVM que leva o Metal do macOS para o Vulkan da sua placa.

Ele instala o QEMU com o reims-vgpu, o `reims-ctl` (que cria e liga as VMs) e
o `reims-util`, prepara o host e confere se está tudo pronto. Vem com o
comando `cuvee` e com o aplicativo **Cuvée** (`cuvee-gui`).

## O que esperar

| | Situação |
|---|---|
| macOS 26 Tahoe | com aceleração gráfica (Metal) |
| macOS 10.13 a 15 | instala e liga, mas **sem aceleração**: a tela fica lenta |
| GPU do host | AMD, Intel e NVIDIA, pelo driver Vulkan |
| Sessão | Wayland e X11 |
| Distros | Ubuntu 22.04 ou mais novo, Debian 12 e 13, Fedora, openSUSE Tumbleweed, Arch |

Acelerar o macOS 10.13 a 15 passando uma GPU física para a VM está nos planos.

## Do que você precisa

- Um processador x86-64 com virtualização (Intel VT-x ou AMD-V) ligada no
  firmware (BIOS/UEFI).
- Uma GPU com driver Vulkan funcionando no Linux.
- 8 GB de RAM ou mais (a VM usa 4 GB por padrão) e uns **64 GB livres** em
  disco para o instalador e o sistema.
- Internet: o macOS é baixado direto dos servidores da Apple.

## Por onde começar

1. [Instale o Cuvée](instalacao.md) pelo jeito da sua distro.
2. [Crie sua primeira VM](primeira-vm.md).
3. Se algo der errado, veja [Problemas comuns](problemas.md) ou
   [relate um bug](relatar-bug.md).

---

O Cuvée é software livre (GPL-3.0-or-later) e um projeto independente: não
tem relação com a Apple nem com o projeto reims-vgpu. macOS e Metal são
marcas da Apple Inc. O macOS não é distribuído pelo Cuvée.
