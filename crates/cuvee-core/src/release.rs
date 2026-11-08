//! Cuvée's GitHub releases: finding one, downloading its files and checking them.
//!
//! Every release carries the same file names, without the version in them, so the newest
//! one is always at `https://github.com/yurisanp/cuvee/releases/latest/download/<name>`:
//!
//! - [`CLI_ASSET`]: the `cuvee` command, a static binary;
//! - [`GUI_ASSET`]: the desktop app;
//! - [`RUNTIME_ASSET`]: QEMU with reims-vgpu, reims-ctl and reims-util (see [`crate::paths`]);
//! - [`SUMS_ASSET`]: the SHA-256 of each, as `sha256sum` writes them;
//! - optionally `<name>.minisig`, each file's signature, checked by [`crate::signature`].
//!
//! Downloads go through `curl` (or `wget`), which every distribution has and which keeps TLS
//! out of this binary. A file whose SHA-256 is not in `SHA256SUMS`, or does not match, is
//! deleted and refused.

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const REPO: &str = "yurisanp/cuvee";
pub const CLI_ASSET: &str = "cuvee-x86_64-linux-musl";
pub const GUI_ASSET: &str = "cuvee-gui-x86_64-linux";
pub const RUNTIME_ASSET: &str = "cuvee-runtime-x86_64-linux.tar.gz";
pub const SUMS_ASSET: &str = "SHA256SUMS";

/// Where a release's files come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The newest release on GitHub.
    Latest,
    /// A release by its tag, `v0.2.0`.
    Tag(String),
    /// A directory holding the files, as CI builds them (tests, offline installs).
    Dir(PathBuf),
}

impl Source {
    /// `$CUVEE_DIST_DIR`, else `$CUVEE_VERSION` (a tag or `latest`), else the newest release.
    pub fn from_env() -> Source {
        if let Some(d) = std::env::var_os("CUVEE_DIST_DIR").filter(|v| !v.is_empty()) {
            return Source::Dir(PathBuf::from(d));
        }
        match std::env::var("CUVEE_VERSION") {
            Ok(v) if !v.is_empty() && v != "latest" => Source::Tag(v),
            _ => Source::Latest,
        }
    }

    fn url(&self, name: &str) -> Option<String> {
        match self {
            Source::Latest => Some(format!(
                "https://github.com/{REPO}/releases/latest/download/{name}"
            )),
            Source::Tag(t) => Some(format!(
                "https://github.com/{REPO}/releases/download/{t}/{name}"
            )),
            Source::Dir(_) => None,
        }
    }

