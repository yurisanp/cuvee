# GPU passthrough (VFIO)

A Reims vGPU só acelera o macOS 26. Do macOS 10.13 ao 15, a única forma de ter
Metal é entregar à VM uma GPU física inteira, que o macOS reconhece com o driver
dele. O Cuvée faz isso com o `vfio-pci` do kernel e o `reims-ctl`.

É recurso de desktop: precisa de **duas GPUs**. A que desenha a tela do Linux
fica com o Linux; a outra vai para a VM, com um monitor ligado nela. Notebook
com uma GPU só não serve.

## Quais GPUs

| GPU | 10.13 | 10.14 | 10.15 | 11 | 12 | 13 a 15 |
|---|---|---|---|---|---|---|
| AMD Polaris (RX 460 a 590) | nativa | nativa | nativa | nativa | nativa | nativa |
| AMD Vega 56/64 | nativa | nativa | nativa | nativa | nativa | nativa |
| AMD Radeon VII | não | nativa | nativa | nativa | nativa | nativa |
| AMD RX 5500/5600/5700 | não | não | nativa¹ | nativa¹ | nativa¹ | nativa¹ |
| AMD RX 6800/6900 | não | não | não | nativa¹ | nativa¹ | nativa¹ |
| AMD RX 6600 | não | não | não | não | nativa¹ | nativa¹ |
| AMD R9 200/300, Fury (GCN 1 a 3) | nativa | nativa | nativa | nativa | nativa | OCLP² |
| NVIDIA Kepler (GTX 600/700, GT 710/730) | nativa | nativa | nativa | nativa | nativa | OCLP² |
| NVIDIA Maxwell e Pascal (GTX 900/1000) | Web Driver³ | não | não | não | não | não |
| NVIDIA RTX (todas), AMD RX 6700, RX 7000 e mais novas, Intel | não | não | não | não | não | não |

1. Placas Navi podem dar tela preta com o modelo de Mac da VM; o conserto
   (WhateverGreen com `agdpmod=pikera`) ainda não é instalado pelo Cuvée.
2. O macOS 13 tirou o driver. Os patches de raiz do
   [OpenCore Legacy Patcher](https://github.com/dortania/OpenCore-Legacy-Patcher)
   devolvem. Veja "GPUs antigas" abaixo.
3. O Web Driver da NVIDIA não é mais oferecido; o Cuvée não o instala.

A recomendação é uma **AMD Polaris (RX 570 ou RX 580)**: barata, nativa em
todas as versões e sem o "reset bug" das Vega e Navi (que pedem reiniciar o
Linux entre um boot e outro da VM, ou o módulo `vendor-reset`).

`cuvee passthrough list` mostra o que o Cuvée achou na sua máquina.

## Passo a passo

1. **Ligue o IOMMU na BIOS**: VT-d (Intel) ou AMD-Vi/IOMMU (AMD). Em placas
   AMD costuma estar ligado. Na Intel, acrescente `intel_iommu=on iommu=pt` à
   linha do kernel no boot loader; na AMD, só `iommu=pt` (opcional, deixa o
   resto do sistema mais rápido). `cuvee passthrough list` diz o que falta.
2. **Deixe a tela do Linux na outra GPU**: ligue o monitor do Linux nela e, na
   BIOS, escolha-a como GPU primária.
3. **Confira a GPU**:
   ```sh
   cuvee passthrough list --macos sequoia
   cuvee passthrough check 0c:00 --macos sequoia --memory 8
   ```
   O endereço (`0c:00`) é o que o `list` mostra. Se aparecer "outros
   dispositivos dividem o grupo IOMMU", troque a placa de slot.
4. **Prepare o host** (pede `sudo`):
   ```sh
   cuvee passthrough setup-host 0c:00          # vfio-pci a partir do próximo boot
   cuvee passthrough setup-host 0c:00 --print  # só mostra o que faria
   ```
   Ele escreve:
   - `/etc/modprobe.d/cuvee-vfio.conf`: o `vfio-pci` pega os IDs da placa no
     boot, antes do `amdgpu`/`nouveau`/`nvidia`;
   - o `vfio-pci` no initramfs (mkinitcpio, dracut ou initramfs-tools) e
     reconstrói o initramfs;
   - `/etc/udev/rules.d/71-cuvee-vfio.rules`: o grupo `kvm` abre
     `/dev/vfio/*`, então o QEMU roda como você;
   - `/etc/security/limits.d/90-cuvee-memlock.conf` e um drop-in do
     `user@.service`: o QEMU trava toda a RAM da VM na memória.

   Reinicie depois. `--now` também passa a placa para o `vfio-pci` na hora,
   sem reiniciar (só funciona se o Linux não estiver usando a placa).
   `cuvee passthrough undo-host` desfaz tudo.
5. **Crie a VM e entregue a GPU**:
   ```sh
   cuvee create mac15 --macos sequoia
   cuvee passthrough attach mac15 0c:00
   cuvee start mac15
   ```
   No aplicativo, o passo Hardware do "Nova máquina" mostra as GPUs que servem
   para o macOS escolhido e faz tudo isso sozinho.

   A janela do QEMU continua existindo, com a tela do firmware: é nela que
   aparecem o OpenCore e o instalador. Depois da instalação, o macOS aparece no
   monitor ligado na GPU, com Metal. `cuvee passthrough detach mac15` devolve
   a VM à GPU virtual.

## GPUs antigas (OCLP)

Kepler e AMD GCN 1 a 3 no macOS 13, 14 e 15 precisam dos patches de raiz do
OpenCore Legacy Patcher. O Cuvée **não distribui o OCLP**; ele só ajusta o
OpenCore da VM do jeito que os patches pedem (SIP parcialmente desligado,
`csr-active-config` 0x867, e `amfi=0x80 ipc_control_port_options=0`):

```sh
cuvee passthrough oclp mac15 --macos sequoia   # com a VM desligada
```

Ele guarda o OpenCore original ao lado (`OpenCore.qcow2.cuvee-bak`). Depois:

1. instale o macOS normalmente (a tela ainda sem aceleração);
2. dentro do macOS, baixe o OCLP da
   [página de releases oficial](https://github.com/dortania/OpenCore-Legacy-Patcher/releases);
3. abra o OCLP e rode só **Post-Install Root Patch** (não instale o OpenCore
   dele: a VM já tem o seu);
4. reinicie a VM.

O OCLP é feito para Macs de verdade e o projeto dele não dá suporte a VMs;
problemas com os patches são para o Cuvée, não para eles.

## O que o Cuvée passa ao QEMU

`attach` acrescenta aos `extra_args` da VM (via `reims-ctl set --extra-args`)
uma porta PCIe própria e cada função da placa nela, com os mesmos números de
função, e troca a GPU virtual pelo VMware SVGA (só para a tela do firmware):

```
-device pcie-root-port,id=cuvee-gpu-port,bus=pcie.0,addr=0x1c,chassis=28,slot=28
-device vfio-pci,host=0000:0c:00.0,id=cuvee-gpu-0,bus=cuvee-gpu-port,addr=0x0.0,multifunction=on
-device vfio-pci,host=0000:0c:00.1,id=cuvee-gpu-1,bus=cuvee-gpu-port,addr=0x0.1
```

Outros `extra_args` que você tenha posto ficam como estão.

## Ainda não validado

Isto foi escrito e testado sem uma segunda GPU: a detecção, os grupos IOMMU,
os limites e a edição do OpenCore foram conferidos numa máquina real, mas o
boot do macOS com uma placa passada ainda não. O critério de pronto da v0.4 é
o macOS 15 rodando com Metal numa GPU passada; até lá, trate como beta.
