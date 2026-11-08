//! Translations with Fluent. Every string a person sees comes from a `.ftl` file in
//! `locales/<lang>/cuvee.ftl`, compiled into the binary. `pt-BR` is the default and the
//! fallback: a message missing from another language falls back to it.

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use std::collections::BTreeMap;
use unic_langid::{langid, LanguageIdentifier};

pub struct Language {
    pub tag: &'static str,
    /// The language's own name, shown in the language switch.
    pub native_name: &'static str,
    source: &'static str,
}

pub const LANGUAGES: &[Language] = &[
    Language {
        tag: "pt-BR",
        native_name: "Português (Brasil)",
        source: include_str!("../locales/pt-BR/cuvee.ftl"),
    },
    Language {
        tag: "en",
        native_name: "English",
        source: include_str!("../locales/en/cuvee.ftl"),
    },
];

pub fn default_language() -> &'static str {
    LANGUAGES[0].tag
}

/// The language the desktop asks for, if the translations have it: `LANG=pt_BR.UTF-8` → pt-BR.
pub fn system_language() -> Option<&'static str> {
    let raw = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))?;
    let tag = raw.split('.').next()?.replace('_', "-");
    LANGUAGES
        .iter()
        .find(|l| l.tag.eq_ignore_ascii_case(&tag))
        .or_else(|| {
            let primary = tag.split('-').next()?;
            LANGUAGES
                .iter()
                .find(|l| l.tag.split('-').next() == Some(primary))
        })
        .map(|l| l.tag)
}

pub struct Locale {
    tag: &'static str,
    bundle: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
}

fn bundle_for(lang: &Language) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = lang.tag.parse().unwrap_or_else(|_| langid!("en"));
    let mut b = FluentBundle::new_concurrent(vec![id]);
    // Fluent wraps placeables in Unicode isolation marks; the GUI renders plain runs of text.
    b.set_use_isolating(false);
    let res = FluentResource::try_new(lang.source.to_string()).unwrap_or_else(|(res, errors)| {
        for e in errors {
            eprintln!("cuvee: {}: {e:?}", lang.tag);
        }
        res
    });
    let _ = b.add_resource(res);
    b
}

impl Locale {
    /// The locale for `tag`, or the default language when the tag is unknown.
    pub fn new(tag: &str) -> Self {
        let lang = LANGUAGES
            .iter()
            .find(|l| l.tag.eq_ignore_ascii_case(tag))
            .unwrap_or(&LANGUAGES[0]);
        let fallback = (lang.tag != LANGUAGES[0].tag).then(|| bundle_for(&LANGUAGES[0]));
        Self {
            tag: lang.tag,
            bundle: bundle_for(lang),
            fallback,
        }
    }

    pub fn tag(&self) -> &'static str {
        self.tag
    }

    fn format(
        bundle: &FluentBundle<FluentResource>,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Option<String> {
        let msg = bundle.get_message(key)?;
        let pattern = msg.value()?;
        let mut errors = vec![];
        let s = bundle.format_pattern(pattern, args, &mut errors);
        Some(s.into_owned())
    }

    /// The message `key`, or the fallback's, or the key itself between brackets so a missing
    /// translation is visible instead of silent.
    pub fn t(&self, key: &str) -> String {
        self.t_args(key, &[])
    }

    pub fn t_args(&self, key: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let fargs = (!args.is_empty()).then(|| {
            let mut a = FluentArgs::new();
            for (k, v) in args {
                a.set(k.to_string(), v.clone());
            }
            a
        });
        Self::format(&self.bundle, key, fargs.as_ref())
            .or_else(|| {
                self.fallback
                    .as_ref()
                    .and_then(|f| Self::format(f, key, fargs.as_ref()))
            })
            .unwrap_or_else(|| format!("[{key}]"))
    }

    /// A number argument: `loc.t_args("vms-count", &[("n", num(3))])`.
    pub fn num(n: impl Into<f64>) -> FluentValue<'static> {
        FluentValue::from(n.into())
    }

    pub fn str(s: &str) -> FluentValue<'static> {
        FluentValue::from(s.to_string())
    }
}

/// Every message id each language defines: the test below keeps them in sync.
pub fn message_ids(lang: &Language) -> BTreeMap<String, ()> {
    let res = FluentResource::try_new(lang.source.to_string()).unwrap_or_else(|(r, _)| r);
    res.entries()
        .filter_map(|e| match e {
            fluent_syntax::ast::Entry::Message(m) => Some((m.id.name.to_string(), ())),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_every_message() {
        let base = message_ids(&LANGUAGES[0]);
        for lang in &LANGUAGES[1..] {
            let ids = message_ids(lang);
            let missing: Vec<_> = base.keys().filter(|k| !ids.contains_key(*k)).collect();
            assert!(missing.is_empty(), "{} lacks {:?}", lang.tag, missing);
            let extra: Vec<_> = ids.keys().filter(|k| !base.contains_key(*k)).collect();
            assert!(
                extra.is_empty(),
                "{} has {:?} that pt-BR lacks",
                lang.tag,
                extra
            );
        }
    }

    #[test]
    fn missing_keys_are_visible() {
        let l = Locale::new("en");
        assert_eq!(l.t("no-such-key"), "[no-such-key]");
    }

    #[test]
    fn plurals_format() {
        let l = Locale::new("pt-BR");
        assert_eq!(
            l.t_args("painel-title", &[("n", Locale::num(3))]),
            "Três máquinas."
        );
        assert_eq!(
            l.t_args("painel-title", &[("n", Locale::num(1))]),
            "Uma máquina."
        );
        let e = Locale::new("en");
        assert_eq!(
            e.t_args("painel-title", &[("n", Locale::num(3))]),
            "Three machines."
        );
    }
}
