# Primeira VM

## Pelo aplicativo

Abra **Cuvée** no menu de aplicativos (ou rode `cuvee-gui`). Em
**Nova máquina**, escolha a versão do macOS, a memória, os núcleos e o disco;
a revisão mostra o comando exato que será executado. Depois, ligue a VM no
**Painel**.

## Pelo terminal

```sh
cuvee create mac --macos tahoe --start
```

Isso cria uma VM chamada `mac` com o macOS 26 Tahoe e já liga. Outros nomes de
versão: `cuvee help create` lista todos. Para ver as VMs: `cuvee list`.

## Instalar o macOS

Na primeira vez, a VM abre no **macOS Recovery**:

1. Abra o **Utilitário de Disco**, escolha o disco **QEMU HARDDISK Media**,
   clique em **Apagar** e escolha o formato **APFS**.
2. Feche o Utilitário de Disco e escolha **Reinstalar macOS**.
3. Espere: o instalador baixa uns 15 GB da Apple e reinicia algumas vezes.

## Passo obrigatório no macOS 26

Sem ele, o desktop não aparece depois da instalação. Siga o item 3 do
[README do reims-vgpu](https://github.com/steelbrain/reims-vgpu#x86-64-guest-on-linux-kvm):
pelo Terminal do Recovery, defina o `METAL_ERROR_MODE` como ele descreve.

## Dicas

- **Soltar teclado e mouse** da janela da VM: **Ctrl+Alt+Super**.
- **Desligar**: pelo menu Apple dentro do macOS, ou `cuvee stop mac`.
- **Mudar memória, núcleos ou resolução**: `cuvee set mac ...` com a VM
  desligada (`cuvee help set` mostra as opções).
- As VMs ficam em `~/.local/share/reims-ctl/vms`.

## Identidade do Mac (SMBIOS)

Cada VM já nasce com um Mac próprio: modelo, número de série, MLB, UUID e
ROM. Não é preciso rodar o GenSMBIOS nem o macserial: o reims-util gera os
valores ao criar a VM, e o aplicativo mostra a identidade na revisão de "Nova
máquina", com o botão **Gerar de novo**.

- Ver a identidade de uma VM: `cuvee smbios show mac`.
- Gerar outra (com a VM desligada): `cuvee smbios new mac`. O macOS passa a
  ver outro Mac, então iCloud e iMessage pedem login de novo.
- Só gerar uma identidade, sem VM: `cuvee smbios generate --macos sequoia`.

Para conferir uma série em <https://checkcoverage.apple.com>, o resultado
esperado é "não foi possível verificar a cobertura": a série não pertence a
um Mac vendido pela Apple.
