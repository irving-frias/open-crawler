//! Revisión de ortografía determinista (sin IA/NLP) del contenido visible.
//!
//! Estrategia: diccionario de **erratas conocidas**, no wordlist completa.
//! Solo se marca lo que seguro está mal, así nombres propios, marcas y
//! términos técnicos nunca dan falsos positivos.
//!
//! - Inglés: crates `typos` + `typos-dict` (Rust puro, sin libhunspell: en CI
//!   Windows no hay diccionarios del sistema).
//! - Español: lista curada de formas inequívocamente erróneas (sin tilde
//!   obligatoria y confusiones comunes). Nada que dependa de contexto
//!   (`si`/`sí`, `mas`/`más`, `esta`/`está`, `halla`/`haya`, `valla`/`baya`…).

use std::borrow::Cow;
use std::collections::HashSet;

use scraper::{Html, Selector};
use typos::Dictionary;

/// Máximo de erratas distintas reportadas por página.
pub const MAX_SPELLING_TYPOS: usize = 25;

/// Fuente del texto donde apareció la errata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellingSource {
    Title,
    Heading,
    Paragraph,
}

impl SpellingSource {
    pub fn label(self) -> &'static str {
        match self {
            SpellingSource::Title => "title",
            SpellingSource::Heading => "heading",
            SpellingSource::Paragraph => "paragraph",
        }
    }
}

/// Una errata detectada con su corrección sugerida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellingTypo {
    pub word: String,
    pub suggestion: String,
    pub source: SpellingSource,
}

/// Idiomas a chequear, derivados de `<html lang>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellingLang {
    Es,
    En,
    Both,
}

pub fn detect_lang(html_lang: Option<&str>) -> SpellingLang {
    match html_lang {
        Some(l) if l.to_ascii_lowercase().starts_with("es") => SpellingLang::Es,
        Some(l) if l.to_ascii_lowercase().starts_with("en") => SpellingLang::En,
        _ => SpellingLang::Both,
    }
}