    /// The release's tag: asked of GitHub for [`Source::Latest`] (where its redirect points),
    /// read from a `VERSION` file for a [`Source::Dir`] (`None` without one).
    pub fn tag(&self) -> Result<Option<String>> {
        match self {
            Source::Tag(t) => Ok(Some(t.clone())),
            Source::Dir(d) => Ok(std::fs::read_to_string(d.join("VERSION"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())),
            Source::Latest => latest_tag().map(Some),
        }
    }
}

/// The newest release's tag, from where `releases/latest` redirects (no API, no rate limit).
pub fn latest_tag() -> Result<String> {
    let url = format!("https://github.com/{REPO}/releases/latest");
    let out = Command::new("curl")
        .args(["-fsSLI", "-o", "/dev/null", "-w", "%{url_effective}", &url])
        .output()
        .context("running curl (is it installed?)")?;
    if !out.status.success() {
        bail!(
            "could not reach {url}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    tag_from_url(&String::from_utf8_lossy(&out.stdout))
        .with_context(|| format!("{url} has no release yet"))
}

fn tag_from_url(url: &str) -> Option<String> {
    let tag = url.trim().rsplit_once("/tag/")?.1;
    (!tag.is_empty()).then(|| tag.to_string())
}

/// Downloads a release's files into a directory and checks each one.
pub struct Fetcher {
    source: Source,
    dir: PathBuf,
    sums: Option<HashMap<String, String>>,
}

impl Fetcher {
    /// Files land in `dir` (created if missing).
    pub fn new(source: Source, dir: impl Into<PathBuf>) -> Self {
        Self {
            source,
            dir: dir.into(),
            sums: None,
        }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Downloads `name` and checks its SHA-256 and, where the release has one, its signature.
    /// Returns the local path.
    pub fn fetch(&mut self, name: &str) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.dir)
            .with_context(|| format!("creating {}", self.dir.display()))?;
        if self.sums.is_none() {
            let p = self.get(SUMS_ASSET, true)?;
            let text = std::fs::read_to_string(&p)?;
            self.sums = Some(parse_sums(&text));
        }
        let path = self.get(name, true)?;
        let want = self
            .sums
            .as_ref()
            .and_then(|s| s.get(name))
            .cloned()
            .with_context(|| format!("{SUMS_ASSET} has no line for {name}"));
        let check = want.and_then(|want| {
            let got = sha256_file(&path)?;
            if got != want {
                bail!("{name}: SHA-256 is {got}, the release says {want}");
            }
            let sig = self.get(&format!("{name}.minisig"), false)?;
            crate::signature::verify(&path, Some(sig.as_path()).filter(|p| p.exists()))
        });
        if let Err(e) = check {
            let _ = std::fs::remove_file(&path);
            return Err(e);
        }
        Ok(path)
    }

    /// Copies or downloads `name` into the directory. With `required` false, a file the
    /// release does not have is `Ok` and leaves nothing behind.
    fn get(&self, name: &str, required: bool) -> Result<PathBuf> {
        let dest = self.dir.join(name);
        let _ = std::fs::remove_file(&dest);
        match &self.source {
            Source::Dir(d) => {
                let src = d.join(name);
                if !src.exists() {
                    if required {
                        bail!("{} is missing", src.display());
                    }
                    return Ok(dest);
                }
                std::fs::copy(&src, &dest).with_context(|| format!("copying {}", src.display()))?;
            }
            s => {
                let url = s.url(name).expect("a remote source has URLs");
                if !download(&url, &dest, required)? && required {
                    bail!("could not download {url}");
                }
            }
        }
        Ok(dest)
    }
}

/// Downloads `url` to `dest` with curl, else wget. `Ok(false)` when the server has no such
/// file (404) and `loud` is false; errors otherwise.
pub fn download(url: &str, dest: &Path, loud: bool) -> Result<bool> {
    let part = dest.with_extension("part");
    let status = if which("curl") {
        let mut c = Command::new("curl");
        c.args(["-fL", "--retry", "3", "--proto", "=https", "-o"])
            .arg(&part)
            .arg(url);
        if loud && std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            c.arg("--progress-bar");
        } else {
            c.arg("-sS");
        }
        if !loud {
            c.stderr(std::process::Stdio::null());
        }
        c.status().context("running curl")?
    } else if which("wget") {
        let mut c = Command::new("wget");
        c.args(["-q", "-O"]).arg(&part).arg(url);
        c.status().context("running wget")?
    } else {
        bail!("neither curl nor wget is installed");
    };
    if !status.success() {
        let _ = std::fs::remove_file(&part);
        if loud {
            bail!("downloading {url} failed");
        }
        return Ok(false);
    }
    std::fs::rename(&part, dest)?;
    Ok(true)
}

fn which(bin: &str) -> bool {
    crate::paths::which(bin).is_some()
}

/// `sha256sum`'s output: `<hex>  <name>` (or `<hex> *<name>`) a line.
pub fn parse_sums(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let (hash, name) = l.trim().split_once(char::is_whitespace)?;
            let name = name.trim_start().trim_start_matches('*');
            (hash.len() == 64 && !name.is_empty())
                .then(|| (name.to_string(), hash.to_ascii_lowercase()))
        })
        .collect()
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut f = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Unpacks a `.tar.gz` into `dest`, which must not exist yet. Unpacks into `dest.partial`
/// first, so a failed unpack never leaves a half runtime where a whole one is expected.
pub fn unpack_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let partial = dest.with_extension("partial");
    let _ = std::fs::remove_dir_all(&partial);
    std::fs::create_dir_all(&partial)?;
    let f = std::fs::File::open(archive)?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(f));
    tar.set_preserve_permissions(true);
    tar.set_overwrite(true);
    // `unpack` refuses entries that would land outside `partial` (`..`, absolute paths).
    tar.unpack(&partial)
        .with_context(|| format!("unpacking {}", archive.display()))?;
    std::fs::rename(&partial, dest)
        .with_context(|| format!("moving the runtime to {}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sha256sum_output() {
        let s = parse_sums(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  cuvee-x86_64-linux-musl\n\
             E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855 *SHA256SUMS\n\
             garbage\n",
        );
        assert_eq!(s.len(), 2);
        assert!(s["SHA256SUMS"].starts_with("e3b0"));
        assert!(s.contains_key("cuvee-x86_64-linux-musl"));
    }

    #[test]
    fn tags_come_from_the_redirect() {
        assert_eq!(
            tag_from_url("https://github.com/yurisanp/cuvee/releases/tag/v0.2.0\n").as_deref(),
            Some("v0.2.0")
        );
        assert_eq!(
            tag_from_url("https://github.com/yurisanp/cuvee/releases"),
            None
        );
    }

    #[test]
    fn fetch_checks_sums_from_a_directory() {
        let base = std::env::temp_dir().join(format!("cuvee-release-{}", std::process::id()));
        let dist = base.join("dist");
        std::fs::create_dir_all(&dist).unwrap();
        std::fs::write(dist.join("a"), b"").unwrap();
        std::fs::write(dist.join("b"), b"x").unwrap();
        std::fs::write(
            dist.join(SUMS_ASSET),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  a\n\
             e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  b\n",
        )
        .unwrap();
        let mut f = Fetcher::new(Source::Dir(dist), base.join("dl"));
        assert!(f.fetch("a").unwrap().exists());
        let err = f.fetch("b").unwrap_err().to_string();
        assert!(err.contains("SHA-256"), "{err}");
        assert!(!base.join("dl/b").exists());
        assert!(f.fetch("c").is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn unpacks_tarballs() {
        let base = std::env::temp_dir().join(format!("cuvee-unpack-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let tgz = base.join("t.tar.gz");
        {
            let f = std::fs::File::create(&tgz).unwrap();
            let gz = flate2::write::GzEncoder::new(f, flate2::Compression::fast());
            let mut b = tar::Builder::new(gz);
            let mut h = tar::Header::new_gnu();
            h.set_size(2);
            h.set_mode(0o755);
            h.set_cksum();
            b.append_data(&mut h, "bin/x", &b"hi"[..]).unwrap();
            b.into_inner().unwrap().finish().unwrap();
        }
        let dest = base.join("out");
        unpack_tar_gz(&tgz, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("bin/x")).unwrap(), b"hi");
        std::fs::remove_dir_all(&base).unwrap();
    }
}
