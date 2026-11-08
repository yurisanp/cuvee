#!/bin/sh
# Cuvée: forget the udev rule after removal. The udmabuf module stays loaded
# until the next boot; the user's VMs in ~/.local/share/reims-ctl stay too.
if command -v udevadm >/dev/null 2>&1; then
  udevadm control --reload 2>/dev/null || true
fi
exit 0