/// Formas españolas incorrectas en todo contexto → corrección.
/// Ordenadas alfabéticamente (búsqueda binaria); el test
/// `spanish_typos_sorted` lo verifica.
const SPANISH_TYPOS: &[(&str, &str)] = &[
    ("academico", "académico"),
    ("ademas", "además"),
    ("agil", "ágil"),
    ("album", "álbum"),
    ("albumes", "álbumes"),
    ("aleman", "alemán"),
    ("anadir", "añadir"),
    ("angulo", "ángulo"),
    ("arabe", "árabe"),
    ("arana", "araña"),
    ("arbol", "árbol"),
    ("arboles", "árboles"),
    ("area", "área"),
    ("articulo", "artículo"),
    ("artistica", "artística"),
    ("artistico", "artístico"),
    ("asi", "así"),
    ("atencion", "atención"),
    ("automatica", "automática"),
    ("automatico", "automático"),
    ("automovil", "automóvil"),
    ("aveces", "a veces"),
    ("aver", "a ver"),
    ("aveses", "a veces"),
    ("avion", "avión"),
    ("azucar", "azúcar"),
    ("basica", "básica"),
    ("basico", "básico"),
    ("bebia", "bebía"),
    ("boligrafo", "bolígrafo"),
    ("buzon", "buzón"),
    ("cafe", "café"),
    ("caia", "caía"),
    ("calculo", "cálculo"),
    ("camion", "camión"),
    ("campeon", "campeón"),
    ("cancion", "canción"),
    ("capitulo", "capítulo"),
    ("caracter", "carácter"),
    ("caracteres", "carácteres"),
    ("cientifica", "científica"),
    ("cientifico", "científico"),
    ("circulo", "círculo"),
    ("clasica", "clásica"),
    ("clasico", "clásico"),
    ("clinica", "clínica"),
    ("clinico", "clínico"),
    ("codigo", "código"),
    ("cojer", "coger"),
    ("comia", "comía"),
    ("comunicacion", "comunicación"),
    ("conocia", "conocía"),
    ("corazon", "corazón"),
    ("correjir", "corregir"),
    ("creeria", "creería"),
    ("creia", "creía"),
    ("daria", "daría"),
    ("deberia", "debería"),
    ("debil", "débil"),
    ("decia", "decía"),
    ("decian", "decían"),
    ("decicion", "decisión"),
    ("decision", "decisión"),
    ("deficit", "déficit"),
    ("dejaria", "dejaría"),
    ("desicion", "decisión"),
    ("despues", "después"),
    ("dia", "día"),
    ("dificil", "difícil"),
    ("direccion", "dirección"),
    ("diria", "diría"),
    ("dirijir", "dirigir"),
    ("docil", "dócil"),
    ("dolar", "dólar"),
    ("dolares", "dólares"),
    ("dormia", "dormía"),
    ("ductil", "dúctil"),
    ("economia", "economía"),
    ("educacion", "educación"),
    ("electronica", "electrónica"),
    ("electronico", "electrónico"),
    ("encantaria", "encantaría"),
    ("escojer", "escoger"),
    ("escribia", "escribía"),
    ("especifica", "específica"),
    ("especifico", "específico"),
    ("espectaculo", "espectáculo"),
    ("estacion", "estación"),
    ("estadistica", "estadística"),
    ("estadistico", "estadístico"),
    ("estaria", "estaría"),
    ("examenes", "exámenes"),
    ("exijir", "exigir"),
    ("expresion", "expresión"),
    ("facil", "fácil"),
    ("fertil", "fértil"),
    ("fisica", "física"),
    ("fisico", "físico"),
    ("frances", "francés"),
    ("frio", "frío"),
    ("funcion", "función"),
    ("grafica", "gráfica"),
    ("grafico", "gráfico"),
    ("guia", "guía"),
    ("gustaria", "gustaría"),
    ("habia", "había"),
    ("habian", "habían"),
    ("habil", "hábil"),
    ("habitat", "hábitat"),
    ("habria", "habría"),
    ("haci", "así"),
    ("haria", "haría"),
    ("haser", "hacer"),
    ("hasi", "así"),
    ("haver", "haber"),
    ("hechar", "echar"),
    ("heroe", "héroe"),
    ("historica", "histórica"),
    ("historico", "histórico"),
    ("imagenes", "imágenes"),
    ("imajen", "imagen"),
    ("imajenes", "imágenes"),
    ("impresion", "impresión"),
    ("informacion", "información"),
    ("ingles", "inglés"),
    ("inmovil", "inmóvil"),
    ("inutil", "inútil"),
    ("iria", "iría"),
    ("jamas", "jamás"),
    ("japones", "japonés"),
    ("jenio", "genio"),
    ("jente", "gente"),
    ("jovenes", "jóvenes"),
    ("lapices", "lápices"),
    ("lapiz", "lápiz"),
    ("leccion", "lección"),
    ("leia", "leía"),
    ("leon", "león"),
    ("lider", "líder"),
    ("lideres", "líderes"),
    ("linea", "línea"),
    ("llegaria", "llegaría"),
    ("logica", "lógica"),
    ("logico", "lógico"),
    ("magica", "mágica"),
    ("magico", "mágico"),
    ("manana", "mañana"),
    ("margenes", "márgenes"),
    ("master", "máster"),
    ("matematica", "matemática"),
    ("matematico", "matemático"),
    ("medica", "médica"),
    ("medico", "médico"),
    ("metodo", "método"),
    ("miercoles", "miércoles"),
    ("mio", "mío"),
    ("mision", "misión"),
    ("monton", "montón"),
    ("movil", "móvil"),
    ("musica", "música"),
    ("musico", "músico"),
    ("nacion", "nación"),
    ("nose", "no sé"),
    ("nucleo", "núcleo"),
    ("numero", "número"),
    ("oia", "oía"),
    ("ojala", "ojalá"),
    ("opcion", "opción"),
    ("opinion", "opinión"),
    ("ordenes", "órdenes"),
    ("origenes", "orígenes"),
    ("osea", "o sea"),
    ("pagina", "página"),
    ("pasaria", "pasaría"),
    ("pasion", "pasión"),
    ("pedia", "pedía"),
    ("pensaria", "pensaría"),
    ("podia", "podía"),
    ("podian", "podían"),
    ("podria", "podría"),
    ("politica", "política"),
    ("politico", "político"),
    ("pondria", "pondría"),
    ("ponia", "ponía"),
    ("porfavor", "por favor"),
    ("portatil", "portátil"),
    ("portugues", "portugués"),
    ("poster", "póster"),
    ("practica", "práctica"),
    ("practico", "práctico"),
    ("presion", "presión"),
    ("probicion", "prohibición"),
    ("prohibicion", "prohibición"),
    ("protejer", "proteger"),
    ("publico", "público"),
    ("queria", "quería"),
    ("querian", "querían"),
    ("querria", "querría"),
    ("quimica", "química"),
    ("quimico", "químico"),
    ("quizas", "quizás"),
    ("rapido", "rápido"),
    ("raton", "ratón"),
    ("reaccion", "reacción"),
    ("recojer", "recoger"),
    ("rectangulo", "rectángulo"),
    ("regimen", "régimen"),
    ("region", "región"),
    ("reia", "reía"),
    ("religion", "religión"),
    ("resumenes", "resúmenes"),
    ("rio", "río"),
    ("sabado", "sábado"),
    ("sabia", "sabía"),
    ("sabian", "sabían"),
    ("sabria", "sabría"),
    ("saldria", "saldría"),
    ("salia", "salía"),
    ("seguia", "seguía"),
    ("segun", "según"),
    ("sentia", "sentía"),
    ("sesion", "sesión"),
    ("simbolo", "símbolo"),
    ("situacion", "situación"),
    ("sofa", "sofá"),
    ("sonreia", "sonreía"),
    ("subia", "subía"),
    ("tabu", "tabú"),
    ("tambien", "también"),
    ("tecnica", "técnica"),
    ("tecnico", "técnico"),
    ("tecnologia", "tecnología"),
    ("telefono", "teléfono"),
    ("television", "televisión"),
    ("tendria", "tendría"),
    ("tenia", "tenía"),
    ("tenian", "tenían"),
    ("teorica", "teórica"),
    ("teorico", "teórico"),
    ("tia", "tía"),
    ("tio", "tío"),
    ("titulo", "título"),
    ("tomaria", "tomaría"),
    ("trafico", "tráfico"),
    ("tragica", "trágica"),
    ("tragico", "trágico"),
    ("traia", "traía"),
    ("trailer", "tráiler"),
    ("triangulo", "triángulo"),
    ("tunel", "túnel"),
    ("tuneles", "túneles"),
    ("ultima", "última"),
    ("ultimo", "último"),
    ("unica", "única"),
    ("unico", "único"),
    ("union", "unión"),
    ("util", "útil"),
    ("utiles", "útiles"),
    ("vehiculo", "vehículo"),
    ("veia", "veía"),
    ("vendria", "vendría"),
    ("venia", "venía"),
    ("veria", "vería"),
    ("version", "versión"),
    ("vinculo", "vínculo"),
    ("virgenes", "vírgenes"),
    ("vision", "visión"),
    ("vivia", "vivía"),
    ("volatil", "volátil"),
    ("volumenes", "volúmenes"),
];

