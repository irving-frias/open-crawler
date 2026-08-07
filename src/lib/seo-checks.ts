import { getLocale } from '$lib/paraglide/runtime.js';

interface CheckTranslation {
  message: string;
  guidance: string;
}

interface CheckDictEntry {
  en: CheckTranslation;
  es: CheckTranslation;
}

const DICT: Record<string, CheckDictEntry> = {
  // ==================== META & CONTENT ====================
  title_present: {
    en: {
      message: 'Page has no <title>',
      guidance: 'Add a unique, descriptive <title> tag to every page.',
    },
    es: {
      message: 'La página no tiene <title>',
      guidance: 'Añade una etiqueta <title> única y descriptiva a cada página.',
    },
  },
  title_length: {
    en: {
      message: 'Title length: {0} chars (target 30-65)',
      guidance: 'Keep titles between 30 and 65 characters so they render fully in search results.',
    },
    es: {
      message: 'Longitud del título: {0} caracteres (objetivo 30-65)',
      guidance: 'Mantén los títulos entre 30 y 65 caracteres para que se muestren completos en los resultados de búsqueda.',
    },
  },
  meta_description_present: {
    en: {
      message: 'Page has no meta description',
      guidance: 'Write a unique 50-160 character meta description for each page.',
    },
    es: {
      message: 'La página no tiene meta description',
      guidance: 'Escribe una meta description única de 50-160 caracteres para cada página.',
    },
  },
  meta_description_length: {
    en: {
      message: 'Meta description length: {0} chars (target 50-160)',
      guidance: 'Adjust the meta description to between 50 and 160 characters.',
    },
    es: {
      message: 'Longitud de la meta description: {0} caracteres (objetivo 50-160)',
      guidance: 'Ajusta la meta description para que tenga entre 50 y 160 caracteres.',
    },
  },
  h1_present: {
    en: {
      message: 'Page has no <h1>',
      guidance: "Add exactly one <h1> summarizing the page's main topic.",
    },
    es: {
      message: 'La página no tiene <h1>',
      guidance: 'Añade exactamente un <h1> que resuma el tema principal de la página.',
    },
  },
  h1_count: {
    en: {
      message: 'Exactly one <h1> on the page',
      guidance: 'Use a single <h1>; additional headings should use h2-h6.',
    },
    es: {
      message: 'Debe haber exactamente un <h1> en la página',
      guidance: 'Usa un único <h1>; los demás encabezados deben usar h2-h6.',
    },
  },
  heading_hierarchy: {
    en: {
      message: 'Heading hierarchy has no skipped levels',
      guidance: 'Do not skip heading levels (h1 → h3). Use a logical outline.',
    },
    es: {
      message: 'La jerarquía de encabezados salta niveles',
      guidance: 'No saltes niveles de encabezado (h1 → h3). Usa un esquema lógico.',
    },
  },
  h1_title_match: {
    en: {
      message: 'H1 and title share topic keywords',
      guidance: 'Align the <title> and <h1> so both clearly describe the page topic.',
    },
    es: {
      message: 'El H1 y el título comparten palabras clave del tema',
      guidance: 'Alinea el <title> y el <h1> para que ambos describan claramente el tema de la página.',
    },
  },
  word_count: {
    en: {
      message: 'Word count: {0} (recommended ≥ 300)',
      guidance: 'Expand thin content to at least 300 words for better ranking.',
    },
    es: {
      message: 'Recuento de palabras: {0} (recomendado ≥ 300)',
      guidance: 'Amplía el contenido escaso a al menos 300 palabras para mejorar el posicionamiento.',
    },
  },
  keyword_density: {
    en: {
      message: 'Top keyword density: {0}% (target 0.5-5%)',
      guidance: 'Avoid keyword stuffing; use the main keyword naturally a few times.',
    },
    es: {
      message: 'Densidad de la palabra clave principal: {0}% (objetivo 0.5-5%)',
      guidance: 'Evita el relleno de palabras clave; usa la palabra clave principal de forma natural algunas veces.',
    },
  },
  internal_links: {
    en: {
      message: 'Page contains at least one internal link',
      guidance: 'Link to other pages of your site so crawlers can discover content.',
    },
    es: {
      message: 'La página no contiene ningún enlace interno',
      guidance: 'Enlaza a otras páginas de tu sitio para que los rastreadores descubran el contenido.',
    },
  },
  outbound_links: {
    en: {
      message: 'Page contains at least one outbound link',
      guidance: 'Cite related external sources to add context and authority.',
    },
    es: {
      message: 'La página no contiene ningún enlace externo',
      guidance: 'Cita fuentes externas relacionadas para aportar contexto y autoridad.',
    },
  },

  // ==================== TECHNICAL & MOBILE ====================
  https_used: {
    en: {
      message: 'Not served over HTTPS',
      guidance: 'Serve the site over HTTPS with a valid certificate.',
    },
    es: {
      message: 'No se sirve a través de HTTPS',
      guidance: 'Sirve el sitio a través de HTTPS con un certificado válido.',
    },
  },
  status_ok: {
    en: {
      message: 'HTTP status {0}',
      guidance: 'Fix the page so it returns a 200 status instead of an error.',
    },
    es: {
      message: 'Estado HTTP {0}',
      guidance: 'Corrige la página para que devuelva un estado 200 en lugar de un error.',
    },
  },
  viewport: {
    en: {
      message: 'No viewport meta tag',
      guidance: 'Add <meta name="viewport" content="width=device-width, initial-scale=1">.',
    },
    es: {
      message: 'No hay etiqueta meta viewport',
      guidance: 'Añade <meta name="viewport" content="width=device-width, initial-scale=1">.',
    },
  },
  favicon: {
    en: {
      message: 'No favicon declared',
      guidance: 'Add a favicon via <link rel="icon">.',
    },
    es: {
      message: 'No se declara ningún favicon',
      guidance: 'Añade un favicon mediante <link rel="icon">.',
    },
  },
  charset: {
    en: {
      message: 'No charset declared',
      guidance: 'Declare <meta charset="utf-8"> in the <head>.',
    },
    es: {
      message: 'No se declara ningún charset',
      guidance: 'Declara <meta charset="utf-8"> en el <head>.',
    },
  },
  doctype: {
    en: {
      message: 'Missing HTML5 doctype',
      guidance: 'Start the document with <!DOCTYPE html>.',
    },
    es: {
      message: 'Falta el doctype HTML5',
      guidance: 'Empieza el documento con <!DOCTYPE html>.',
    },
  },
  canonical_present: {
    en: {
      message: 'No canonical tag',
      guidance: 'Add <link rel="canonical"> pointing to the page\'s preferred URL.',
    },
    es: {
      message: 'No hay etiqueta canonical',
      guidance: 'Añade <link rel="canonical"> que apunte a la URL preferida de la página.',
    },
  },
  indexable: {
    en: {
      message: 'Page is marked noindex',
      guidance: 'Remove the noindex directive if this page should appear in search results.',
    },
    es: {
      message: 'La página está marcada como noindex',
      guidance: 'Elimina la directiva noindex si esta página debe aparecer en los resultados de búsqueda.',
    },
  },
  html_lang: {
    en: {
      message: 'Missing HTML lang attribute',
      guidance: 'Set the lang attribute on <html> (e.g. lang="en").',
    },
    es: {
      message: 'Falta el atributo lang en el HTML',
      guidance: 'Define el atributo lang en <html> (p. ej. lang="en").',
    },
  },
  url_length: {
    en: {
      message: 'URL length: {0} chars (≤ 100 recommended)',
      guidance: 'Shorten long, deeply nested URLs.',
    },
    es: {
      message: 'Longitud de la URL: {0} caracteres (≤ 100 recomendado)',
      guidance: 'Acorta las URL largas y muy anidadas.',
    },
  },
  url_underscores: {
    en: {
      message: 'URL path contains no underscores',
      guidance: 'Use hyphens instead of underscores in URL paths.',
    },
    es: {
      message: 'La ruta de la URL contiene guiones bajos',
      guidance: 'Usa guiones en lugar de guiones bajos en las rutas de las URL.',
    },
  },

  // ==================== SOCIAL & OPEN GRAPH ====================
  og_title: {
    en: {
      message: 'Missing og:title',
      guidance: 'Add <meta property="og:title"> matching the page title.',
    },
    es: {
      message: 'Falta og:title',
      guidance: 'Añade <meta property="og:title"> acorde con el título de la página.',
    },
  },
  og_description: {
    en: {
      message: 'Missing og:description',
      guidance: 'Add a concise <meta property="og:description">.',
    },
    es: {
      message: 'Falta og:description',
      guidance: 'Añade un <meta property="og:description"> conciso.',
    },
  },
  og_image: {
    en: {
      message: 'Missing og:image',
      guidance: 'Add an <meta property="og:image"> (1200×630 recommended).',
    },
    es: {
      message: 'Falta og:image',
      guidance: 'Añade un <meta property="og:image"> (1200×630 recomendado).',
    },
  },
  og_image_alt: {
    en: {
      message: 'Missing og:image:alt',
      guidance: 'Describe the Open Graph image with og:image:alt.',
    },
    es: {
      message: 'Falta og:image:alt',
      guidance: 'Describe la imagen de Open Graph con og:image:alt.',
    },
  },
  og_url: {
    en: {
      message: 'Missing og:url',
      guidance: 'Add og:url pointing to the canonical page URL.',
    },
    es: {
      message: 'Falta og:url',
      guidance: 'Añade og:url que apunte a la URL canónica de la página.',
    },
  },
  og_type: {
    en: {
      message: 'Missing og:type',
      guidance: 'Add <meta property="og:type"> (e.g. website or article).',
    },
    es: {
      message: 'Falta og:type',
      guidance: 'Añade <meta property="og:type"> (p. ej. website o article).',
    },
  },
  og_site_name: {
    en: {
      message: 'Missing og:site_name',
      guidance: 'Add <meta property="og:site_name"> with your brand name.',
    },
    es: {
      message: 'Falta og:site_name',
      guidance: 'Añade <meta property="og:site_name"> con el nombre de tu marca.',
    },
  },
  twitter_card: {
    en: {
      message: 'Missing twitter:card',
      guidance: 'Add <meta name="twitter:card" content="summary_large_image">.',
    },
    es: {
      message: 'Falta twitter:card',
      guidance: 'Añade <meta name="twitter:card" content="summary_large_image">.',
    },
  },
  twitter_title: {
    en: {
      message: 'Missing twitter:title',
      guidance: 'Add a twitter:title meta tag.',
    },
    es: {
      message: 'Falta twitter:title',
      guidance: 'Añade una etiqueta meta twitter:title.',
    },
  },
  twitter_description: {
    en: {
      message: 'Missing twitter:description',
      guidance: 'Add a twitter:description meta tag.',
    },
    es: {
      message: 'Falta twitter:description',
      guidance: 'Añade una etiqueta meta twitter:description.',
    },
  },
  twitter_image: {
    en: {
      message: 'Missing twitter:image',
      guidance: 'Add a twitter:image meta tag.',
    },
    es: {
      message: 'Falta twitter:image',
      guidance: 'Añade una etiqueta meta twitter:image.',
    },
  },

  // ==================== ACCESSIBILITY ====================
  img_alt: {
    en: {
      message: 'All images have alt text',
      guidance: 'Add descriptive alt attributes to every <img>.',
    },
    es: {
      message: 'Hay imágenes sin texto alternativo (alt)',
      guidance: 'Añade atributos alt descriptivos a cada <img>.',
    },
  },
  img_dimensions: {
    en: {
      message: 'Images with explicit dimensions: {0}/{1}',
      guidance: 'Specify width/height on images to avoid layout shift.',
    },
    es: {
      message: 'Imágenes con dimensiones explícitas: {0}/{1}',
      guidance: 'Especifica width/height en las imágenes para evitar saltos de layout.',
    },
  },
  form_labels: {
    en: {
      message: 'All form inputs have labels',
      guidance: 'Associate every input with a <label> via the for/id attributes.',
    },
    es: {
      message: 'Hay campos de formulario sin etiqueta',
      guidance: 'Asocia cada input con un <label> mediante los atributos for/id.',
    },
  },
  input_ids: {
    en: {
      message: 'Form controls have id attributes',
      guidance: 'Give each input/select/textarea an id for label association.',
    },
    es: {
      message: 'Hay controles de formulario sin atributo id',
      guidance: 'Asigna un id a cada input/select/textarea para asociar la etiqueta.',
    },
  },
  aria_controls: {
    en: {
      message: 'Form controls are accessible by ARIA',
      guidance: 'Add aria-label or aria-labelledby to form controls lacking labels.',
    },
    es: {
      message: 'Hay controles de formulario inaccesibles por ARIA',
      guidance: 'Añade aria-label o aria-labelledby a los controles sin etiqueta.',
    },
  },
  empty_link_text: {
    en: {
      message: 'All links have accessible text',
      guidance: 'Give every link visible text or an aria-label.',
    },
    es: {
      message: 'Hay enlaces sin texto accesible',
      guidance: 'Da a cada enlace texto visible o un aria-label.',
    },
  },
  main_landmark: {
    en: {
      message: 'Page has a <main> landmark',
      guidance: 'Wrap the primary content in <main>.',
    },
    es: {
      message: 'La página no tiene un landmark <main>',
      guidance: 'Envuelve el contenido principal en <main>.',
    },
  },
  header_landmark: {
    en: {
      message: 'Page has a <header> landmark',
      guidance: 'Add a <header> element for the page banner.',
    },
    es: {
      message: 'La página no tiene un landmark <header>',
      guidance: 'Añade un elemento <header> para el banner de la página.',
    },
  },
  footer_landmark: {
    en: {
      message: 'Page has a <footer> landmark',
      guidance: 'Add a <footer> element.',
    },
    es: {
      message: 'La página no tiene un landmark <footer>',
      guidance: 'Añade un elemento <footer>.',
    },
  },
  nav_landmark: {
    en: {
      message: 'Page has a <nav> landmark',
      guidance: 'Wrap navigation links in a <nav> element.',
    },
    es: {
      message: 'La página no tiene un landmark <nav>',
      guidance: 'Envuelve los enlaces de navegación en un elemento <nav>.',
    },
  },
  nesting_valid: {
    en: {
      message: 'Element nesting follows HTML rules',
      guidance: 'Fix invalid element nesting flagged by the semantic analysis.',
    },
    es: {
      message: 'El anidamiento de elementos no sigue las reglas HTML',
      guidance: 'Corrige el anidamiento inválido que detecta el análisis semántico.',
    },
  },

  // ==================== PERFORMANCE ====================
  page_weight: {
    en: {
      message: 'Page weight: {0} KB (limit ~1.5 MB)',
      guidance: 'Reduce page weight by minifying HTML, CSS and inline assets.',
    },
    es: {
      message: 'Peso de la página: {0} KB (límite ~1.5 MB)',
      guidance: 'Reduce el peso de la página minificando HTML, CSS y recursos en línea.',
    },
  },
  load_time: {
    en: {
      message: 'Server load time: {0} ms',
      guidance: 'Improve server response time (target under 2.5 s).',
    },
    es: {
      message: 'Tiempo de carga del servidor: {0} ms',
      guidance: 'Mejora el tiempo de respuesta del servidor (objetivo: menos de 2.5 s).',
    },
  },
  image_optimization: {
    en: {
      message: 'Images declare dimensions for lazy layout',
      guidance: 'Add width/height to images so browsers can reserve space.',
    },
    es: {
      message: 'Hay imágenes sin dimensiones declaradas',
      guidance: 'Añade width/height a las imágenes para que el navegador reserve el espacio.',
    },
  },
  resource_hints: {
    en: {
      message: 'Resource hints (preconnect/preload) present',
      guidance: 'Preconnect to critical origins and preload key resources.',
    },
    es: {
      message: 'No hay resource hints (preconnect/preload)',
      guidance: 'Conecta con preconnect a orígenes críticos y precarga recursos clave.',
    },
  },
  pagespeed: {
    en: {
      message: 'PageSpeed (Lighthouse) performance: {0}/100',
      guidance: 'Follow the Lighthouse performance audits to improve this score.',
    },
    es: {
      message: 'Rendimiento PageSpeed (Lighthouse): {0}/100',
      guidance: 'Sigue las auditorías de rendimiento de Lighthouse para mejorar esta puntuación.',
    },
  },

  // ==================== AI READABILITY ====================
  readability_score: {
    en: {
      message: 'Flesch reading ease: {0}/100 (≥ 50 recommended)',
      guidance: 'Simplify language and shorten sentences to raise readability.',
    },
    es: {
      message: 'Legibilidad Flesch: {0}/100 (≥ 50 recomendado)',
      guidance: 'Simplifica el lenguaje y acorta las frases para mejorar la legibilidad.',
    },
  },
  flesch_kincaid_grade: {
    en: {
      message: 'Flesch-Kincaid grade: {0} (≤ 12 recommended)',
      guidance: 'Target a reading grade level most of your audience can read.',
    },
    es: {
      message: 'Grado Flesch-Kincaid: {0} (≤ 12 recomendado)',
      guidance: 'Apunta a un nivel de lectura que la mayoría de tu audiencia pueda leer.',
    },
  },
  sentence_length: {
    en: {
      message: 'Average sentence length: {0} words (≤ 25 recommended)',
      guidance: 'Break long sentences into shorter, single-idea sentences.',
    },
    es: {
      message: 'Longitud media de la frase: {0} palabras (≤ 25 recomendado)',
      guidance: 'Divide las frases largas en frases más cortas con una sola idea.',
    },
  },
  paragraph_structure: {
    en: {
      message: 'Paragraph count: {0} (≥ 3 recommended)',
      guidance: 'Structure the content into short paragraphs and subheadings.',
    },
    es: {
      message: 'Número de párrafos: {0} (≥ 3 recomendado)',
      guidance: 'Estructura el contenido en párrafos cortos y subtítulos.',
    },
  },
  semantic_html: {
    en: {
      message: 'Semantic landmarks present (main/header/footer/nav)',
      guidance: 'Use semantic HTML so AI and assistive tools understand the page structure.',
    },
    es: {
      message: 'No hay landmarks semánticos (main/header/footer/nav)',
      guidance: 'Usa HTML semántico para que la IA y las herramientas de asistencia entiendan la estructura.',
    },
  },
  content_present: {
    en: {
      message: 'Page has enough text to extract meaning',
      guidance: 'Add substantive text content to the page.',
    },
    es: {
      message: 'La página no tiene suficiente texto',
      guidance: 'Añade contenido de texto sustancial a la página.',
    },
  },

  // ==================== SXO / AEO / AIO ====================
  structured_data: {
    en: {
      message: 'No JSON-LD structured data',
      guidance: 'Add JSON-LD structured data describing the page\'s content.',
    },
    es: {
      message: 'No hay datos estructurados JSON-LD',
      guidance: 'Añade datos estructurados JSON-LD que describan el contenido.',
    },
  },
  faq_schema: {
    en: {
      message: 'FAQPage schema present',
      guidance: 'Mark up questions and answers with FAQPage schema.',
    },
    es: {
      message: 'No hay esquema FAQPage',
      guidance: 'Marca las preguntas y respuestas con el esquema FAQPage.',
    },
  },
  howto_schema: {
    en: {
      message: 'HowTo schema present',
      guidance: 'Mark up step-by-step instructions with HowTo schema.',
    },
    es: {
      message: 'No hay esquema HowTo',
      guidance: 'Marca las instrucciones paso a paso con el esquema HowTo.',
    },
  },
  breadcrumb_schema: {
    en: {
      message: 'BreadcrumbList schema present',
      guidance: 'Add BreadcrumbList schema for navigation paths.',
    },
    es: {
      message: 'No hay esquema BreadcrumbList',
      guidance: 'Añade el esquema BreadcrumbList para las rutas de navegación.',
    },
  },
  article_schema: {
    en: {
      message: 'Article schema present',
      guidance: 'Mark up articles with Article/NewsArticle/BlogPosting schema.',
    },
    es: {
      message: 'No hay esquema Article',
      guidance: 'Marca los artículos con el esquema Article/NewsArticle/BlogPosting.',
    },
  },
  organization_schema: {
    en: {
      message: 'Organization/WebSite schema present',
      guidance: 'Add Organization or WebSite schema to define entity and brand.',
    },
    es: {
      message: 'No hay esquema Organization/WebSite',
      guidance: 'Añade el esquema Organization o WebSite para definir la entidad y la marca.',
    },
  },
  question_headings: {
    en: {
      message: 'Question-style headings: {0} (≥ 1 recommended)',
      guidance: "Use headings phrased as questions users actually ask (e.g. 'How does…').",
    },
    es: {
      message: 'Encabezados en forma de pregunta: {0} (≥ 1 recomendado)',
      guidance: "Usa encabezados formulados como preguntas que los usuarios hacen (p. ej. '¿Cómo funciona…?').",
    },
  },
  direct_answer: {
    en: {
      message: 'Page can answer the query in its opening text',
      guidance: 'Lead with a direct, concise answer to the main question.',
    },
    es: {
      message: 'La página no responde la consulta en el texto inicial',
      guidance: 'Comienza con una respuesta directa y concisa a la pregunta principal.',
    },
  },
};

