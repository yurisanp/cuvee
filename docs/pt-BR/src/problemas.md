# Problemas comuns

Comece sempre pelo `cuvee doctor`: ele confere KVM, udmabuf, GPU, Vulkan,
sessão gráfica e instalação, e diz o que fazer em cada `✗`. Se o problema não
estiver aqui, [relate um bug](relatar-bug.md).

## `/dev/kvm não existe`

A virtualização está desligada ou o módulo do KVM não carregou.

1. Ligue a virtualização no firmware (BIOS/UEFI): **Intel VT-x** (às vezes
   "Intel Virtualization Technology") ou **AMD-V/SVM**.
2. Carregue o módulo: `sudo modprobe kvm_intel` (Intel) ou
   `sudo modprobe kvm_amd` (AMD).

O Linux está dentro de outra VM? Então o hipervisor de fora precisa de
virtualização aninhada ligada.

## Sem acesso a `/dev/kvm` ou a `/dev/udmabuf`

Você não está no grupo `kvm`, ou entrou nele e ainda não saiu da sessão.

```sh
cuvee setup-host
```

Depois **saia e entre de novo**. Para conferir: `id -nG` precisa listar
`kvm`.

## `/dev/udmabuf não existe`

```sh
sudo modprobe udmabuf
```

Se der `Module udmabuf not found`, seu kernel foi compilado sem
`CONFIG_UDMABUF` (acontece com alguns kernels customizados). Use o kernel
padrão da sua distro: Ubuntu, Debian, Fedora, openSUSE e Arch já trazem o
módulo.

## Vulkan não funciona (`vulkaninfo falhou`)

Instale o driver Vulkan da sua GPU (tabelas em [Instalação](instalacao.md)) e
teste com:

```sh
vulkaninfo --summary
```

A lista de `Devices` precisa mostrar a sua GPU de verdade, e não só o
`llvmpipe` (renderização por software). Em notebooks com duas GPUs, as duas
aparecem; `cuvee gpus` mostra qual delas a VM pode usar.

O diagnóstico acha o driver pelo arquivo que o carregador do Vulkan usa (em
`/usr/share/vulkan/icd.d` ou `/etc/vulkan/icd.d`, como `nvidia_icd.json`), e
não pelo nome do pacote. Se ele diz que o driver está instalado mas o
`vulkaninfo` não achou a GPU, reinicie depois de atualizar o driver: o módulo
do kernel e as bibliotecas precisam ser da mesma versão.

## NVIDIA: VM lenta ou travando em resolução alta

Com GPU NVIDIA, o reims-vgpu precisa de um `vm.max_map_count` alto. O
`cuvee setup-host` já ajusta isso (grava em `/etc/sysctl.d/70-cuvee.conf`).
Confira com `sysctl vm.max_map_count`: o valor deve ser `1048576` ou mais. O
driver `nvidia-open` é o recomendado.

## macOS 26: a instalação termina e o desktop não aparece

Falta o passo do `METAL_ERROR_MODE`. Veja
[Passo obrigatório no macOS 26](primeira-vm.md#passo-obrigatório-no-macos-26).

## macOS 10.13 a 15 muito lento

É esperado: a aceleração gráfica do reims-vgpu só existe no macOS 26. As
versões anteriores rodam com uma tela simples, sem Metal. Para uso real,
prefira o macOS 26.

## O mouse e o teclado ficaram presos na janela

Aperte **Ctrl+Alt+Super** para soltar.

## Problemas de janela no Wayland

O Cuvée funciona em Wayland e X11. Se a janela da VM não abrir, piscar ou não
pegar o teclado no Wayland, tente rodar pelo XWayland:

```sh
GDK_BACKEND=x11 cuvee start mac
```

E conte no [relatório de bug](relatar-bug.md) qual ambiente você usa (GNOME,
KDE, Hyprland...).

## Pouco espaço em disco

Um macOS instalado com o instalador ocupa de 40 a 60 GB. As VMs ficam em
`~/.local/share/reims-ctl/vms`. Para liberar espaço, apague VMs que não usa
com `cuvee delete NOME` (apaga o disco dela também).

## Erro de dependência ao instalar o .deb ou o .rpm

Mensagens como `Depends: libalguma-coisa but it is not installable` querem
dizer que a sua versão da distro não está entre as suportadas
([Instalação](instalacao.md)), ou que um repositório padrão está desligado
(no Ubuntu, o `universe`: `sudo add-apt-repository universe`). Se a sua
versão é suportada, [relate um bug](relatar-bug.md) com a mensagem completa.

## `cuvee update` recusa atualizar

Se você instalou pelo `.deb` ou pelo `.rpm`, quem atualiza é o gerenciador da
sua distro. Veja [Atualizar e desinstalar](desinstalar.md).

## Ver o que a VM está fazendo

```sh
cuvee logs mac
```

Mostra o fim do log do QEMU. Erros de GPU costumam aparecer ali.