/// Diccionario combinado ES/EN sobre los mapas de erratas conocidas.
struct SpellingDictionary {
    lang: SpellingLang,
}

impl SpellingDictionary {
    fn new(lang: SpellingLang) -> Self {
        Self { lang }
    }

    /// Primera corrección del diccionario inglés (`typos-dict`).
    fn correct_en(word: &str) -> Option<&'static str> {
        typos_dict::WORD
            .find(&unicase::UniCase::unicode(word))
            .and_then(|corrections| corrections.first().copied())
    }

    /// Corrección de la lista curada española (búsqueda binaria).
    fn correct_es(word: &str) -> Option<&'static str> {
        let lower = word.to_lowercase();
        SPANISH_TYPOS
            .binary_search_by(|(key, _)| key.cmp(&lower.as_str()))
            .ok()
            .map(|idx| SPANISH_TYPOS[idx].1)
    }
}

impl Dictionary for SpellingDictionary {
    fn correct_ident<'s>(
        &'s self,
        _ident: typos::tokens::Identifier<'_>,
    ) -> Option<typos::Status<'s>> {
        // Sin veredicto a nivel de identificador: que se parta en palabras.
        None
    }

    fn correct_word<'s>(&'s self, word: typos::tokens::Word<'_>) -> Option<typos::Status<'s>> {
        let token = word.token();
        let suggestion = match self.lang {
            SpellingLang::Es => Self::correct_es(token),
            SpellingLang::En => Self::correct_en(token),
            SpellingLang::Both => Self::correct_es(token).or_else(|| Self::correct_en(token)),
        };
        suggestion.map(|s| {
            let adjusted = case_correct(s, token);
            typos::Status::Corrections(vec![adjusted])
        })
    }
}

