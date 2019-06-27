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

/// Represents an executed translation by the tika server
#[derive(Debug, Clone)]
pub struct Translation {
    /// the translated content in the `dest_lang`
    pub content: String,
    /// if a source language was supplied
    /// if the source language was auto detected, this is `None`
    pub src_lang: Option<Language>,
    /// the language, tika translated to
    pub dest_lang: Language,
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct TranslatorKey {
    /// the tika translator
    pub translator: Translator,
    /// api key of the translation service
    pub api_key: String,
}

impl TranslatorKey {
    /// key value entry for the property file, with the translator specific key and the api-key as value
    pub fn property_entry(&self) -> String {
        format!(
            "translator.{}={}",
            self.translator.property_key(),
            self.api_key
        )
    }

    pub fn google<T: ToString>(api_key: T) -> Self {
        TranslatorKey {
            translator: Translator::Google,
            api_key: api_key.to_string(),
        }
    }

    pub fn lingo24<T: ToString>(api_key: T) -> Self {
        TranslatorKey {
            translator: Translator::Lingo24,
            api_key: api_key.to_string(),
        }
    }

    pub fn yandex<T: ToString>(api_key: T) -> Self {
        TranslatorKey {
            translator: Translator::Yandex,
            api_key: api_key.to_string(),
        }
    }
}

/// represents how the api keys should be included
#[derive(Debug, Clone)]
pub enum TranslatorProperties {
    /// directory where property files are already available
    Dir(PathBuf),
    /// Translator keys added on demand
    Keys(Vec<TranslatorKey>),
}

impl TranslatorProperties {
    /// returns the folder where the property files are stored
    /// if only keys are supplied, the necessary files/folder will be created
    pub(crate) fn property_dir<P: AsRef<Path>>(&self, tika_path: P) -> Result<PathBuf> {
        match self {
            TranslatorProperties::Dir(dir) => Ok(dir.clone()),
            TranslatorProperties::Keys(keys) => {
                let root = tika_path.as_ref().join("language-keys");

                let dir = root.join("org/apache/tika/language/translate");

                fs::create_dir_all(&dir)?;
                debug!(
                    "Created {} directory for translator properties",
                    dir.display()
                );
                for key in keys {
                    let property_file =
                        dir.join(format!("translator.{}.properties", key.translator.id()));
                    fs::write(property_file, key.property_entry())?;
                }

                Ok(root)
            }
        }
    }
}

/// Available translators on the tika server
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

    /// unique identifier, lowercase class name without `translator`
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

    /// name of the translator specific identifier in the property file
    pub fn property_key(&self) -> &str {
        match self {
            Translator::Lingo24 => "user-key",
            Translator::Other(_) => "user-key",
            Translator::Google => "client-secret",
            Translator::Yandex => "api-key",
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Translator::Lingo24
    }
}
