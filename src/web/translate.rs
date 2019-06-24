use crate::error::Result;
use std::path::Path;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Language(pub String);

impl Language {
    pub fn en() -> Self {
        "en".into()
    }

    pub fn de() -> Self {
        "de".into()
    }

    pub fn it() -> Self {
        "it".into()
    }

    pub fn fr() -> Self {
        "fr".into()
    }
}

impl<T: ToString> From<T> for Language {
    fn from(lang: T) -> Self {
        Language(lang.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Translation {
    pub content: String,

    pub src_lang: Option<Language>,

    pub dest_lang: Language,
}

#[derive(Debug, Clone)]
pub struct TranslatorKey {
    pub translator: Translator,
    pub api_key: String,
}

impl TranslatorKey {
    pub fn property_content(&self) -> String {
        format!("{}={}", self.translator.key_value(), self.api_key)
    }
}

#[derive(Debug, Clone)]
pub enum TranslatorProperties {
    Dir(PathBuf),
    Keys(Vec<TranslatorKey>),
}

impl TranslatorProperties {
    pub(crate) fn dir<P: AsRef<Path>>(&self, tika_path: P) -> Result<PathBuf> {
        match self {
            TranslatorProperties::Dir(dir) => Ok(dir.clone()),
            TranslatorProperties::Keys(keys) => {
                let dir = tika_path
                    .as_ref()
                    .join("language-keys/org/apache/tika/language/translate");

                let _ = fs::create_dir(&dir)?;

                for key in keys {
                    let property_file =
                        dir.join(format!("translator.{}.properties", key.translator.id()));
                    let _ = fs::write(property_file, key.property_content())?;
                }

                Ok(dir)
            }
        }
    }
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum Translator {
    Lingo24,
    Google,
    Yandex,
    /// use another translator, like `org.apache.tika.language.translate.MicrosoftTranslator`
    Other(String),
}

impl Translator {
    /// creates a new `Translator::Other` with the full java name
    pub fn other<T: Into<String>>(jvm_pkg_name: T) -> Self {
        Translator::Other(jvm_pkg_name.into())
    }

    /// the full java class name of the translator
    pub fn as_str(&self) -> &str {
        match self {
            Translator::Lingo24 => "org.apache.tika.language.translate.Lingo24Translator",
            Translator::Other(s) => s.as_str(),
            Translator::Google => "org.apache.tika.language.translate.GoogleTranslator",
            Translator::Yandex => "org.apache.tika.language.translate.YandexTranslator",
        }
    }

    pub fn id(&self) -> String {
        match self {
            Translator::Lingo24 => "lingo24".to_string(),
            Translator::Other(s) => s
                .replace("org.apache.tika.language.translate.", "")
                .to_lowercase()
                .replace("translator", ""),
            Translator::Google => "google".to_string(),
            Translator::Yandex => "yandex".to_string(),
        }
    }

    pub fn key_value(&self) -> &str {
        match self {
            Translator::Lingo24 => "user-key",
            Translator::Other(_) => "user-key",
            Translator::Google => "client-secret",
            Translator::Yandex => "translator.api-key",
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Translator::Lingo24
    }
}
