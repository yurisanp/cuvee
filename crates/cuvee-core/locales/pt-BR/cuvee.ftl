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
advice-reims-tools = O reims-ctl, o reims-util ou o QEMU da Reims vGPU não estão instalados. Instale o pacote reims-vgpu.
advice-vulkan-broken = O vulkaninfo falhou. Instale o driver Vulkan da sua GPU: vulkan-radeon (AMD), vulkan-intel (Intel) ou nvidia-utils (NVIDIA).
advice-vulkan-untested = O vulkaninfo não está instalado, então o Vulkan não foi testado. Instale o pacote vulkan-tools.
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
err-reims-ctl-missing = O reims-ctl não está instalado. Instale o pacote reims-vgpu e abra o Cuvée de novo.
err-generic = Algo deu errado: { $error }
exec-no-reading = sem leitura
exec-force-off = Forçar desligamento
exec-stopping = Desligando…
