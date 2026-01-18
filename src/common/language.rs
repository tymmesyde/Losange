#[derive(Clone, Copy)]
pub struct Language {
    pub name: &'static str,
    pub code: &'static str,
    ietf: Option<&'static str>,
}

const POB_LANGUAGE: Language = Language {
    name: "Português (Brasil)",
    code: "pob",
    ietf: Some("pt-BR"),
};

const SPL_LANGUAGE: Language = Language {
    name: "Español (América)",
    code: "spl",
    ietf: Some("es-419"),
};

const FRC_LANGUAGE: Language = Language {
    name: "Français (Canada)",
    code: "frc",
    ietf: Some("fr-CA"),
};

const SPECIAL_LANGUAGES: [Language; 3] = [POB_LANGUAGE, SPL_LANGUAGE, FRC_LANGUAGE];

impl TryFrom<String> for Language {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Language {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains('-') {
            if let Some(lang) = SPECIAL_LANGUAGES
                .iter()
                .find(|lang| lang.ietf == Some(value))
            {
                return Ok(*lang);
            }

            let iso639_code = value.split('-').next().ok_or("Invalid IETF tag format")?;

            Self::from_iso639(iso639_code).ok_or("Unknown language from IETF BCP 47")
        } else {
            Self::from_iso639(value).ok_or("Unknown language from ISO 639")
        }
    }
}

impl Language {
    fn from_iso639(code: &str) -> Option<Self> {
        let lang = rust_iso639::from_code_1(code)
            .or_else(|| rust_iso639::from_code_2t(code))
            .or_else(|| rust_iso639::from_code_2b(code))
            .or_else(|| rust_iso639::from_code_3(code))?;

        Some(Self {
            name: lang.name,
            code: lang.code_3,
            ietf: None,
        })
    }
}
