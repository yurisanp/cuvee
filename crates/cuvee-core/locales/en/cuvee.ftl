# Cuvée · en

app-name = Cuvée

## Navigation
nav-painel = Dashboard
nav-nova = New machine
nav-diagnostico = Host check-up
nav-comportamento = Behaviour guide
nav-ajustes = Settings

## Dashboard
painel-title = { $n ->
    [0] No machines.
    [1] One machine.
    [2] Two machines.
    [3] Three machines.
    [4] Four machines.
   *[other] { $n } machines.
}
painel-running = { $n ->
    [0] None running.
    [1] One running.
   *[other] { $n } running.
}
painel-empty-hint = Create the first one with “New machine”.
painel-host-ok = Host ready
painel-host-warn = Host ready · { $n ->
    [1] 1 suggested fix
   *[other] { $n } suggested fixes
}
painel-host-bad = Host needs attention
painel-new = + New machine
painel-change-language = change language
vm-state-running = Running
vm-state-stopped = Off
vm-state-paused = Paused
vm-accelerated = accelerated by Reims vGPU
vm-not-accelerated = no acceleration
vm-passthrough = dedicated GPU in passthrough
vm-open = Open { $name }
vm-start = Start { $name }
vm-unknown-macos = macOS

## New machine
nova-step = Step { $step } of { $total }
nova-step-system = System
nova-step-hardware = Hardware
nova-step-disk = Disk
nova-step-review = Review
nova-title = Which macOS?
nova-video = Video
nova-installer = Install
nova-suggested = Suggested
nova-tag-accelerated = Accelerated by Reims vGPU
nova-tag-plain = No acceleration · optional dedicated GPU
nova-video-metal = Metal, through the Linux GPU
nova-video-plain = Basic, no Metal
nova-installer-both = Online or offline
nova-installer-online = Online only
nova-hw = { $cpus } cores · { $gib } GB
nova-cancel = Cancel
nova-continue = Continue
nova-back = Back
nova-create = Create machine
nova-name = Machine name
nova-cpus = Cores
nova-memory = Memory
nova-disk-size = Disk size
nova-disk-hint = The disk grows as it is used; the size is only the limit.
nova-review-title = Ready to create
nova-review-command = Cuvée will run:

tip-tahoe = The best experience today. No extra graphics card needed.
tip-sequoia = With a second AMD GPU in passthrough, video becomes native.
tip-sonoma = Turning off transparency makes the interface lighter.
tip-ventura = Good for Xcode 14 and 15.
tip-monterey = The last one with a native NVIDIA Kepler driver.
tip-big-sur = The oldest version with an offline installer.
tip-catalina = Works from 10.15.6. No longer runs 32-bit apps.
tip-mojave = The last one that runs 32-bit apps.
tip-high-sierra = The only one with an NVIDIA driver for Maxwell and Pascal.

## Check-up
diag-of = of { $total }
diag-all-ok = All set.
diag-all-ok-sub = Your Linux is ready to run macOS with acceleration.
diag-one-warn = One fix to go.
diag-many-warn = { $n } suggested fixes.
diag-bad = Something stops machines from starting.
diag-fix = Fix now
diag-show-command = Show the command
diag-hide-command = Hide the command
diag-rerun = Check again
diag-sudo-note = The command asks for your administrator password.
diag-ok = ok
diag-needs-fix = needs a fix

check-cpu-virt = CPU virtualization
check-kvm = Access to /dev/kvm
check-reims-tools = reims-ctl and reims-util
check-vulkan = Vulkan driver
check-udmabuf = udmabuf module
check-max-map-count = vm.max_map_count (NVIDIA)
check-session = Wayland or X11
check-disk = Disk space
check-iommu = IOMMU for passthrough

