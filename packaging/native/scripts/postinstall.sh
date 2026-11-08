#!/bin/sh
# Cuvée: load udmabuf now and let the kvm group open /dev/udmabuf.
# Runs for .deb (configure) and .rpm (%post); nothing here may fail the install.
getent group kvm >/dev/null 2>&1 || groupadd -r kvm 2>/dev/null || true
modprobe udmabuf 2>/dev/null || true
if command -v udevadm >/dev/null 2>&1; then
  udevadm control --reload 2>/dev/null || true
  udevadm trigger --subsystem-match=misc --action=change 2>/dev/null || true
fi
echo ">> Cuvée installed. Now, as your own user (no sudo):"
echo ">>   cuvee setup-host   # adds you to the kvm group"
echo ">>   cuvee doctor       # checks that everything is ready"
exit 0
