# Pendências do Yuri

O que só você pode fazer: contas, chaves, permissões, hardware e comandos com
`sudo`. Marque `[x]` quando terminar e faça commit; eu leio este arquivo antes
de cada release. O AUR fica de fora até o registro de contas reabrir.

## Agora

- [ ] Liberar ~10 GB de sobras de build (os dois `.pkg.tar.zst` ficam):
  ```sh
  cd /mnt/dados/Desenvolvimento/Linux/cuvee
  rm -rf packaging/reims-vgpu-git/{src,pkg,qemu-reims-vgpu,reims-vgpu,reims-util} \
         packaging/cuvee-git/{src,pkg,cuvee}
  ```
- [ ] Apagar os branches já mesclados no GitHub:
  `git push origin --delete arch-packaging gui`
- [x] Logar o GitHub CLI na sua máquina: `gh auth login`. Assim eu consigo
  acompanhar a CI e as releases daí.
- [ ] Remover a cópia de teste da v0.4 quando o PR #4 for mesclado:
  ```sh
  cd /mnt/dados/Desenvolvimento/Linux/cuvee
  git worktree remove ../cuvee-pt-test && rm -rf ~/.cache/cuvee-pt-*
  ```
- [ ] Permitir que eu envie tags de release pela sua máquina. No Claude Code do
  computador, adicione regras de permissão para `git tag -a v*` e
  `git push origin v*`. Sem isso, cada release depende de você rodar os dois
  comandos de tag.

## Antes da v0.2 (instalador universal)

- [x] Em Settings > Actions > General do repositório, em "Workflow
  permissions", marcar **Read and write permissions**. A CI precisa disso para
  publicar os binários do QEMU e do instalador.
- [x] Ativar o GitHub Pages (Settings > Pages > Source: GitHub Actions). O
  instalador fica em `https://yurisanp.github.io/cuvee/install.sh`.
- [ ] Opcional: registrar um domínio curto (por exemplo `cuvee.sh`) e apontar
  para o Pages.
- [ ] Ter uma VM de Ubuntu LTS e uma de Fedora para testar o instalador.

## Antes da v0.3 (pacotes nativos e documentação)

- [ ] Criar a chave de assinatura das releases com
  [minisign](https://jedisct1.github.io/minisign/):
  ```sh
  minisign -G -p cuvee.pub -s cuvee.key
  ```
  Guarde `cuvee.key` e a senha num gerenciador de senhas. Cadastre o conteúdo
  de `cuvee.key` como secret `MINISIGN_KEY` e a senha como
  `MINISIGN_PASSWORD` (Settings > Secrets and variables > Actions). Faça commit
  de `cuvee.pub` na raiz do repositório.
- [ ] Conseguir três pessoas de fora para instalar só lendo a documentação.

## Antes da v0.4 (GPU passthrough)

- [ ] Uma GPU dedicada suportada pelo macOS para passar à VM. A mais barata e
  segura é uma AMD Polaris (RX 570 ou RX 580).
- [ ] Ligar IOMMU na BIOS (VT-d na Intel, AMD-Vi/SVM na AMD) e, se o
  `cuvee doctor` pedir, os parâmetros de kernel que ele indicar.
- [ ] Com a segunda GPU instalada e o monitor do Linux na outra GPU (marque-a
  como primária na BIOS), rodar e me mandar a saída de:
  `cuvee passthrough list --macos sequoia`
- [ ] Rodar `cuvee passthrough setup-host <endereço da GPU>` (pede sudo; grava
  modprobe.d, initramfs, udev e limites) e reiniciar. Em CPU Intel, acrescentar
  `intel_iommu=on iommu=pt` à linha de boot do seu boot loader; o `list` mostra
  o que ainda falta.
- [ ] Criar uma VM com o macOS 15 Sequoia, rodar
  `cuvee passthrough attach <vm> <endereço da GPU>`, instalar e confirmar
  "Metal" em Sobre este Mac > Relatório do Sistema > Gráficos. Esse é o
  critério de pronto da v0.4.
- [ ] Opcional: com uma NVIDIA Kepler (GT 710, GTX 770), testar
  `cuvee passthrough oclp` e o Post-Install Root Patch do OCLP no Sonoma ou
  Sequoia.
- [ ] Abrir o app e conferir a escolha de GPU no passo Hardware de Nova máquina.

## Antes da v1.0

- [ ] Acesso a três GPUs para a matriz de testes: AMD, Intel e NVIDIA.
  Cada pessoa roda `cuvee report` e te manda o arquivo.
- [ ] Escolher o canal de suporte. O padrão são as issues do GitHub; se quiser
  Discord ou Telegram, crie o grupo e me passe o link.
- [ ] Deixar a v1.0 rodando 30 dias sem bug que impeça instalar ou ligar a VM.

## Feito

- [x] Instalar os pacotes do PR #1 e rodar `cuvee doctor` (8 de novembro de 2026)
- [x] Enviar a tag v0.1.0 (8 de novembro de 2026)