advice-cpu-virt = Your CPU does not expose VT-x or AMD-V. Enable virtualization in the firmware (BIOS/UEFI).
advice-kvm-missing = /dev/kvm does not exist. The kvm_intel or kvm_amd module is not loaded.
advice-kvm-access = You cannot open /dev/kvm. Join the kvm group and log in again.
advice-reims-tools = reims-ctl, reims-util or Reims vGPU's QEMU is not installed. Run cuvee install (or install your distribution's package).
advice-vulkan-broken = vulkaninfo failed. Install your GPU's Vulkan driver: vulkan-radeon (AMD), vulkan-intel (Intel) or nvidia-utils (NVIDIA).
advice-vulkan-untested = vulkaninfo is not installed, so Vulkan was not tested. Install the vulkan-tools package.
advice-udmabuf-missing = The udmabuf module is not loaded. Without it, Reims vGPU cannot bring the VM's memory to the GPU.
advice-udmabuf-access = You cannot open /dev/udmabuf. Cuvée creates the udev rule and adds you to the kvm group.
advice-max-map-count = On an NVIDIA GPU, a low vm.max_map_count makes the GPU receive copies of the memory. Raising it speeds the VM up.
advice-session-none = No graphical session was detected. The VM can only run without a window.
advice-disk-low = Less than 64 GB free. An installed macOS with its installer takes 40 to 60 GB.
advice-iommu-off = IOMMU is off. You only need it to hand a physical GPU to the VM (macOS 10.13 to 15).

## Settings
ajustes-title = Settings
ajustes-language = Language
ajustes-motion = Animations
ajustes-motion-on = On
ajustes-motion-off = Reduced
ajustes-motion-hint = Reduced follows the system's accessibility preference and turns motion off.
ajustes-defaults = Defaults for new machines
ajustes-cpus = Cores
ajustes-memory = Memory
ajustes-advice-cpus-high = Leave at least 2 cores for Linux. Your host has { $host }.
ajustes-advice-cpus-ok = A good balance between the VM and the host.
ajustes-advice-mem-high = Above half of the host's memory ({ $host } GB); Linux may run out.
ajustes-advice-mem-low = Under 8 GB makes macOS 26 slow to open apps.
ajustes-advice-mem-ok = Comfortable for most uses.
ajustes-about = About
ajustes-about-text = Cuvée { $version } · on Reims vGPU · GPL-3.0-or-later
ajustes-saved = Saved.

## Behaviour
comp-title = What to expect
comp-normal = Normal
comp-normal-text = A slow first boot and kext warnings in the log.
comp-ask-help = Ask for help if
comp-ask-help-text = The screen stays black after the logo, or the VM reboots in a loop.
comp-legend = Legend
comp-legend-good = works well
comp-legend-limited = with limits
comp-legend-bad = does not work
comp-validate = Table still to be validated on real hardware.

group-tahoe = macOS 26
group-ventura-sequoia = macOS 13 to 15
group-big-sur-monterey = macOS 11 and 12
group-high-sierra-catalina = macOS 10.13 to 10.15

aspect-video-no-gpu = Video without an extra GPU
aspect-amd-passthrough = Dedicated AMD GPU
aspect-nvidia-passthrough = Dedicated NVIDIA GPU
aspect-network = Network
aspect-apple-services = Apple services

beh-tahoe-video = Accelerated, with Metal
beh-tahoe-amd = Possible, but unnecessary
beh-video-usable = Usable, no Metal
beh-video-slow = Slow, no Metal
beh-amd-native-wide = Native (RX 400 to 6000, almost all)
beh-amd-polaris-vega = Polaris and Vega native
beh-nvidia-no-driver = No driver
beh-nvidia-kepler = Kepler only (GTX 600 and 700)
beh-nvidia-kepler-pascal = Kepler; Maxwell and Pascal only on 10.13
beh-network-ok = Works
beh-apple-services = Need a unique serial