/// Ajusta la capitalización de la sugerencia a la de la palabra original
/// ("Teh" → "The", "TEH" → "THE"; "pagina" → "página" sin cambios).
fn case_correct(suggestion: &'static str, token: &str) -> Cow<'static, str> {
    let mut chars = token.chars();
    let first_upper = chars.next().is_some_and(|c| c.is_uppercase());
    if !first_upper {
        return Cow::Borrowed(suggestion);
    }
    let all_upper = token
        .chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase())
        && token.chars().any(|c| c.is_alphabetic());
    if all_upper {
        return Cow::Owned(suggestion.to_uppercase());
    }
    let mut out = String::with_capacity(suggestion.len());
    let mut s_chars = suggestion.chars();
    if let Some(c) = s_chars.next() {
        out.extend(c.to_uppercase());
    }
    out.extend(s_chars);
    Cow::Owned(out)
}

/// Erratas en el texto visible de la página: `<title>`, `h1–h6` y `<p>`.
/// Deduplica por palabra (minúsculas) y capa en `MAX_SPELLING_TYPOS`.
pub fn check_visible_text(document: &Html, html_lang: Option<&str>) -> Vec<SpellingTypo> {
    let dict = SpellingDictionary::new(detect_lang(html_lang));
    let tokenizer = typos::tokens::Tokenizer::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<SpellingTypo> = Vec::new();

    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = document.select(&sel).next() {
            check_text_into(
                &el.text().collect::<Vec<_>>().join(" "),
                SpellingSource::Title,
                &tokenizer,
                &dict,
                &mut seen,
                &mut out,
            );
        }
    }
    if let Ok(sel) = Selector::parse("h1, h2, h3, h4, h5, h6") {
        for el in document.select(&sel) {
            check_text_into(
                &el.text().collect::<Vec<_>>().join(" "),
                SpellingSource::Heading,
                &tokenizer,
                &dict,
                &mut seen,
                &mut out,
            );
            if out.len() >= MAX_SPELLING_TYPOS {
                break;
            }
        }
    }
    if let Ok(sel) = Selector::parse("p") {
        for el in document.select(&sel) {
            check_text_into(
                &el.text().collect::<Vec<_>>().join(" "),
                SpellingSource::Paragraph,
                &tokenizer,
                &dict,
                &mut seen,
                &mut out,
            );
            if out.len() >= MAX_SPELLING_TYPOS {
                break;
            }
        }
    }

    out
}

