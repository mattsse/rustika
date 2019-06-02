#[derive(Debug, Clone)]
pub struct Language(pub String);

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
