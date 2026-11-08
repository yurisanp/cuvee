# Cuvée · pt-BR (idioma padrão e de fallback)

app-name = Cuvée

## Navegação
nav-painel = Painel
nav-nova = Nova máquina
nav-diagnostico = Diagnóstico do host
nav-comportamento = Guia de comportamento
nav-ajustes = Ajustes

## Painel
painel-title = { $n ->
    [0] Nenhuma máquina.
    [1] Uma máquina.
    [2] Duas máquinas.
    [3] Três máquinas.
    [4] Quatro máquinas.
   *[other] { $n } máquinas.
}
painel-running = { $n ->
    [0] Nenhuma em execução.
    [1] Uma em execução.
   *[other] { $n } em execução.
}
painel-empty-hint = Crie a primeira com “Nova máquina”.
painel-host-ok = Host pronto
painel-host-warn = Host pronto · { $n ->
    [1] 1 ajuste sugerido
   *[other] { $n } ajustes sugeridos
}
painel-host-bad = Host precisa de atenção
painel-new = + Nova máquina
painel-change-language = trocar idioma
vm-state-running = Em execução
vm-state-stopped = Desligada
vm-state-paused = Pausada
vm-accelerated = acelerado pela Reims vGPU
vm-not-accelerated = sem aceleração
vm-passthrough = GPU dedicada em passthrough
vm-open = Abrir { $name }
vm-start = Iniciar { $name }
vm-unknown-macos = macOS

## Nova máquina
nova-step = Passo { $step } de { $total }
nova-step-system = Sistema
nova-step-hardware = Hardware
nova-step-disk = Disco
nova-step-review = Revisão
nova-title = Qual macOS?
nova-video = Vídeo
nova-installer = Instalação
nova-suggested = Sugerido
nova-tag-accelerated = Acelerado pela Reims vGPU
nova-tag-plain = Sem aceleração · GPU dedicada opcional
nova-video-metal = Metal, pela GPU do Linux
nova-video-plain = Simples, sem Metal
nova-installer-both = Online ou offline
nova-installer-online = Só online
nova-hw = { $cpus } núcleos · { $gib } GB
nova-cancel = Cancelar
nova-continue = Continuar
nova-back = Voltar
nova-create = Criar máquina
nova-name = Nome da máquina
nova-cpus = Núcleos
nova-memory = Memória
nova-disk-size = Tamanho do disco
nova-disk-hint = O disco cresce conforme o uso; o tamanho é só o limite.
nova-review-title = Tudo pronto para criar
nova-review-command = O Cuvée vai rodar:

tip-tahoe = A melhor experiência hoje. Nenhuma placa extra é necessária.
tip-sequoia = Com uma segunda GPU AMD em passthrough, o vídeo fica nativo.
tip-sonoma = Desligar a transparência deixa a interface mais leve.
tip-ventura = Bom para Xcode 14 e 15.
tip-monterey = Último com driver NVIDIA Kepler nativo.
tip-big-sur = A versão mais antiga com instalador offline.
tip-catalina = Funciona a partir do 10.15.6. Já não roda apps 32 bits.
tip-mojave = O último que roda apps 32 bits.
tip-high-sierra = Único com driver NVIDIA para Maxwell e Pascal.

## Diagnóstico
diag-of = de { $total }
diag-all-ok = Tudo pronto.
diag-all-ok-sub = Seu Linux está pronto para rodar macOS com aceleração.
diag-one-warn = Falta um ajuste.
diag-many-warn = { $n } ajustes sugeridos.
diag-bad = Algo impede as máquinas de ligar.
diag-fix = Corrigir agora
diag-show-command = Ver o comando
diag-hide-command = Esconder o comando
diag-rerun = Verificar de novo
diag-sudo-note = O comando pede sua senha de administrador.
diag-ok = ok
diag-needs-fix = precisa de ajuste
diag-report = Gerar relatório para relatar um bug
diag-report-saved = Relatório salvo em { $path }. Anexe-o à issue no GitHub.

