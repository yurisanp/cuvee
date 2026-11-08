//! The macOS versions a VM can run, and what to expect from each.
//!
//! Sources: reims-util's installer list (10.13 to 26 online, 11 to 26 offline), reims-vgpu's
//! README (acceleration on macOS 26 alone), and Apple's own GPU driver history for passthrough.
//! The behaviour table is a starting point to validate on real hardware; `Behavior::level` is
//! what the GUI colours.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Version {
    Tahoe,
    Sequoia,
    Sonoma,
    Ventura,
    Monterey,
    BigSur,
    Catalina,
    Mojave,
    HighSierra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Installer {
    OnlineOrOffline,
    OnlineOnly,
}

/// The default Mac model reims-util presents, per version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    IMac20_1,
    IMacPro1_1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suggested {
    pub cpus: u32,
    pub memory_gib: u32,
}

impl Version {
    /// Newest first, as the picker shows them.
    pub const ALL: [Version; 9] = [
        Version::Tahoe,
        Version::Sequoia,
        Version::Sonoma,
        Version::Ventura,
        Version::Monterey,
        Version::BigSur,
        Version::Catalina,
        Version::Mojave,
        Version::HighSierra,
    ];

    /// The number shown in the picker: "26", "10.13".
    pub fn number(self) -> &'static str {
        match self {
            Version::Tahoe => "26",
            Version::Sequoia => "15",
            Version::Sonoma => "14",
            Version::Ventura => "13",
            Version::Monterey => "12",
            Version::BigSur => "11",
            Version::Catalina => "10.15",
            Version::Mojave => "10.14",
            Version::HighSierra => "10.13",
        }
    }

    /// Apple's marketing name; not translated.
    pub fn name(self) -> &'static str {
        match self {
            Version::Tahoe => "Tahoe",
            Version::Sequoia => "Sequoia",
            Version::Sonoma => "Sonoma",
            Version::Ventura => "Ventura",
            Version::Monterey => "Monterey",
            Version::BigSur => "Big Sur",
            Version::Catalina => "Catalina",
            Version::Mojave => "Mojave",
            Version::HighSierra => "High Sierra",
        }
    }

    /// The slug reims-util and reims-ctl take (`--macos tahoe`).
    pub fn slug(self) -> &'static str {
        match self {
            Version::Tahoe => "tahoe",
            Version::Sequoia => "sequoia",
            Version::Sonoma => "sonoma",
            Version::Ventura => "ventura",
            Version::Monterey => "monterey",
            Version::BigSur => "big-sur",
            Version::Catalina => "catalina",
            Version::Mojave => "mojave",
            Version::HighSierra => "high-sierra",
        }
    }

    /// A version from what a person types: its slug (`sequoia`, `big-sur`), its number (`15`,
    /// `10.13`) or its name (`Big Sur`), in any case.
    pub fn parse(s: &str) -> Option<Version> {
        let s = s.trim().to_ascii_lowercase();
        Version::ALL
            .into_iter()
            .find(|v| v.slug() == s || v.number() == s || v.name().to_ascii_lowercase() == s)
    }

    /// Whether reims-vgpu accelerates this guest's graphics (Metal through the host's Vulkan).
    pub fn accelerated(self) -> bool {
        matches!(self, Version::Tahoe)
    }

    pub fn installer(self) -> Installer {
        match self {
            Version::Catalina | Version::Mojave | Version::HighSierra => Installer::OnlineOnly,
            _ => Installer::OnlineOrOffline,
        }
    }

    pub fn model(self) -> Model {
        match self {
            Version::Mojave | Version::HighSierra => Model::IMacPro1_1,
            _ => Model::IMac20_1,
        }
    }

    pub fn suggested(self) -> Suggested {
        match self {
            Version::Tahoe => Suggested {
                cpus: 8,
                memory_gib: 16,
            },
            Version::Sequoia | Version::Sonoma | Version::Ventura => Suggested {
                cpus: 4,
                memory_gib: 8,
            },
            Version::Monterey | Version::BigSur => Suggested {
                cpus: 4,
                memory_gib: 6,
            },
            Version::Catalina | Version::Mojave | Version::HighSierra => Suggested {
                cpus: 2,
                memory_gib: 4,
            },
        }
    }

    /// The Fluent id of the one-line tip the picker shows for this version.
    pub fn tip_key(self) -> &'static str {
        match self {
            Version::Tahoe => "tip-tahoe",
            Version::Sequoia => "tip-sequoia",
            Version::Sonoma => "tip-sonoma",
            Version::Ventura => "tip-ventura",
            Version::Monterey => "tip-monterey",
            Version::BigSur => "tip-big-sur",
            Version::Catalina => "tip-catalina",
            Version::Mojave => "tip-mojave",
            Version::HighSierra => "tip-high-sierra",
        }
    }

    pub fn group(self) -> Group {
        match self {
            Version::Tahoe => Group::Tahoe,
            Version::Sequoia | Version::Sonoma | Version::Ventura => Group::VenturaToSequoia,
            Version::Monterey | Version::BigSur => Group::BigSurMonterey,
            Version::Catalina | Version::Mojave | Version::HighSierra => {
                Group::HighSierraToCatalina
            }
        }
    }
}

