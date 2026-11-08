# Instalação

Escolha **um** dos jeitos abaixo. Todos instalam as mesmas coisas: o comando
`cuvee`, o aplicativo `cuvee-gui`, o QEMU com o reims-vgpu (em
`/usr/lib/cuvee/qemu`, separado do QEMU da sua distro), o `reims-ctl` e o
`reims-util`.

Depois de instalar, siga [Preparar o host](#preparar-o-host): esse passo vale
para todos.

## Instalador de um comando (qualquer distro)

```sh
curl -fsSL https://yurisanp.github.io/cuvee/install.sh | sh
```

O instalador descobre a sua distro, instala as dependências com o gerenciador
de pacotes dela (pede sua senha de `sudo`), baixa o Cuvée já compilado e
confere as somas SHA-256. Quer ler o script antes? Baixe e abra:

```sh
curl -fsSLO https://yurisanp.github.io/cuvee/install.sh
less install.sh
sh install.sh
```

## Ubuntu e Debian (.deb)

Funciona no Ubuntu 22.04, 24.04 ou mais novo e no Debian 12 e 13.

```sh
cd /tmp
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee-runtime_amd64.deb
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee_amd64.deb
sudo apt install ./cuvee-runtime_amd64.deb ./cuvee_amd64.deb
```

Use `apt install ./arquivo.deb` (com o `./`), e não `dpkg -i`: o apt baixa as
dependências sozinho.

Driver Vulkan:

| GPU | Pacote |
|---|---|
| AMD ou Intel | `sudo apt install mesa-vulkan-drivers` |
| NVIDIA | o driver proprietário (`sudo ubuntu-drivers install` no Ubuntu; `nvidia-driver` no Debian, do repositório non-free) |

## Fedora (.rpm)

```sh
cd /tmp
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee-runtime.x86_64.rpm
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/cuvee.x86_64.rpm
sudo dnf install ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm
```

Driver Vulkan: `sudo dnf install mesa-vulkan-drivers` (AMD e Intel). Para
NVIDIA, o `akmod-nvidia` do [RPM Fusion](https://rpmfusion.org/Howto/NVIDIA).

## openSUSE Tumbleweed (.rpm)

Baixe os dois `.rpm` como no Fedora e instale com:

```sh
sudo zypper install --allow-unsigned-rpm ./cuvee-runtime.x86_64.rpm ./cuvee.x86_64.rpm
```

Driver Vulkan: `libvulkan_radeon` (AMD) ou `libvulkan_intel` (Intel). Para
NVIDIA, o repositório oficial da NVIDIA para openSUSE.

## Arch Linux

Por enquanto, a partir do repositório (o pacote no AUR vem depois):

```sh
git clone https://github.com/yurisanp/cuvee
cd cuvee/packaging/reims-vgpu-git && makepkg -si   # compila o QEMU: 20 a 40 minutos
cd ../cuvee-git && makepkg -si
```

Driver Vulkan: `vulkan-radeon` (AMD), `vulkan-intel` (Intel) ou `nvidia-utils`
(NVIDIA).

## Conferir a assinatura (opcional)

Cada release traz o arquivo `SHA256SUMS`, assinado com
[minisign](https://jedisct1.github.io/minisign/). A chave pública é o
`cuvee.pub` da raiz do repositório.

```sh
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/SHA256SUMS
curl -fLO https://github.com/yurisanp/cuvee/releases/latest/download/SHA256SUMS.minisig
curl -fLO https://raw.githubusercontent.com/yurisanp/cuvee/main/cuvee.pub
minisign -Vm SHA256SUMS -p cuvee.pub
sha256sum -c --ignore-missing SHA256SUMS
```

As duas últimas linhas precisam terminar sem erro: a primeira diz
`Signature and comment signature verified`, a segunda mostra `OK` para cada
arquivo que você baixou.

## Preparar o host

Como seu usuário normal, **sem** `sudo` (o comando pede a senha quando
precisa):

```sh
cuvee setup-host
```

Ele carrega o módulo `udmabuf`, cria a regra que deixa o grupo `kvm` usar o
`/dev/udmabuf`, ajusta o `vm.max_map_count` em GPUs NVIDIA e põe você no grupo
`kvm`.

**Saia da sessão e entre de novo** (ou reinicie) para o grupo valer. Depois:

```sh
cuvee doctor
```

Tudo verde? Vá para a [primeira VM](primeira-vm.md). Algum `✗`? O próprio
doctor diz o que fazer; os casos mais comuns estão em
[Problemas comuns](problemas.md).