check-cpu-virt = Virtualização na CPU
check-kvm = Acesso ao /dev/kvm
check-reims-tools = reims-ctl e reims-util
check-vulkan = Driver Vulkan
check-udmabuf = Módulo udmabuf
check-max-map-count = vm.max_map_count (NVIDIA)
check-session = Wayland ou X11
check-disk = Espaço em disco
check-iommu = IOMMU para passthrough

advice-cpu-virt = Sua CPU não expõe VT-x ou AMD-V. Ative a virtualização no firmware (BIOS/UEFI).
advice-kvm-missing = O /dev/kvm não existe. O módulo kvm_intel ou kvm_amd não está carregado.
advice-kvm-access = Você não tem acesso ao /dev/kvm. Entre no grupo kvm e abra a sessão de novo.
advice-reims-tools = O reims-ctl, o reims-util ou o QEMU da Reims vGPU não estão instalados. Rode cuvee install (ou instale o pacote da sua distro).
advice-vulkan-broken = O driver Vulkan da sua GPU está instalado, mas o vulkaninfo não encontrou a GPU. Se o driver foi atualizado, reinicie o computador. Se continuar, rode cuvee report e anexe o arquivo.
advice-vulkan-untested = O driver Vulkan foi encontrado, mas o vulkaninfo não está instalado, então ele não foi testado. Instale o pacote vulkan-tools.
advice-vulkan-missing = Nenhum driver Vulkan foi encontrado para a sua GPU. Instale o da Mesa: vulkan-radeon ou vulkan-intel (Arch), mesa-vulkan-drivers (Debian, Ubuntu, Fedora), libvulkan_radeon ou libvulkan_intel (openSUSE).
advice-vulkan-nvidia = GPU NVIDIA sem driver Vulkan. Instale o driver proprietário ou o nvidia-open da sua distribuição.
advice-vulkan-nvidia-pacman = GPU NVIDIA sem driver Vulkan. Instale o nvidia-utils (com nvidia-open ou nvidia-open-dkms).
advice-vulkan-nvidia-apt = GPU NVIDIA sem driver Vulkan. No Debian, instale o nvidia-driver (repositório non-free). No Ubuntu, rode sudo ubuntu-drivers install.
advice-vulkan-nvidia-dnf = GPU NVIDIA sem driver Vulkan. No Fedora, ative o RPM Fusion e instale o akmod-nvidia e o xorg-x11-drv-nvidia.
advice-vulkan-nvidia-zypper = GPU NVIDIA sem driver Vulkan. No openSUSE, adicione o repositório da NVIDIA e instale o nvidia-video-G06 e o nvidia-gl-G06.
advice-udmabuf-missing = O módulo udmabuf não está carregado. Sem ele, a Reims vGPU não leva a memória da VM até a GPU.
advice-udmabuf-access = Você não tem acesso ao /dev/udmabuf. O Cuvée cria a regra udev e põe você no grupo kvm.
advice-max-map-count = Em GPU NVIDIA, um vm.max_map_count baixo faz a GPU receber cópias da memória. Subir o valor acelera a VM.
advice-session-none = Nenhuma sessão gráfica foi detectada. A VM só roda sem janela.
advice-disk-low = Menos de 64 GB livres. Um macOS instalado com seu instalador ocupa de 40 a 60 GB.
advice-iommu-off = O IOMMU está desligado. Só precisa dele para passar uma GPU física à VM (macOS 10.13 a 15).

## Ajustes
ajustes-title = Ajustes
ajustes-language = Idioma
ajustes-motion = Animações
ajustes-motion-on = Ligadas
ajustes-motion-off = Reduzidas
ajustes-motion-hint = Reduzidas segue a preferência de acessibilidade do sistema e desliga os movimentos.
ajustes-defaults = Padrões para novas máquinas
ajustes-cpus = Núcleos
ajustes-memory = Memória
ajustes-advice-cpus-high = Deixe ao menos 2 núcleos para o Linux. Seu host tem { $host }.
ajustes-advice-cpus-ok = Bom equilíbrio entre a VM e o host.
ajustes-advice-mem-high = Acima da metade da memória do host ({ $host } GB); o Linux pode ficar sem.
ajustes-advice-mem-low = Menos de 8 GB deixa o macOS 26 lento ao abrir apps.
ajustes-advice-mem-ok = Confortável para a maioria dos usos.
ajustes-about = Sobre
ajustes-about-text = Cuvée { $version } · sobre a Reims vGPU · GPL-3.0-or-later
ajustes-saved = Salvo.