const READABILITY_NO_TEXT = {
  en: {
    message: 'Not enough text to measure readability',
    guidance: 'Simplify language and shorten sentences to raise readability.',
  },
  es: {
    message: 'No hay suficiente texto para medir la legibilidad',
    guidance: 'Simplifica el lenguaje y acorta las frases para mejorar la legibilidad.',
  },
} as const;

export interface LocalizedCheck {
  message: string;
  guidance: string;
}

function applyParams(template: string, params: string[]): string {
  return template.replace(/\{(\d+)\}/g, (_, idx: string) => params[Number(idx)] ?? '');
}

function extractParams(enMessage: string, evidence?: string | null): string[] {
  if (evidence != null && evidence !== '') return evidence.split('/');
  const nums = enMessage.match(/\d+(?:\.\d+)?/g) ?? [];
  return nums;
}

export function localizeSeoCheck(
  id: string,
  message: string,
  guidance: string,
  evidence?: string | null
): LocalizedCheck {
  let entry = DICT[id];
  if (id === 'readability_score' && !/Flesch reading ease/i.test(message)) {
    const noText = READABILITY_NO_TEXT;
    const tr = getLocale().startsWith('es') ? noText.es : noText.en;
    return { message: tr.message, guidance: tr.guidance };
  }
  if (!entry) return { message, guidance };
  const tr = getLocale().startsWith('es') ? entry.es : entry.en;
  const params = extractParams(message, evidence);
  return {
    message: applyParams(tr.message, params),
    guidance: applyParams(tr.guidance, params),
  };
}
