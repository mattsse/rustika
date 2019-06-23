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
    pub content_lang: Option<Language>,

    pub target_lang: Language,
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum Translator {
    Lingo24,
    Google,
    /// use another translator, like `org.apache.tika.language.translate.YandexTranslator`
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
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Translator::Lingo24
    }
}