## Comportamento
comp-title = O que esperar
comp-normal = Normal
comp-normal-text = Primeiro boot demorado e avisos de kext no registro.
comp-ask-help = Peça ajuda se
comp-ask-help-text = A tela fica preta após o logo, ou a VM reinicia em loop.
comp-legend = Legenda
comp-legend-good = bem
comp-legend-limited = com limites
comp-legend-bad = não funciona
comp-validate = Tabela a validar em hardware real.

group-tahoe = macOS 26
group-ventura-sequoia = macOS 13 a 15
group-big-sur-monterey = macOS 11 e 12
group-high-sierra-catalina = macOS 10.13 a 10.15

aspect-video-no-gpu = Vídeo sem GPU extra
aspect-amd-passthrough = GPU AMD dedicada
aspect-nvidia-passthrough = GPU NVIDIA dedicada
aspect-network = Rede
aspect-apple-services = Serviços Apple

beh-tahoe-video = Acelerado, com Metal
beh-tahoe-amd = Possível, mas desnecessária
beh-video-usable = Usável, sem Metal
beh-video-slow = Lento, sem Metal
beh-amd-native-wide = Nativa (RX 400 a 6000, quase todas)
beh-amd-polaris-vega = Polaris e Vega nativas
beh-nvidia-no-driver = Sem driver
beh-nvidia-kepler = Só Kepler (GTX 600 e 700)
beh-nvidia-kepler-pascal = Kepler; Maxwell e Pascal só no 10.13
beh-network-ok = Funciona
beh-apple-services = Pedem um serial único

## Em execução
exec-show-window = Mostrar janela
exec-pause = Pausar
exec-resume = Continuar
exec-power-off = Desligar
exec-snapshot = Snapshot
exec-is-this-normal = Isto é normal?
exec-cpu = CPU
exec-host-gpu = GPU do host
exec-memory = Memória
exec-sample-note = valores de exemplo
exec-summary = macOS { $macos } · Reims vGPU · { $cpus } núcleos · { $gib } GB
exec-back = Voltar ao painel

## Erros e avisos
err-reims-ctl-missing = O reims-ctl não está instalado. Rode cuvee install e abra o Cuvée de novo.
err-generic = Algo deu errado: { $error }
exec-no-reading = sem leitura
exec-force-off = Forçar desligamento
exec-stopping = Desligando…

