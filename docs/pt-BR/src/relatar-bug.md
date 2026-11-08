# Relatar um bug

Antes, veja se o problema já está em [Problemas comuns](problemas.md) ou nas
[issues abertas](https://github.com/yurisanp/cuvee/issues).

## 1. Gere o relatório

```sh
cuvee report mac
```

Troque `mac` pelo nome da VM com problema, ou rode só `cuvee report` se o
problema não é de uma VM. O comando grava dois arquivos na sua pasta
Downloads (ou na sua pasta pessoal):

- `cuvee-report-DATA.txt`: distro, kernel, processador, GPUs e drivers,
  sessão gráfica, versões instaladas, o resultado de cada verificação do
  `cuvee doctor`, o `vulkaninfo --summary` e a lista de VMs.
- um `.zip` do `reims-ctl report`, com os logs e as configurações da VM.

O texto não leva dados pessoais: o caminho da sua pasta pessoal vira `~`, o
seu nome de usuário vira `<user>` e o nome do computador vira `<host>`. Da
linha de comando do kernel, só entram os parâmetros de IOMMU e VFIO. Mesmo
assim, abra o arquivo e confira antes de enviar.

## 2. Abra uma issue

Em [github.com/yurisanp/cuvee/issues/new](https://github.com/yurisanp/cuvee/issues/new),
conte:

- o que você fez, passo a passo;
- o que esperava que acontecesse;
- o que aconteceu (a mensagem de erro exata, ou uma captura de tela).

Arraste o `.txt` e o `.zip` para a caixa da issue. Pode escrever em português
ou em inglês.