## Running
exec-show-window = Show window
exec-pause = Pause
exec-resume = Resume
exec-power-off = Power off
exec-snapshot = Snapshot
exec-is-this-normal = Is this normal?
exec-cpu = CPU
exec-host-gpu = Host GPU
exec-memory = Memory
exec-sample-note = sample values
exec-summary = macOS { $macos } · Reims vGPU · { $cpus } cores · { $gib } GB
exec-back = Back to dashboard

## Errors and notices
err-reims-ctl-missing = reims-ctl is not installed. Run cuvee install and open Cuvée again.
err-generic = Something went wrong: { $error }
exec-no-reading = no reading
exec-force-off = Force power off
exec-stopping = Shutting down…

## Command line (cuvee)
cli-about = Cuvée { $version }: macOS VMs on Linux, on top of reims-ctl.
cli-doctor-title = Host check
cli-doctor-summary-ok = All good ({ $warnings } warning(s)).
cli-doctor-summary-bad = { $problems } problem(s) and { $warnings } warning(s).
cli-doctor-fix = fix with: { $command }
cli-doctor-reims = reims-ctl's own check
cli-gpus = GPUs: { $list }
cli-distro = Distribution: { $name }
cli-distro-unknown = I don't know how to install packages on { $name }. Install the equivalents of these Ubuntu packages yourself, then run cuvee install --skip-deps: { $list }
cli-packaged = Cuvée on this system came from a package ({ $kind }). Update and install it with your distribution's package manager.
cli-deps-ok = Dependencies already installed.
cli-deps-missing = Missing packages: { $list }
cli-deps-running = Running: { $command }
cli-deps-failed = Installing packages failed ({ $command }).
cli-nvidia-driver = NVIDIA GPU without a Vulkan driver: install your distribution's proprietary or nvidia-open driver (nvidia-utils on Arch).
cli-fetching = Downloading { $name }…
cli-runtime-ready = Runtime { $tag } installed in { $path }
cli-runtime-current = Runtime { $tag } is already installed.
cli-runtime-broken = The downloaded QEMU does not run on this distribution: { $error }
cli-installed-bin = { $name } installed in { $path }
cli-path-missing = { $dir } is not in your PATH. Open a new terminal, or add this to ~/.profile: export PATH="$HOME/.local/bin:$PATH"
cli-install-done = Done. Run cuvee doctor whenever you want to check the host.
cli-update-current = You already have the newest version ({ $tag }).
cli-update-to = Updating from { $from } to { $to }.
cli-uninstall-done = Cuvée removed. Your VMs are still in { $path }.
cli-setup-title = Preparing the host
cli-setup-no-root = Run this as your normal user, without sudo: cuvee asks for sudo when it needs it.
cli-setup-udmabuf = udmabuf module loaded.
cli-setup-udmabuf-boot = udmabuf will load at every boot ({ $path }).
cli-setup-udev = udev rule created for /dev/udmabuf ({ $path }).
cli-setup-sysctl = vm.max_map_count = { $value } (NVIDIA; { $path })
cli-setup-group = You were added to the kvm group.
cli-setup-relogin = Log out and back in (or reboot) for the kvm group to apply.
cli-setup-nothing = Nothing to change: the host was already prepared.
cli-setup-failed = Could not { $what }: { $error }
cli-sudo = Asking sudo for: { $command }
cli-reims-missing = reims-ctl is not installed: run cuvee install.