## Linha de comando (cuvee)
cli-about = Cuvée { $version }: VMs de macOS no Linux, sobre o reims-ctl.
cli-doctor-title = Diagnóstico do host
cli-doctor-summary-ok = Tudo certo ({ $warnings } aviso(s)).
cli-doctor-summary-bad = { $problems } problema(s) e { $warnings } aviso(s).
cli-doctor-fix = corrige com: { $command }
cli-doctor-reims = Diagnóstico do reims-ctl
cli-gpus = GPUs: { $list }
cli-distro = Distribuição: { $name }
cli-report-saved = Relatório salvo em { $path }
cli-report-next = Abra o arquivo, confira e anexe-o a uma issue: https://github.com/yurisanp/cuvee/issues/new
cli-distro-unknown = Não sei instalar pacotes em { $name }. Instale à mão o equivalente destes pacotes do Ubuntu e rode cuvee install --skip-deps: { $list }
cli-packaged = O Cuvée deste sistema veio de um pacote ({ $kind }). Atualize e instale pelo gerenciador de pacotes da sua distro.
cli-deps-ok = Dependências já instaladas.
cli-deps-missing = Faltam pacotes: { $list }
cli-deps-running = Rodando: { $command }
cli-deps-failed = A instalação de pacotes falhou ({ $command })
cli-fetching = Baixando { $name }…
cli-runtime-ready = Runtime { $tag } instalado em { $path }
cli-runtime-current = Runtime { $tag } já instalado.
cli-runtime-broken = O QEMU baixado não roda nesta distro: { $error }
cli-installed-bin = { $name } instalado em { $path }
cli-path-missing = { $dir } não está no seu PATH. Abra um terminal novo ou adicione ao ~/.profile: export PATH="$HOME/.local/bin:$PATH"
cli-install-done = Pronto. Rode cuvee doctor quando quiser conferir o host.
cli-update-current = Você já está na versão mais nova ({ $tag }).
cli-update-to = Atualizando de { $from } para { $to }.
cli-uninstall-done = Cuvée removido. Suas VMs continuam em { $path }.
cli-setup-title = Preparando o host
cli-setup-no-root = Rode como seu usuário normal, sem sudo: o cuvee pede sudo quando precisa.
cli-setup-udmabuf = Módulo udmabuf carregado.
cli-setup-udmabuf-boot = O udmabuf vai carregar em todo boot ({ $path }).
cli-setup-udev = Regra udev criada para o /dev/udmabuf ({ $path }).
cli-setup-sysctl = vm.max_map_count = { $value } (NVIDIA; { $path })
cli-setup-group = Você foi adicionado ao grupo kvm.
cli-setup-relogin = Saia e entre de novo na sessão (ou reinicie) para valer o grupo kvm.
cli-setup-nothing = Nada a mudar: o host já estava preparado.
cli-setup-failed = Não deu para { $what }: { $error }
cli-sudo = Pedindo sudo para: { $command }
cli-reims-missing = O reims-ctl não está instalado: rode cuvee install.

