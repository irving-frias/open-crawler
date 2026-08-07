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

interface CheckFixEntry {
  en: { fix: string; expected: string };
  es: { fix: string; expected: string };
}

/**
 * Offline fix + expected-markup catalog for every audit check. Shown next to a
 * failing check so the user always sees how to make the error disappear,
 * without needing an AI call.
 */
const CHECK_FIXES: Record<string, CheckFixEntry> = {
  title_present: {
    en: { fix: 'Add a unique, descriptive <title> inside the <head>.', expected: '<head>\n  <title>Unique descriptive title</title>\n</head>' },
    es: { fix: 'Añade un <title> único y descriptivo dentro del <head>.', expected: '<head>\n  <title>Título único y descriptivo</title>\n</head>' },
  },
  title_length: {
    en: { fix: 'Shorten or lengthen the title to 30-65 characters so it renders fully in results.', expected: '<title>Between 30 and 65 characters</title>' },
    es: { fix: 'Acorta o alarga el título hasta 30-65 caracteres para que se muestre completo en los resultados.', expected: '<title>Entre 30 y 65 caracteres</title>' },
  },
  meta_description_present: {
    en: { fix: 'Add a meta description of 50-160 characters that summarizes the page.', expected: '<meta name="description" content="A 50-160 character summary of the page.">' },
    es: { fix: 'Añade una meta description de 50-160 caracteres que resuma la página.', expected: '<meta name="description" content="Un resumen de 50-160 caracteres de la página.">' },
  },
  meta_description_length: {
    en: { fix: 'Adjust the meta description to 50-160 characters.', expected: '<meta name="description" content="A correctly sized description.">' },
    es: { fix: 'Ajusta la meta description a 50-160 caracteres.', expected: '<meta name="description" content="Una descripción con el tamaño correcto.">' },
  },
  h1_present: {
    en: { fix: 'Add exactly one <h1> that summarizes the page topic.', expected: '<h1>Main topic of the page</h1>' },
    es: { fix: 'Añade exactamente un <h1> que resuma el tema de la página.', expected: '<h1>Tema principal de la página</h1>' },
  },
  h1_count: {
    en: { fix: 'Use a single <h1>; convert extra <h1> elements to <h2>-<h6>.', expected: '<h1>Main title</h1>\n<h2>Section</h2>' },
    es: { fix: 'Usa un único <h1>; convierte los <h1> sobrantes en <h2>-<h6>.', expected: '<h1>Título principal</h1>\n<h2>Sección</h2>' },
  },
  heading_hierarchy: {
    en: { fix: 'Restore a sequential outline (h1 → h2 → h3) without skipped levels.', expected: '<h2>Section</h2>\n<h3>Subsection</h3>' },
    es: { fix: 'Restaura un esquema secuencial (h1 → h2 → h3) sin saltos de nivel.', expected: '<h2>Sección</h2>\n<h3>Subsección</h3>' },
  },
  h1_title_match: {
    en: { fix: 'Align the <title> and <h1> so both mention the main topic.', expected: '<title>Best Running Shoes</title>\n<h1>Best Running Shoes</h1>' },
    es: { fix: 'Alinea el <title> y el <h1> para que ambos mencionen el tema principal.', expected: '<title>Las mejores zapatillas</title>\n<h1>Las mejores zapatillas</h1>' },
  },
  word_count: {
    en: { fix: 'Expand thin content to at least 300 words.', expected: 'Add substantive paragraphs covering the topic in depth.' },
    es: { fix: 'Amplía el contenido escaso a al menos 300 palabras.', expected: 'Añade párrafos sustanciales que cubran el tema en profundidad.' },
  },
  keyword_density: {
    en: { fix: 'Use the main keyword naturally 0.5-5% of the time; avoid stuffing.', expected: 'Mention the keyword a few times in headings, intro and body.' },
    es: { fix: 'Usa la palabra clave principal de forma natural entre 0.5-5%; evita el relleno.', expected: 'Menciona la palabra clave unas pocas veces en encabezados, introducción y cuerpo.' },
  },
  internal_links: {
    en: { fix: 'Add a link to another page of the same site.', expected: '<a href="/related-page">Related content</a>' },
    es: { fix: 'Añade un enlace a otra página del mismo sitio.', expected: '<a href="/pagina-relacionada">Contenido relacionado</a>' },
  },
  outbound_links: {
    en: { fix: 'Add a link to a relevant external source.', expected: '<a href="https://example.com" rel="nofollow noopener">Source</a>' },
    es: { fix: 'Añade un enlace a una fuente externa relevante.', expected: '<a href="https://example.com" rel="nofollow noopener">Fuente</a>' },
  },
  https_used: {
    en: { fix: 'Serve the site over HTTPS with a valid certificate.', expected: 'Redirect http:// to https:// and update the canonical URL.' },
    es: { fix: 'Sirve el sitio a través de HTTPS con un certificado válido.', expected: 'Redirige http:// a https:// y actualiza la URL canónica.' },
  },
  status_ok: {
    en: { fix: 'Make the page return 200 instead of an error status.', expected: 'Fix the resource, the route or the redirect that is failing.' },
    es: { fix: 'Haz que la página devuelva 200 en lugar de un estado de error.', expected: 'Corrige el recurso, la ruta o la redirección que está fallando.' },
  },
  viewport: {
    en: { fix: 'Add the viewport meta tag in the <head>.', expected: '<meta name="viewport" content="width=device-width, initial-scale=1">' },
    es: { fix: 'Añade la etiqueta meta viewport en el <head>.', expected: '<meta name="viewport" content="width=device-width, initial-scale=1">' },
  },
  favicon: {
    en: { fix: 'Add a favicon link in the <head>.', expected: '<link rel="icon" href="/favicon.ico" type="image/x-icon">' },
    es: { fix: 'Añade un enlace de favicon en el <head>.', expected: '<link rel="icon" href="/favicon.ico" type="image/x-icon">' },
  },
  charset: {
    en: { fix: 'Declare the UTF-8 charset in the <head>.', expected: '<meta charset="utf-8">' },
    es: { fix: 'Declara el charset UTF-8 en el <head>.', expected: '<meta charset="utf-8">' },
  },
  doctype: {
    en: { fix: 'Start the document with the HTML5 doctype.', expected: '<!DOCTYPE html>\n<html lang="en">' },
    es: { fix: 'Empieza el documento con el doctype HTML5.', expected: '<!DOCTYPE html>\n<html lang="es">' },
  },
  canonical_present: {
    en: { fix: 'Add a canonical tag pointing to the preferred URL.', expected: '<link rel="canonical" href="https://example.com/preferred-url">' },
    es: { fix: 'Añade una etiqueta canonical que apunte a la URL preferida.', expected: '<link rel="canonical" href="https://example.com/url-preferida">' },
  },
  indexable: {
    en: { fix: 'Remove the noindex directive so the page can be indexed.', expected: '<meta name="robots" content="index, follow">' },
    es: { fix: 'Elimina la directiva noindex para que la página se pueda indexar.', expected: '<meta name="robots" content="index, follow">' },
  },
  html_lang: {
    en: { fix: 'Set the lang attribute on the <html> element.', expected: '<html lang="en">' },
    es: { fix: 'Define el atributo lang en el elemento <html>.', expected: '<html lang="es">' },
  },
  url_length: {
    en: { fix: 'Shorten deeply nested or over-long URLs.', expected: '/products/shoes → shorter, keyword-based slugs' },
    es: { fix: 'Acorta las URL largas o muy anidadas.', expected: '/productos/zapatos → slugs cortos basados en palabras clave' },
  },
  url_underscores: {
    en: { fix: 'Use hyphens instead of underscores in URL paths.', expected: '/best-shoes → /best-shoes (hyphens, not underscores)' },
    es: { fix: 'Usa guiones en lugar de guiones bajos en las rutas.', expected: '/mejores-zapatos → /mejores-zapatos (guiones, no guiones bajos)' },
  },
  og_title: {
    en: { fix: 'Add og:title matching the page title.', expected: '<meta property="og:title" content="Page title">' },
    es: { fix: 'Añade og:title acorde con el título de la página.', expected: '<meta property="og:title" content="Título de la página">' },
  },
  og_description: {
    en: { fix: 'Add a concise og:description.', expected: '<meta property="og:description" content="Short summary">' },
    es: { fix: 'Añade un og:description conciso.', expected: '<meta property="og:description" content="Resumen breve">' },
  },
  og_image: {
    en: { fix: 'Add an og:image (1200×630 recommended).', expected: '<meta property="og:image" content="https://example.com/og-image.jpg">' },
    es: { fix: 'Añade un og:image (1200×630 recomendado).', expected: '<meta property="og:image" content="https://example.com/og-image.jpg">' },
  },
  og_image_alt: {
    en: { fix: 'Describe the Open Graph image.', expected: '<meta property="og:image:alt" content="Description of the image">' },
    es: { fix: 'Describe la imagen de Open Graph.', expected: '<meta property="og:image:alt" content="Descripción de la imagen">' },
  },
  og_url: {
    en: { fix: 'Add og:url pointing to the canonical URL.', expected: '<meta property="og:url" content="https://example.com/preferred-url">' },
    es: { fix: 'Añade og:url que apunte a la URL canónica.', expected: '<meta property="og:url" content="https://example.com/url-preferida">' },
  },
  og_type: {
    en: { fix: 'Add the og:type meta tag.', expected: '<meta property="og:type" content="website">' },
    es: { fix: 'Añade la etiqueta meta og:type.', expected: '<meta property="og:type" content="website">' },
  },
  og_site_name: {
    en: { fix: 'Add og:site_name with your brand name.', expected: '<meta property="og:site_name" content="Your Brand">' },
    es: { fix: 'Añade og:site_name con el nombre de tu marca.', expected: '<meta property="og:site_name" content="Tu Marca">' },
  },
  twitter_card: {
    en: { fix: 'Add the twitter:card meta tag.', expected: '<meta name="twitter:card" content="summary_large_image">' },
    es: { fix: 'Añade la etiqueta meta twitter:card.', expected: '<meta name="twitter:card" content="summary_large_image">' },
  },
  twitter_title: {
    en: { fix: 'Add a twitter:title meta tag.', expected: '<meta name="twitter:title" content="Page title">' },
    es: { fix: 'Añade una etiqueta meta twitter:title.', expected: '<meta name="twitter:title" content="Título de la página">' },
  },
  twitter_description: {
    en: { fix: 'Add a twitter:description meta tag.', expected: '<meta name="twitter:description" content="Short summary">' },
    es: { fix: 'Añade una etiqueta meta twitter:description.', expected: '<meta name="twitter:description" content="Resumen breve">' },
  },
  twitter_image: {
    en: { fix: 'Add a twitter:image meta tag.', expected: '<meta name="twitter:image" content="https://example.com/og-image.jpg">' },
    es: { fix: 'Añade una etiqueta meta twitter:image.', expected: '<meta name="twitter:image" content="https://example.com/og-image.jpg">' },
  },
  img_alt: {
    en: { fix: 'Add a descriptive alt to every image (alt="" for decorative ones).', expected: '<img src="photo.jpg" alt="Person walking on the beach at sunset">' },
    es: { fix: 'Añade un alt descriptivo a cada imagen (alt="" para las decorativas).', expected: '<img src="foto.jpg" alt="Persona caminando por la playa al atardecer">' },
  },
  img_dimensions: {
    en: { fix: 'Add width and height to images to avoid layout shift.', expected: '<img src="photo.jpg" alt="…" width="800" height="600">' },
    es: { fix: 'Añade width y height a las imágenes para evitar saltos de layout.', expected: '<img src="foto.jpg" alt="…" width="800" height="600">' },
  },
  form_labels: {
    en: { fix: 'Associate every control with a <label> via for/id.', expected: '<label for="email">Email</label>\n<input type="email" id="email">' },
    es: { fix: 'Asocia cada control con un <label> mediante for/id.', expected: '<label for="email">Correo</label>\n<input type="email" id="email">' },
  },
  input_ids: {
    en: { fix: 'Give each form control a unique id.', expected: '<input type="text" id="name">' },
    es: { fix: 'Asigna un id único a cada control de formulario.', expected: '<input type="text" id="nombre">' },
  },
  aria_controls: {
    en: { fix: 'Add aria-label/aria-labelledby to controls without a label.', expected: '<select aria-label="Country">\n  <option>…</option>\n</select>' },
    es: { fix: 'Añade aria-label/aria-labelledby a los controles sin etiqueta.', expected: '<select aria-label="País">\n  <option>…</option>\n</select>' },
  },
  empty_link_text: {
    en: { fix: 'Give every link visible text or an aria-label.', expected: '<a href="/products">View products</a>\n<a href="/search" aria-label="Search"><svg>…</svg></a>' },
    es: { fix: 'Da a cada enlace texto visible o un aria-label.', expected: '<a href="/productos">Ver productos</a>\n<a href="/buscar" aria-label="Buscar"><svg>…</svg></a>' },
  },
  main_landmark: {
    en: { fix: 'Wrap the primary content in a <main> element.', expected: '<main>Primary content</main>' },
    es: { fix: 'Envuelve el contenido principal en un elemento <main>.', expected: '<main>Contenido principal</main>' },
  },
  header_landmark: {
    en: { fix: 'Add a <header> element for the page banner.', expected: '<header>Site banner</header>' },
    es: { fix: 'Añade un elemento <header> para el banner de la página.', expected: '<header>Banner del sitio</header>' },
  },
  footer_landmark: {
    en: { fix: 'Add a <footer> element.', expected: '<footer>Copyright and links</footer>' },
    es: { fix: 'Añade un elemento <footer>.', expected: '<footer>Copyright y enlaces</footer>' },
  },
  nav_landmark: {
    en: { fix: 'Wrap navigation links in a <nav> element.', expected: '<nav><ul><li><a href="/">Home</a></li></ul></nav>' },
    es: { fix: 'Envuelve los enlaces de navegación en un elemento <nav>.', expected: '<nav><ul><li><a href="/">Inicio</a></li></ul></nav>' },
  },
  nesting_valid: {
    en: { fix: 'Move nested elements into valid parents per the HTML spec.', expected: '<div>\n  <span>Inline</span>\n  <p>Block</p>\n</div>' },
    es: { fix: 'Mueve los elementos anidados a padres válidos según la especificación HTML.', expected: '<div>\n  <span>En línea</span>\n  <p>Bloque</p>\n</div>' },
  },
  page_weight: {
    en: { fix: 'Reduce page weight by minifying HTML/CSS and removing inline assets.', expected: 'Target < 1.5 MB total transferred.' },
    es: { fix: 'Reduce el peso de la página minificando HTML/CSS y eliminando recursos en línea.', expected: 'Objetivo: < 1.5 MB transferidos.' },
  },
  load_time: {
    en: { fix: 'Improve server response time (target < 2.5 s).', expected: 'Use caching, CDN or faster hosting.' },
    es: { fix: 'Mejora el tiempo de respuesta del servidor (objetivo < 2.5 s).', expected: 'Usa caché, CDN u hosting más rápido.' },
  },
  image_optimization: {
    en: { fix: 'Add width/height to images so browsers reserve space.', expected: '<img src="photo.jpg" alt="…" width="800" height="600">' },
    es: { fix: 'Añade width/height a las imágenes para que el navegador reserve el espacio.', expected: '<img src="foto.jpg" alt="…" width="800" height="600">' },
  },
  resource_hints: {
    en: { fix: 'Preconnect to critical origins and preload key resources.', expected: '<link rel="preconnect" href="https://api.example.com">\n<link rel="preload" as="style" href="/app.css">' },
    es: { fix: 'Conecta con preconnect a orígenes críticos y precarga recursos clave.', expected: '<link rel="preconnect" href="https://api.example.com">\n<link rel="preload" as="style" href="/app.css">' },
  },
  pagespeed: {
    en: { fix: 'Follow the Lighthouse performance audits to raise the score above 50.', expected: 'Optimize images, lazy-load below-the-fold content, avoid long tasks.' },
    es: { fix: 'Sigue las auditorías de rendimiento de Lighthouse para superar 50 puntos.', expected: 'Optimiza imágenes, carga diferida bajo el pliegue y evita tareas largas.' },
  },
  readability_score: {
    en: { fix: 'Simplify language and shorten sentences to reach ≥ 50.', expected: 'Short sentences, common words, clear structure.' },
    es: { fix: 'Simplifica el lenguaje y acorta las frases para alcanzar ≥ 50.', expected: 'Frases cortas, palabras comunes y estructura clara.' },
  },
  flesch_kincaid_grade: {
    en: { fix: 'Target a reading grade level ≤ 12.', expected: 'Use plain language most of your audience understands.' },
    es: { fix: 'Apunta a un nivel de lectura ≤ 12.', expected: 'Usa un lenguaje sencillo que entienda la mayor parte de tu audiencia.' },
  },
  sentence_length: {
    en: { fix: 'Break long sentences into shorter, single-idea sentences.', expected: 'Average sentence length ≤ 25 words.' },
    es: { fix: 'Divide las frases largas en frases cortas con una sola idea.', expected: 'Longitud media de frase ≤ 25 palabras.' },
  },
  paragraph_structure: {
    en: { fix: 'Structure the content into short paragraphs and subheadings.', expected: 'At least 3 paragraphs, each with one main idea.' },
    es: { fix: 'Estructura el contenido en párrafos cortos y subtítulos.', expected: 'Al menos 3 párrafos, cada uno con una idea principal.' },
  },
  semantic_html: {
    en: { fix: 'Use semantic landmarks (main/header/footer/nav).', expected: '<header>…</header>\n<nav>…</nav>\n<main>…</main>\n<footer>…</footer>' },
    es: { fix: 'Usa landmarks semánticos (main/header/footer/nav).', expected: '<header>…</header>\n<nav>…</nav>\n<main>…</main>\n<footer>…</footer>' },
  },
  content_present: {
    en: { fix: 'Add substantive text content to the page.', expected: 'Write original paragraphs, not just keywords or images.' },
    es: { fix: 'Añade contenido de texto sustancial a la página.', expected: 'Escribe párrafos originales, no solo palabras clave o imágenes.' },
  },
  structured_data: {
    en: { fix: 'Add JSON-LD structured data describing the content.', expected: '<script type="application/ld+json">{"@context":"https://schema.org","@type":"WebPage"}</script>' },
    es: { fix: 'Añade datos estructurados JSON-LD que describan el contenido.', expected: '<script type="application/ld+json">{"@context":"https://schema.org","@type":"WebPage"}</script>' },
  },
  faq_schema: {
    en: { fix: 'Mark up questions and answers with FAQPage schema.', expected: '<script type="application/ld+json">{"@type":"FAQPage","mainEntity":[{"@type":"Question","name":"…","acceptedAnswer":{"@type":"Answer","text":"…"}}]}</script>' },
    es: { fix: 'Marca las preguntas y respuestas con el esquema FAQPage.', expected: '<script type="application/ld+json">{"@type":"FAQPage","mainEntity":[{"@type":"Question","name":"…","acceptedAnswer":{"@type":"Answer","text":"…"}}]}</script>' },
  },
  howto_schema: {
    en: { fix: 'Mark up step-by-step instructions with HowTo schema.', expected: '<script type="application/ld+json">{"@type":"HowTo","step":[{"@type":"HowToStep","text":"…"}]}</script>' },
    es: { fix: 'Marca las instrucciones paso a paso con el esquema HowTo.', expected: '<script type="application/ld+json">{"@type":"HowTo","step":[{"@type":"HowToStep","text":"…"}]}</script>' },
  },
  breadcrumb_schema: {
    en: { fix: 'Add BreadcrumbList schema for navigation paths.', expected: '<script type="application/ld+json">{"@type":"BreadcrumbList","itemListElement":[{"@type":"ListItem","position":1,"item":{"@id":"/","name":"Home"}}]}</script>' },
    es: { fix: 'Añade el esquema BreadcrumbList para las rutas de navegación.', expected: '<script type="application/ld+json">{"@type":"BreadcrumbList","itemListElement":[{"@type":"ListItem","position":1,"item":{"@id":"/","name":"Inicio"}}]}</script>' },
  },
  article_schema: {
    en: { fix: 'Mark up articles with Article/NewsArticle/BlogPosting schema.', expected: '<script type="application/ld+json">{"@type":"Article","headline":"…","author":{"@type":"Person","name":"…"}}</script>' },
    es: { fix: 'Marca los artículos con el esquema Article/NewsArticle/BlogPosting.', expected: '<script type="application/ld+json">{"@type":"Article","headline":"…","author":{"@type":"Person","name":"…"}}</script>' },
  },
  organization_schema: {
    en: { fix: 'Add Organization or WebSite schema to define entity and brand.', expected: '<script type="application/ld+json">{"@type":"Organization","name":"Your Brand","url":"https://example.com"}</script>' },
    es: { fix: 'Añade el esquema Organization o WebSite para definir la entidad y la marca.', expected: '<script type="application/ld+json">{"@type":"Organization","name":"Tu Marca","url":"https://example.com"}</script>' },
  },
  question_headings: {
    en: { fix: 'Use headings phrased as questions users actually ask.', expected: '<h2>How does this feature work?</h2>' },
    es: { fix: 'Usa encabezados formulados como preguntas que los usuarios hacen.', expected: '<h2>¿Cómo funciona esta función?</h2>' },
  },
  direct_answer: {
    en: { fix: 'Lead with a direct, concise answer to the main question.', expected: 'Answer in the first sentence, then expand with details.' },
    es: { fix: 'Comienza con una respuesta directa y concisa a la pregunta principal.', expected: 'Responde en la primera frase y luego amplía con detalles.' },
  },
};

export interface LocalizedCheck {
  message: string;
  guidance: string;
  fix?: string;
  expected?: string;
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
    const fixEntry = CHECK_FIXES[id];
    const fixTr = fixEntry ? (getLocale().startsWith('es') ? fixEntry.es : fixEntry.en) : null;
    return {
      message: tr.message,
      guidance: tr.guidance,
      ...(fixTr ? { fix: fixTr.fix, expected: fixTr.expected } : {}),
    };
  }
  if (!entry) return { message, guidance };
  const tr = getLocale().startsWith('es') ? entry.es : entry.en;
  const params = extractParams(message, evidence);
  const fixEntry = CHECK_FIXES[id];
  const fixTr = fixEntry ? (getLocale().startsWith('es') ? fixEntry.es : fixEntry.en) : null;
  return {
    message: applyParams(tr.message, params),
    guidance: applyParams(tr.guidance, params),
    ...(fixTr ? { fix: fixTr.fix, expected: fixTr.expected } : {}),
  };
}