/// The behaviour guide groups versions that behave alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    Tahoe,
    VenturaToSequoia,
    BigSurMonterey,
    HighSierraToCatalina,
}

impl Group {
    pub const ALL: [Group; 4] = [
        Group::Tahoe,
        Group::VenturaToSequoia,
        Group::BigSurMonterey,
        Group::HighSierraToCatalina,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Group::Tahoe => "group-tahoe",
            Group::VenturaToSequoia => "group-ventura-sequoia",
            Group::BigSurMonterey => "group-big-sur-monterey",
            Group::HighSierraToCatalina => "group-high-sierra-catalina",
        }
    }

    pub fn behaviors(self) -> [Behavior; 5] {
        use Aspect::*;
        use Rating::*;
        match self {
            Group::Tahoe => [
                Behavior::new(VideoNoGpu, Good, "beh-tahoe-video"),
                Behavior::new(AmdPassthrough, Good, "beh-tahoe-amd"),
                Behavior::new(NvidiaPassthrough, Bad, "beh-nvidia-no-driver"),
                Behavior::new(Network, Good, "beh-network-ok"),
                Behavior::new(AppleServices, Limited, "beh-apple-services"),
            ],
            Group::VenturaToSequoia => [
                Behavior::new(VideoNoGpu, Limited, "beh-video-usable"),
                Behavior::new(AmdPassthrough, Good, "beh-amd-native-wide"),
                Behavior::new(NvidiaPassthrough, Limited, "beh-nvidia-kepler-oclp"),
                Behavior::new(Network, Good, "beh-network-ok"),
                Behavior::new(AppleServices, Limited, "beh-apple-services"),
            ],
            Group::BigSurMonterey => [
                Behavior::new(VideoNoGpu, Limited, "beh-video-usable"),
                Behavior::new(AmdPassthrough, Good, "beh-amd-native-wide"),
                Behavior::new(NvidiaPassthrough, Limited, "beh-nvidia-kepler"),
                Behavior::new(Network, Good, "beh-network-ok"),
                Behavior::new(AppleServices, Limited, "beh-apple-services"),
            ],
            Group::HighSierraToCatalina => [
                Behavior::new(VideoNoGpu, Limited, "beh-video-slow"),
                Behavior::new(AmdPassthrough, Good, "beh-amd-polaris-vega"),
                Behavior::new(NvidiaPassthrough, Limited, "beh-nvidia-kepler-pascal"),
                Behavior::new(Network, Good, "beh-network-ok"),
                Behavior::new(AppleServices, Limited, "beh-apple-services"),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    VideoNoGpu,
    AmdPassthrough,
    NvidiaPassthrough,
    Network,
    AppleServices,
}

impl Aspect {
    pub fn label_key(self) -> &'static str {
        match self {
            Aspect::VideoNoGpu => "aspect-video-no-gpu",
            Aspect::AmdPassthrough => "aspect-amd-passthrough",
            Aspect::NvidiaPassthrough => "aspect-nvidia-passthrough",
            Aspect::Network => "aspect-network",
            Aspect::AppleServices => "aspect-apple-services",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Good,
    Limited,
    Bad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Behavior {
    pub aspect: Aspect,
    pub rating: Rating,
    pub text_key: &'static str,
}

impl Behavior {
    const fn new(aspect: Aspect, rating: Rating, text_key: &'static str) -> Self {
        Self {
            aspect,
            rating,
            text_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_tahoe_is_accelerated() {
        assert!(Version::Tahoe.accelerated());
        assert!(Version::ALL.iter().filter(|v| v.accelerated()).count() == 1);
    }

    #[test]
    fn parses_what_people_type() {
        assert_eq!(Version::parse("Sequoia"), Some(Version::Sequoia));
        assert_eq!(Version::parse("15"), Some(Version::Sequoia));
        assert_eq!(Version::parse("10.13"), Some(Version::HighSierra));
        assert_eq!(Version::parse("big sur"), Some(Version::BigSur));
        assert_eq!(Version::parse("big-sur"), Some(Version::BigSur));
        assert_eq!(Version::parse("9"), None);
    }

    #[test]
    fn slugs_match_reims_util() {
        let slugs: Vec<_> = Version::ALL.iter().map(|v| v.slug()).collect();
        assert_eq!(
            slugs,
            [
                "tahoe",
                "sequoia",
                "sonoma",
                "ventura",
                "monterey",
                "big-sur",
                "catalina",
                "mojave",
                "high-sierra"
            ]
        );
    }
}