## GPU passthrough (VFIO)
beh-nvidia-kepler-oclp = Kepler only, with OCLP's patches
pt-title = GPU passthrough
pt-intro = Hand a physical GPU to the VM and macOS 10.13 to 15 run with Metal. It takes a second GPU: the one drawing Linux's screen stays with Linux.
pt-none = No GPU found.
pt-support-native = native driver
pt-support-oclp = with OCLP's patches
pt-support-webdriver = only with NVIDIA's Web Driver
pt-support-none = no macOS driver
pt-family-amd-gcn = AMD GCN 1 to 3 (R9 200/300, Fury)
pt-family-polaris = AMD Polaris (RX 460 to 590)
pt-family-polaris-spoof = AMD Polaris 12 or RX 580 2048SP
pt-family-vega10 = AMD Vega 56/64
pt-family-vega20 = AMD Radeon VII
pt-family-navi10 = AMD RX 5600/5700
pt-family-navi14 = AMD RX 5500
pt-family-navi21 = AMD RX 6800/6900
pt-family-navi23 = AMD RX 6600
pt-family-amd-other = AMD with no macOS driver
pt-family-kepler = NVIDIA Kepler (GTX 600/700)
pt-family-maxwell-pascal = NVIDIA Maxwell or Pascal (GTX 900/1000)
pt-family-nvidia-other = NVIDIA with no macOS driver
pt-family-intel = Intel (iGPU)
pt-family-other = Other
pt-find-iommu-off = IOMMU is off. Turn on VT-d (Intel) or AMD-Vi (AMD) in the firmware and add the kernel parameters shown.
pt-find-unsupported = macOS has no driver for this GPU on this version.
pt-find-webdriver = Works only with NVIDIA's Web Driver on 10.13, which NVIDIA no longer offers. Cuvée does not install it.
pt-find-oclp = macOS dropped this GPU's driver. Once installed, download OpenCore Legacy Patcher inside macOS and run "Post-Install Root Patch". Run cuvee passthrough oclp first.
pt-caveat-navi-agdp = Navi cards may show a black screen with the VM's Mac model (it needs WhateverGreen with agdpmod=pikera, which Cuvée does not install yet).
pt-find-reset-bug = This GPU does not reset itself: after the VM shuts down, Linux may need a reboot before the VM starts again (or the vendor-reset module).
pt-find-group-shared = Other devices share the GPU's IOMMU group and would go to the VM too. Move the card to another slot or use another card.
pt-find-host-screen-only = This is the host's only GPU: Linux would lose its screen. Passthrough needs a second GPU.
pt-find-host-screen = Linux draws its boot screen on this GPU. Plug Linux's monitor into the other GPU and make it the primary one in the firmware.
pt-find-not-bound = The GPU still has its Linux driver. cuvee passthrough setup-host hands it to vfio-pci.
pt-find-vfio-access = You cannot open the VFIO group. cuvee passthrough setup-host adds the udev rule for the kvm group.
pt-find-memlock = The locked-memory limit is below the VM's RAM. cuvee passthrough setup-host lifts it for the kvm group (log out and back in afterwards).
pt-ready = Ready to hand to the VM.
pt-possible = Usable after cuvee passthrough setup-host.
pt-blocked = Not usable with this macOS.
pt-group = IOMMU group { $group }
pt-driver = driver { $driver }
pt-kernel-params = Add to the boot loader and reboot: { $params }
pt-setup-done = Done. Reboot so vfio-pci takes the GPU at boot.
pt-setup-bound = The GPU is on vfio-pci for this session already.
pt-attach-done = { $vm } will use the GPU at { $gpu } from its next boot. macOS shows on the monitor plugged into it; QEMU's window shows only OpenCore and the installer.
pt-detach-done = { $vm } is back on the virtual GPU.
pt-oclp-done = { $vm }'s OpenCore is ready for OCLP's patches (the original is kept at { $backup }).
pt-oclp-already = { $vm }'s OpenCore was already ready for OCLP's patches.
pt-oclp-not-needed = This GPU has a native driver on this macOS; it needs no OCLP.
pt-vm-running = Shut { $vm } down first.
pt-vm-missing = reims-ctl knows no VM named { $vm }.
pt-gpu-missing = No GPU at { $gpu }. See the GPUs with cuvee passthrough list.
nova-gpu = Physical GPU
nova-gpu-none = Virtual GPU only
nova-gpu-hint = Hand an AMD or NVIDIA Kepler GPU to the VM for Metal on macOS { $macos }.
nova-gpu-no-candidate = No GPU in this computer works with macOS { $macos }. The VM runs without acceleration.