fn check_text_into(
    text: &str,
    source: SpellingSource,
    tokenizer: &typos::tokens::Tokenizer,
    dict: &SpellingDictionary,
    seen: &mut HashSet<String>,
    out: &mut Vec<SpellingTypo>,
) {
    for typo in typos::check_str(text, tokenizer, dict) {
        if out.len() >= MAX_SPELLING_TYPOS {
            return;
        }
        let word = typo.typo.into_owned();
        if !seen.insert(word.to_lowercase()) {
            continue;
        }
        let suggestion = match typo.corrections {
            typos::Status::Corrections(corrections) => {
                corrections.into_iter().next().map(|s| s.into_owned())
            }
            _ => None,
        };
        if let Some(suggestion) = suggestion.filter(|s| !s.is_empty()) {
            out.push(SpellingTypo {
                word,
                suggestion,
                source,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_typos_sorted() {
        let mut keys: Vec<&str> = SPANISH_TYPOS.iter().map(|(k, _)| *k).collect();
        keys.sort_unstable();
        let current: Vec<&str> = SPANISH_TYPOS.iter().map(|(k, _)| *k).collect();
        assert_eq!(current, keys, "SPANISH_TYPOS debe estar ordenada");
    }

    #[test]
    fn detect_lang_variants() {
        assert_eq!(detect_lang(Some("es")), SpellingLang::Es);
        assert_eq!(detect_lang(Some("es-MX")), SpellingLang::Es);
        assert_eq!(detect_lang(Some("en-US")), SpellingLang::En);
        assert_eq!(detect_lang(Some("fr")), SpellingLang::Both);
        assert_eq!(detect_lang(None), SpellingLang::Both);
    }

    #[test]
    fn spanish_typo_detected() {
        let html = Html::parse_document(
            "<html lang=\"es\"><head><title>Pagina de prueba</title></head>\
             <body><p>Esta es una pagina con informacion tambien.</p></body></html>",
        );
        let typos = check_visible_text(&html, Some("es"));
        // "pagina" aparece en título y párrafo: se deduplica (primera vez, título).
        assert_eq!(typos.len(), 3, "typos: {typos:?}");
        let words: Vec<&str> = typos.iter().map(|t| t.word.as_str()).collect();
        assert!(words.contains(&"Pagina"), "falta Pagina: {words:?}");
        assert!(
            words.contains(&"informacion"),
            "falta informacion: {words:?}"
        );
        assert!(words.contains(&"tambien"), "falta tambien: {words:?}");
        let info = typos.iter().find(|t| t.word == "informacion").unwrap();
        assert_eq!(info.suggestion, "información");
        assert_eq!(info.source, SpellingSource::Paragraph);
    }

    #[test]
    fn english_typo_detected() {
        let html = Html::parse_document(
            "<html lang=\"en\"><head><title>Teh test page</title></head>\
             <body><h1>Recieve our newslatter</h1></body></html>",
        );
        let typos = check_visible_text(&html, Some("en"));
        let words: Vec<&str> = typos.iter().map(|t| t.word.as_str()).collect();
        assert!(words.contains(&"Teh"), "falta Teh: {words:?}");
        assert!(words.contains(&"Recieve"), "falta Recieve: {words:?}");
        let teh = typos.iter().find(|t| t.word == "Teh").unwrap();
        assert_eq!(teh.suggestion, "The");
        assert_eq!(teh.source, SpellingSource::Title);
    }

    #[test]
    fn valid_words_and_brands_ignored() {
        let html = Html::parse_document(
            "<html lang=\"en\"><head><title>OpenCrawler SvelteKit guide</title></head>\
             <body><p>The quick brown fox jumps over the lazy dog.</p></body></html>",
        );
        let typos = check_visible_text(&html, Some("en"));
        assert!(typos.is_empty(), "falsos positivos: {typos:?}");
    }

    #[test]
    fn ambiguous_spanish_forms_ignored() {
        // `mas`, `si`, `esta` son válidas según contexto: nunca se marcan.
        let html = Html::parse_document(
            "<html lang=\"es\"><body><p>Si esta mas cerca, ven.</p></body></html>",
        );
        let typos = check_visible_text(&html, Some("es"));
        assert!(typos.is_empty(), "falsos positivos: {typos:?}");
    }

    #[test]
    fn typos_deduped_and_capped() {
        let mut paras = String::new();
        for _ in 0..40 {
            paras.push_str("<p>Esta pagina tambien tiene informacion.</p>");
        }
        let html = Html::parse_document(&format!("<html lang=\"es\"><body>{paras}</body></html>"));
        let typos = check_visible_text(&html, Some("es"));
        assert_eq!(typos.len(), 3, "dedupe por palabra: {typos:?}");
    }
}