## GPU passthrough (VFIO)
beh-nvidia-kepler-oclp = Só Kepler, com os patches do OCLP
pt-title = GPU passthrough
pt-intro = Passe uma GPU física para a VM e o macOS 10.13 a 15 rodam com Metal. Precisa de uma segunda GPU: a que desenha a tela do Linux fica com o Linux.
pt-none = Nenhuma GPU encontrada.
pt-support-native = driver nativo
pt-support-oclp = com os patches do OCLP
pt-support-webdriver = só com o Web Driver da NVIDIA
pt-support-none = sem driver no macOS
pt-family-amd-gcn = AMD GCN 1 a 3 (R9 200/300, Fury)
pt-family-polaris = AMD Polaris (RX 460 a 590)
pt-family-polaris-spoof = AMD Polaris 12 ou RX 580 2048SP
pt-family-vega10 = AMD Vega 56/64
pt-family-vega20 = AMD Radeon VII
pt-family-navi10 = AMD RX 5600/5700
pt-family-navi14 = AMD RX 5500
pt-family-navi21 = AMD RX 6800/6900
pt-family-navi23 = AMD RX 6600
pt-family-amd-other = AMD sem driver no macOS
pt-family-kepler = NVIDIA Kepler (GTX 600/700)
pt-family-maxwell-pascal = NVIDIA Maxwell ou Pascal (GTX 900/1000)
pt-family-nvidia-other = NVIDIA sem driver no macOS
pt-family-intel = Intel (iGPU)
pt-family-other = Outra
pt-find-iommu-off = O IOMMU está desligado. Ligue VT-d (Intel) ou AMD-Vi (AMD) na BIOS e acrescente os parâmetros de kernel indicados.
pt-find-unsupported = O macOS não tem driver para esta GPU nesta versão.
pt-find-webdriver = Só funciona com o Web Driver da NVIDIA no 10.13, que a NVIDIA não oferece mais. O Cuvée não instala.
pt-find-oclp = O macOS tirou o driver desta GPU. Depois de instalar, baixe o OpenCore Legacy Patcher dentro do macOS e rode "Post-Install Root Patch". Antes, rode cuvee passthrough oclp.
pt-caveat-navi-agdp = Placas Navi podem dar tela preta com o modelo de Mac da VM (falta o WhateverGreen com agdpmod=pikera, que o Cuvée ainda não instala).
pt-find-reset-bug = Esta GPU não se reinicia sozinha: depois de desligar a VM, pode ser preciso reiniciar o Linux para ligá-la de novo (ou instalar o módulo vendor-reset).
pt-find-group-shared = Outros dispositivos dividem o grupo IOMMU com a GPU e iriam junto para a VM. Troque a placa de slot ou use outra.
pt-find-host-screen-only = Esta é a única GPU do host: o Linux ficaria sem tela. Passthrough precisa de uma segunda GPU.
pt-find-host-screen = O Linux desenha a tela de boot nesta GPU. Ligue o monitor do Linux na outra GPU e escolha-a como primária na BIOS.
pt-find-not-bound = A GPU ainda está com o driver do Linux. cuvee passthrough setup-host passa ela para o vfio-pci.
pt-find-vfio-access = Você não tem acesso ao grupo VFIO. cuvee passthrough setup-host cria a regra udev para o grupo kvm.
pt-find-memlock = O limite de memória travada é menor que a RAM da VM. cuvee passthrough setup-host libera o limite para o grupo kvm (saia e entre na sessão depois).
pt-ready = Pronta para passar à VM.
pt-possible = Dá para usar depois do cuvee passthrough setup-host.
pt-blocked = Não dá para usar com este macOS.
pt-group = grupo IOMMU { $group }
pt-driver = driver { $driver }
pt-kernel-params = Acrescente ao boot loader e reinicie: { $params }
pt-setup-done = Pronto. Reinicie o computador para o vfio-pci assumir a GPU no boot.
pt-setup-bound = A GPU já está no vfio-pci nesta sessão.
pt-attach-done = { $vm } vai usar a GPU { $gpu } no próximo boot. O macOS aparece no monitor ligado nela; a janela do QEMU mostra só o OpenCore e o instalador.
pt-detach-done = { $vm } voltou para a GPU virtual.
pt-oclp-done = OpenCore de { $vm } pronto para os patches do OCLP (cópia do original em { $backup }).
pt-oclp-already = O OpenCore de { $vm } já estava pronto para os patches do OCLP.
pt-oclp-not-needed = Esta GPU tem driver nativo neste macOS; não precisa do OCLP.
pt-vm-running = Desligue { $vm } antes.
pt-vm-missing = O reims-ctl não conhece a VM { $vm }.
pt-gpu-missing = Nenhuma GPU em { $gpu }. Veja as GPUs com cuvee passthrough list.
nova-gpu = GPU física
nova-gpu-none = Só a GPU virtual
nova-gpu-hint = Passe uma GPU AMD ou NVIDIA Kepler à VM para ter Metal no macOS { $macos }.
nova-gpu-no-candidate = Nenhuma GPU deste computador serve para o macOS { $macos }. A VM roda sem aceleração.

# Identidade do Mac (SMBIOS)
smbios-model = Modelo
smbios-serial = Série
smbios-show-title = O Mac que { $vm } apresenta
smbios-new-warning = Com uma identidade nova, o macOS vê outro Mac: iCloud, iMessage e FaceTime pedem login de novo, e apps com licença por máquina podem pedir ativação.
smbios-new-confirm = Gerar uma identidade nova para { $vm }?
smbios-new-needs-yes = Sem terminal para confirmar: rode de novo com --yes.
smbios-new-done = { $vm } agora apresenta outro Mac
smbios-new-old = A série anterior era { $serial }; o OpenCore como estava antes do Cuvée mudá-lo está em { $backup }.
smbios-check-coverage = Se quiser conferir a série em { $url }, o esperado é "não foi possível verificar a cobertura": quer dizer que ela não é de um Mac vendido pela Apple.
nova-identity = Identidade do Mac
nova-identity-making = Gerando série, MLB, UUID e ROM…
nova-identity-again = Gerar de novo
nova-identity-hint = Cada máquina recebe uma identidade própria, no formato das da Apple. A ROM é o endereço da placa de rede da VM.
nova-identity-failed = Não deu para gerar a identidade agora ({ $error }). O reims-ctl vai gerar uma ao criar a máquina.
