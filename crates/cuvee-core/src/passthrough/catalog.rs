//! Which GPUs macOS drives, by PCI ID, and from which version to which.
//!
//! Sources: Apple's driver history as the Dortania GPU Buyers Guide and the OpenCore Legacy
//! Patcher documentation describe it (both public). The ranges are the chips' PCI device-ID
//! blocks; a card whose maker changed its ID may need a spoof, which Cuvée does not do. This is
//! a starting point to confirm on real cards (see PENDENCIAS.md).

use crate::macos::Version;

/// A GPU family, as far as macOS cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// AMD GCN 1 to 3: Tahiti, Pitcairn, Bonaire, Hawaii, Tonga, Fiji (R9 200/300, Fury).
    AmdLegacyGcn,
    /// AMD Polaris 10/11/20/21/30: RX 460 to 590.
    Polaris,
    /// AMD Polaris 12 (Lexa: RX 540/550) and the RX 580 2048SP: need an ID spoof.
    PolarisSpoof,
    /// AMD Vega 10: Vega 56/64, Frontier Edition.
    Vega10,
    /// AMD Vega 20: Radeon VII.
    Vega20,
    /// AMD Navi 10: RX 5600 and 5700.
    Navi10,
    /// AMD Navi 14: RX 5500.
    Navi14,
    /// AMD Navi 21: RX 6800 and 6900.
    Navi21,
    /// AMD Navi 23: RX 6600.
    Navi23,
    /// Other AMD GPUs (Navi 22/24, RDNA 3 and newer, very old chips): no macOS driver.
    AmdUnsupported,
    /// NVIDIA Kepler: GTX 600/700 (most), GT 710/730 (GK208).
    Kepler,
    /// NVIDIA Maxwell and Pascal: GTX 900/1000. Only NVIDIA's Web Driver on 10.13.
    MaxwellPascal,
    /// Other NVIDIA GPUs (Turing, Ampere, Ada, Blackwell, Fermi): no macOS driver.
    NvidiaUnsupported,
    /// Intel graphics: Cuvée does not pass an iGPU to the VM.
    Intel,
    Other,
}

/// How a family runs on one macOS version, when passed to the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// macOS has the driver.
    Native,
    /// macOS has the driver, with a known catch (the Fluent id says which).
    NativeCaveat(&'static str),
    /// macOS dropped the driver; OpenCore Legacy Patcher's root patches bring it back.
    OclpRootPatch,
    /// Only with NVIDIA's Web Driver, which NVIDIA no longer offers. Not set up by Cuvée.
    WebDriver,
    Unsupported,
}

impl Support {
    /// Whether Cuvée offers passthrough for this pairing.
    pub fn usable(self) -> bool {
        matches!(
            self,
            Support::Native | Support::NativeCaveat(_) | Support::OclpRootPatch
        )
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Support::Native | Support::NativeCaveat(_) => "pt-support-native",
            Support::OclpRootPatch => "pt-support-oclp",
            Support::WebDriver => "pt-support-webdriver",
            Support::Unsupported => "pt-support-none",
        }
    }
}

const fn within(d: u16, lo: u16, hi: u16) -> bool {
    d >= lo && d <= hi
}

impl Family {
    pub fn of(vendor: u16, device: u16) -> Family {
        let d = device;
        match vendor {
            0x1002 => {
                if d == 0x6fdf || within(d, 0x6980, 0x699f) {
                    Family::PolarisSpoof
                } else if within(d, 0x67c0, 0x67ff) {
                    Family::Polaris
                } else if within(d, 0x6860, 0x687f) {
                    Family::Vega10
                } else if within(d, 0x66a0, 0x66af) {
                    Family::Vega20
                } else if within(d, 0x7310, 0x731f) {
                    Family::Navi10
                } else if within(d, 0x7340, 0x734f) {
                    Family::Navi14
                } else if within(d, 0x73a0, 0x73bf) {
                    Family::Navi21
                } else if within(d, 0x73e0, 0x73ff) {
                    Family::Navi23
                } else if within(d, 0x6780, 0x67bf) // Tahiti, Hawaii
                    || within(d, 0x6800, 0x681f) // Pitcairn, Curaçao
                    || within(d, 0x6640, 0x665f) // Bonaire
                    || within(d, 0x6920, 0x693f) // Tonga
                    || within(d, 0x7300, 0x730f)
                // Fiji
                {
                    Family::AmdLegacyGcn
                } else {
                    Family::AmdUnsupported
                }
            }
            0x10de => {
                if within(d, 0x0fc0, 0x0fff) // GK107
                    || within(d, 0x1000, 0x103f) // GK110
                    || within(d, 0x1180, 0x11ff) // GK104, GK106
                    || within(d, 0x1280, 0x12bf)
                // GK208
                {
                    Family::Kepler
                } else if within(d, 0x1340, 0x143f) // GM108, GM107, GM204, GM206
                    || within(d, 0x17c0, 0x17ff) // GM200
                    || within(d, 0x15f0, 0x15ff) // GP100
                    || within(d, 0x1b00, 0x1b3f) // GP102
                    || within(d, 0x1b80, 0x1bff) // GP104
                    || within(d, 0x1c00, 0x1c3f) // GP106
                    || within(d, 0x1c80, 0x1cbf) // GP107
                    || within(d, 0x1d00, 0x1d3f)
                // GP108
                {
                    Family::MaxwellPascal
                } else {
                    Family::NvidiaUnsupported
                }
            }
            0x8086 => Family::Intel,
            _ => Family::Other,
        }
    }

    /// The Fluent id of the family's name.
    pub fn label_key(self) -> &'static str {
        match self {
            Family::AmdLegacyGcn => "pt-family-amd-gcn",
            Family::Polaris => "pt-family-polaris",
            Family::PolarisSpoof => "pt-family-polaris-spoof",
            Family::Vega10 => "pt-family-vega10",
            Family::Vega20 => "pt-family-vega20",
            Family::Navi10 => "pt-family-navi10",
            Family::Navi14 => "pt-family-navi14",
            Family::Navi21 => "pt-family-navi21",
            Family::Navi23 => "pt-family-navi23",
            Family::AmdUnsupported => "pt-family-amd-other",
            Family::Kepler => "pt-family-kepler",
            Family::MaxwellPascal => "pt-family-maxwell-pascal",
            Family::NvidiaUnsupported => "pt-family-nvidia-other",
            Family::Intel => "pt-family-intel",
            Family::Other => "pt-family-other",
        }
    }

    /// Whether the chip is known to stay stuck after a VM shuts down (the AMD "reset bug"),
    /// so the next start needs a host reboot or the vendor-reset module.
    pub fn reset_bug(self) -> bool {
        matches!(
            self,
            Family::Vega10
                | Family::Vega20
                | Family::Navi10
                | Family::Navi14
                | Family::AmdLegacyGcn
        )
    }

    /// How this family runs on `v`. macOS 26 is not offered: it is accelerated through
    /// reims-vgpu, and passthrough is for 10.13 to 15.
    pub fn support(self, v: Version) -> Support {
        use Version::*;
        if v == Tahoe {
            return Support::Unsupported;
        }
        let navi_black_screen = Support::NativeCaveat("pt-caveat-navi-agdp");
        match self {
            Family::Polaris => Support::Native,
            Family::Vega10 => Support::Native,
            Family::Vega20 => match v {
                HighSierra => Support::Unsupported,
                _ => Support::Native,
            },
            Family::Navi10 | Family::Navi14 => match v {
                HighSierra | Mojave => Support::Unsupported,
                _ => navi_black_screen,
            },
            Family::Navi21 => match v {
                HighSierra | Mojave | Catalina => Support::Unsupported,
                _ => navi_black_screen,
            },
            Family::Navi23 => match v {
                HighSierra | Mojave | Catalina | BigSur => Support::Unsupported,
                _ => navi_black_screen,
            },
            Family::AmdLegacyGcn => match v {
                Ventura | Sonoma | Sequoia => Support::OclpRootPatch,
                _ => Support::Native,
            },
            Family::Kepler => match v {
                Ventura | Sonoma | Sequoia => Support::OclpRootPatch,
                _ => Support::Native,
            },
            Family::MaxwellPascal => match v {
                HighSierra => Support::WebDriver,
                _ => Support::Unsupported,
            },
            Family::PolarisSpoof
            | Family::AmdUnsupported
            | Family::NvidiaUnsupported
            | Family::Intel
            | Family::Other => Support::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macos::Version::*;

    #[test]
    fn known_cards_land_in_their_family() {
        assert_eq!(Family::of(0x1002, 0x67df), Family::Polaris); // RX 580
        assert_eq!(Family::of(0x1002, 0x67ef), Family::Polaris); // RX 560
        assert_eq!(Family::of(0x1002, 0x6fdf), Family::PolarisSpoof); // RX 580 2048SP
        assert_eq!(Family::of(0x1002, 0x699f), Family::PolarisSpoof); // RX 550
        assert_eq!(Family::of(0x1002, 0x687f), Family::Vega10); // Vega 64
        assert_eq!(Family::of(0x1002, 0x66af), Family::Vega20); // Radeon VII
        assert_eq!(Family::of(0x1002, 0x731f), Family::Navi10); // RX 5700 XT
        assert_eq!(Family::of(0x1002, 0x73bf), Family::Navi21); // RX 6800 XT
        assert_eq!(Family::of(0x1002, 0x73ff), Family::Navi23); // RX 6600 XT
        assert_eq!(Family::of(0x1002, 0x73df), Family::AmdUnsupported); // RX 6700 XT
        assert_eq!(Family::of(0x1002, 0x744c), Family::AmdUnsupported); // RX 7900 XTX
        assert_eq!(Family::of(0x1002, 0x6798), Family::AmdLegacyGcn); // R9 280X
        assert_eq!(Family::of(0x10de, 0x1184), Family::Kepler); // GTX 770
        assert_eq!(Family::of(0x10de, 0x128b), Family::Kepler); // GT 710
        assert_eq!(Family::of(0x10de, 0x1b81), Family::MaxwellPascal); // GTX 1070
        assert_eq!(Family::of(0x10de, 0x2f04), Family::NvidiaUnsupported); // RTX 5070
        assert_eq!(Family::of(0x8086, 0x3e92), Family::Intel);
    }

    #[test]
    fn polaris_runs_on_every_passthrough_version() {
        for v in [
            Sequoia, Sonoma, Ventura, Monterey, BigSur, Catalina, Mojave, HighSierra,
        ] {
            assert_eq!(Family::Polaris.support(v), Support::Native, "{v:?}");
        }
        assert!(!Family::Polaris.support(Tahoe).usable());
    }

    #[test]
    fn dropped_drivers_need_oclp() {
        assert_eq!(Family::Kepler.support(Monterey), Support::Native);
        assert_eq!(Family::Kepler.support(Ventura), Support::OclpRootPatch);
        assert_eq!(
            Family::AmdLegacyGcn.support(Sequoia),
            Support::OclpRootPatch
        );
        assert_eq!(
            Family::MaxwellPascal.support(HighSierra),
            Support::WebDriver
        );
        assert!(!Family::MaxwellPascal.support(HighSierra).usable());
        assert_eq!(Family::Navi23.support(BigSur), Support::Unsupported);
        assert!(Family::Navi23.support(Monterey).usable());
        assert!(!Family::NvidiaUnsupported.support(Sequoia).usable());
    }
}
