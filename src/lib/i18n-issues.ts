import { getLocale } from '$lib/paraglide/runtime.js';
import { m } from '$lib/paraglide/messages.js';

type IssueParams = Record<string, string | number>;

const issueNames: Record<string, () => string> = {
  missing_html_lang: () => m['issue.missing_html_lang.name'](),
  missing_title: () => m['issue.missing_title.name'](),
  missing_meta_description: () => m['issue.missing_meta_description.name'](),
  missing_canonical: () => m['issue.missing_canonical.name'](),
  missing_main: () => m['issue.missing_main.name'](),
  missing_header: () => m['issue.missing_header.name'](),
  missing_footer: () => m['issue.missing_footer.name'](),
  missing_nav: () => m['issue.missing_nav.name'](),
  missing_h1: () => m['issue.missing_h1.name'](),
  multiple_h1: () => m['issue.multiple_h1.name'](),
  heading_skip: () => m['issue.heading_skip.name'](),
  img_no_alt: () => m['issue.img_no_alt.name'](),
  img_no_dimensions: () => m['issue.img_no_dimensions.name'](),
  input_no_id: () => m['issue.input_no_id.name'](),
  input_no_label: () => m['issue.input_no_label.name'](),
  empty_link_text: () => m['issue.empty_link_text.name'](),
  missing_aria: () => m['issue.missing_aria.name'](),
  invalid_nesting: () => m['issue.invalid_nesting.name'](),
  context_nesting: () => m['issue.context_nesting.name'](),
};

const issueMessages: Record<string, (params?: IssueParams) => string> = {
  missing_html_lang: () => m['issue.missing_html_lang.message'](),
  missing_title: () => m['issue.missing_title.message'](),
  missing_meta_description: () => m['issue.missing_meta_description.message'](),
  missing_canonical: () => m['issue.missing_canonical.message'](),
  missing_main: () => m['issue.missing_main.message'](),
  missing_header: () => m['issue.missing_header.message'](),
  missing_footer: () => m['issue.missing_footer.message'](),
  missing_nav: () => m['issue.missing_nav.message'](),
  missing_h1: () => m['issue.missing_h1.message'](),
  multiple_h1: (p) => m['issue.multiple_h1.message']({ count: String(p?.count ?? 0) }),
  heading_skip: (p) =>
    m['issue.heading_skip.message']({ prev: String(p?.prev ?? 0), level: String(p?.level ?? 0) }),
  img_no_alt: () => m['issue.img_no_alt.message'](),
  img_no_dimensions: () => m['issue.img_no_dimensions.message'](),
  input_no_id: () => m['issue.input_no_id.message'](),
  input_no_label: () => m['issue.input_no_label.message'](),
  empty_link_text: () => m['issue.empty_link_text.message'](),
  missing_aria: () => m['issue.missing_aria.message'](),
  invalid_nesting: (p) =>
    m['issue.invalid_nesting.message']({
      child: String(p?.child ?? ''),
      parent: String(p?.parent ?? ''),
    }),
  context_nesting: (p) =>
    m['issue.context_nesting.message']({
      child: String(p?.child ?? ''),
      parent: String(p?.parent ?? ''),
    }),
};

/**
 * Static fix + expected-markup catalog for every semantic issue type.
 * Kept as a plain en/es map (like seo-checks.ts) so the panel can always offer
 * a localized, offline solution next to the offending element.
 *
 * - `why`: the impact of the problem, in one clear sentence.
 * - `fix`: what to do, in one imperative sentence.
 * - `expected`: a concrete markup example that resolves the issue.
 */
interface IssueFixEntry {
  en: { why: string; fix: string; expected: string };
  es: { why: string; fix: string; expected: string };
}

const ISSUE_FIXES: Record<string, IssueFixEntry> = {
  missing_html_lang: {
    en: {
      why: 'Search engines and screen readers rely on the lang attribute to pronounce and interpret the page correctly; missing it blocks language-aware ranking and accessibility features.',
      fix: 'Add a lang attribute to the <html> element so search engines and screen readers know the page language.',
      expected: '<html lang="en">',
    },
    es: {
      why: 'Los buscadores y lectores de pantalla dependen del atributo lang para pronunciar e interpretar correctamente la página; sin él se pierden las funciones de idioma y accesibilidad.',
      fix: 'Añade un atributo lang al elemento <html> para que los buscadores y lectores de pantalla conozcan el idioma de la página.',
      expected: '<html lang="es">',
    },
  },
  missing_title: {
    en: {
      why: 'The title is the first thing users and search engines see; it is the main signal for ranking and the text shown in the browser tab and search results.',
      fix: 'Add a unique, descriptive <title> in the <head> (30-65 characters).',
      expected: '<head>\n  <title>Unique, descriptive page title</title>\n</head>',
    },
    es: {
      why: 'El título es lo primero que ven los usuarios y los buscadores; es la principal señal de posicionamiento y el texto que se muestra en la pestaña del navegador y en los resultados.',
      fix: 'Añade un <title> único y descriptivo en el <head> (30-65 caracteres).',
      expected: '<head>\n  <title>Título único y descriptivo de la página</title>\n</head>',
    },
  },
  missing_meta_description: {
    en: {
      why: 'The meta description is the snippet text shown under the title in results; a good one raises the click-through rate even if it is not a ranking factor.',
      fix: 'Add a meta description between 50 and 160 characters summarizing the page.',
      expected:
        '<meta name="description" content="A 50-160 character summary of the page content.">',
    },
    es: {
      why: 'La meta description es el texto del fragmento que se muestra bajo el título en los resultados; una buena descripción aumenta el porcentaje de clics aunque no sea un factor de posicionamiento.',
      fix: 'Añade una meta description de entre 50 y 160 caracteres que resuma la página.',
      expected:
        '<meta name="description" content="Un resumen de 50-160 caracteres del contenido de la página.">',
    },
  },
  missing_canonical: {
    en: {
      why: 'Without a canonical, duplicate or near-duplicate URLs can split ranking signals and let search engines pick the wrong version of your page.',
      fix: 'Point a canonical link at the preferred URL of this page.',
      expected: '<link rel="canonical" href="https://example.com/preferred-url">',
    },
    es: {
      why: 'Sin canonical, las URL duplicadas o casi duplicadas pueden dividir las señales de posicionamiento y hacer que los buscadores elijan la versión equivocada de tu página.',
      fix: 'Apunta un enlace canonical a la URL preferida de esta página.',
      expected: '<link rel="canonical" href="https://example.com/url-preferida">',
    },
  },
  missing_main: {
    en: {
      why: 'A single <main> landmark tells browsers, assistive tech and AI crawlers where the unique page content begins, making the page easier to navigate and understand.',
      fix: 'Wrap the primary page content in a <main> element.',
      expected:
        '<body>\n  <header>…</header>\n  <main>Primary content</main>\n  <footer>…</footer>\n</body>',
    },
    es: {
      why: 'Un landmark <main> único indica a los navegadores, a las tecnologías de asistencia y a los rastreadores de IA dónde empieza el contenido único de la página, facilitando la navegación y la comprensión.',
      fix: 'Envuelve el contenido principal de la página en un elemento <main>.',
      expected:
        '<body>\n  <header>…</header>\n  <main>Contenido principal</main>\n  <footer>…</footer>\n</body>',
    },
  },
  missing_header: {
    en: {
      why: 'A <header> landmark marks the page banner so assistive technology users can skip straight to it or past it, and search engines understand the site structure.',
      fix: 'Add a <header> element for the page banner at the top.',
      expected: '<body>\n  <header>Site banner and navigation</header>\n  <main>…</main>\n</body>',
    },
    es: {
      why: 'El landmark <header> marca el banner de la página para que los usuarios de tecnologías de asistencia puedan saltar a él o pasarlo, y los buscadores entiendan la estructura del sitio.',
      fix: 'Añade un elemento <header> para el banner de la página al inicio.',
      expected:
        '<body>\n  <header>Banner del sitio y navegación</header>\n  <main>…</main>\n</body>',
    },
  },
  missing_footer: {
    en: {
      why: 'A <footer> landmark groups site-wide information so users can find it predictably and crawlers can ignore it when extracting the main content.',
      fix: 'Add a <footer> element with site-wide information.',
      expected: '<body>\n  <main>…</main>\n  <footer>Copyright and contact</footer>\n</body>',
    },
    es: {
      why: 'El landmark <footer> agrupa la información general del sitio para que los usuarios la encuentren siempre en el mismo sitio y los rastreadores la ignoren al extraer el contenido principal.',
      fix: 'Añade un elemento <footer> con la información general del sitio.',
      expected: '<body>\n  <main>…</main>\n  <footer>Copyright y contacto</footer>\n</body>',
    },
  },
  missing_nav: {
    en: {
      why: 'A <nav> landmark exposes the navigation to screen readers and lets users jump directly to the menus, avoiding the need to tab through every link.',
      fix: 'Wrap the primary navigation links in a <nav> element.',
      expected: '<nav>\n  <ul>\n    <li><a href="/">Home</a></li>\n  </ul>\n</nav>',
    },
    es: {
      why: 'El landmark <nav> expone la navegación a los lectores de pantalla y permite saltar directamente a los menús sin tener que pasar por cada enlace.',
      fix: 'Envuelve los enlaces de navegación principales en un elemento <nav>.',
      expected: '<nav>\n  <ul>\n    <li><a href="/">Inicio</a></li>\n  </ul>\n</nav>',
    },
  },
  missing_h1: {
    en: {
      why: 'The <h1> is the strongest on-page signal of what a page is about; without it, search engines and AI tools have to guess the topic from weaker signals.',
      fix: 'Add exactly one <h1> summarizing the main topic of the page.',
      expected: '<main>\n  <h1>Main topic of the page</h1>\n</main>',
    },
    es: {
      why: 'El <h1> es la señal en página más fuerte sobre el tema de la página; sin él, los buscadores y las herramientas de IA tienen que adivinar el tema a partir de señales más débiles.',
      fix: 'Añade exactamente un <h1> que resuma el tema principal de la página.',
      expected: '<main>\n  <h1>Tema principal de la página</h1>\n</main>',
    },
  },
  multiple_h1: {
    en: {
      why: 'Multiple <h1> elements dilute the topic signal and confuse the heading outline used by screen readers, AI extractors and search snippets.',
      fix: 'Keep a single <h1>; convert the extra <h1> elements to <h2> (or lower) in the heading hierarchy.',
      expected: '<h1>Main title</h1>\n<h2>First section</h2>\n<h2>Second section</h2>',
    },
    es: {
      why: 'Varios <h1> diluyen la señal del tema y confunden el esquema de encabezados que usan los lectores de pantalla, los extractores de IA y los fragmentos de búsqueda.',
      fix: 'Mantén un único <h1>; convierte los <h1> sobrantes en <h2> (o inferiores) en la jerarquía de encabezados.',
      expected: '<h1>Título principal</h1>\n<h2>Primera sección</h2>\n<h2>Segunda sección</h2>',
    },
  },
  heading_skip: {
    en: {
      why: 'Skipped heading levels break the document outline, so assistive technology users cannot navigate the structure and the page hierarchy looks inconsistent.',
      fix: 'Restore a sequential heading outline (h1 → h2 → h3) without skipping levels.',
      expected: '<h1>Section</h1>\n<h2>Subsection</h2>\n<h3>Detail</h3>',
    },
    es: {
      why: 'Saltar niveles de encabezado rompe el esquema del documento, de modo que los usuarios de tecnologías de asistencia no pueden navegar la estructura y la jerarquía parece incoherente.',
      fix: 'Restaura un esquema de encabezados secuencial (h1 → h2 → h3) sin saltar niveles.',
      expected: '<h1>Sección</h1>\n<h2>Subsección</h2>\n<h3>Detalle</h3>',
    },
  },
  img_no_alt: {
    en: {
      why: 'Images without alt text are invisible to screen readers and AI image understanding, and search engines lose a ranking opportunity for image results.',
      fix: 'Add a descriptive alt attribute to the image, or set alt="" for decorative images.',
      expected: '<img src="photo.jpg" alt="A person walking on the beach at sunset">',
    },
    es: {
      why: 'Las imágenes sin texto alternativo son invisibles para los lectores de pantalla y para la comprensión de imágenes por IA, y los buscadores pierden una oportunidad en la búsqueda de imágenes.',
      fix: 'Añade un atributo alt descriptivo a la imagen, o usa alt="" para imágenes decorativas.',
      expected: '<img src="foto.jpg" alt="Una persona caminando por la playa al atardecer">',
    },
  },
  img_no_dimensions: {
    en: {
      why: 'Images without width and height make the browser guess the size and reflow the layout while loading, causing layout shift (CLS), a Core Web Vital that hurts user experience and ranking.',
      fix: 'Set explicit width and height attributes (matching the intrinsic size) so the browser reserves layout space and avoids layout shift.',
      expected: '<img src="photo.jpg" alt="Photo" width="1200" height="800">',
    },
    es: {
      why: 'Las imágenes sin width y height obligan al navegador a adivinar el tamaño y a reordenar el layout mientras cargan, causando desplazamiento de layout (CLS), una métrica Core Web Vitals que perjudica la experiencia y el posicionamiento.',
      fix: 'Define los atributos width y height explícitos (coincidiendo con el tamaño intrínseco) para que el navegador reserve el espacio y evite el desplazamiento de layout.',
      expected: '<img src="foto.jpg" alt="Foto" width="1200" height="800">',
    },
  },
  input_no_id: {
    en: {
      why: 'Form controls need an id so they can be linked to a <label>; without it, the field is announced to assistive technology with no associated name.',
      fix: 'Give the form control a unique id so it can be associated with a <label>.',
      expected: '<input type="text" id="email">\n<label for="email">Email</label>',
    },
    es: {
      why: 'Los controles de formulario necesitan un id para poder vincularse a un <label>; sin él, el campo se anuncia a las tecnologías de asistencia sin nombre asociado.',
      fix: 'Asigna al control un id único para poder asociarlo con un <label>.',
      expected: '<input type="text" id="email">\n<label for="email">Correo</label>',
    },
  },
  input_no_label: {
    en: {
      why: 'A field without a label leaves users guessing what data to enter, and screen readers cannot announce a name for it, making forms unusable for assistive technology users.',
      fix: 'Associate the control with a <label> using the for/id attributes, or wrap it inside a <label>.',
      expected: '<label for="name">Name</label>\n<input type="text" id="name">',
    },
    es: {
      why: 'Un campo sin etiqueta deja al usuario adivinando qué dato introducir, y los lectores de pantalla no pueden anunciar su nombre, lo que hace los formularios inutilizables para quienes usan tecnologías de asistencia.',
      fix: 'Asocia el control con un <label> mediante los atributos for/id, o envuélvelo dentro de un <label>.',
      expected: '<label for="nombre">Nombre</label>\n<input type="text" id="nombre">',
    },
  },
  empty_link_text: {
    en: {
      why: 'Links without accessible text are announced as blank by screen readers and give search engines no anchor signal, so both users and crawlers lose the destination context.',
      fix: 'Add visible link text, or an aria-label describing the destination when a link is icon-only.',
      expected: '<a href="/products">View products</a>',
    },
    es: {
      why: 'Los enlaces sin texto accesible se anuncian como vacíos a los lectores de pantalla y no dan señal de ancla a los buscadores, de modo que usuarios y rastreadores pierden el contexto del destino.',
      fix: 'Añade texto visible al enlace, o un aria-label que describa el destino cuando el enlace es solo un icono.',
      expected: '<a href="/productos">Ver productos</a>',
    },
  },
  missing_aria: {
    en: {
      why: 'Interactive controls without an accessible name cannot be identified by assistive technology, leaving users unable to know what the control does.',
      fix: 'Add an accessible name to the control with aria-label, or associate a <label> via for/id.',
      expected: '<select aria-label="Country">\n  <option value="es">Spain</option>\n</select>',
    },
    es: {
      why: 'Los controles interactivos sin nombre accesible no pueden identificarse con tecnologías de asistencia, de modo que los usuarios no saben qué hace el control.',
      fix: 'Añade un nombre accesible al control con aria-label, o asocia un <label> mediante for/id.',
      expected: '<select aria-label="País">\n  <option value="es">España</option>\n</select>',
    },
  },
  invalid_nesting: {
    en: {
      why: 'Nesting that contradicts the HTML spec makes browsers auto-correct the DOM in unpredictable ways, so the rendered structure (and the accessibility tree) differs from what you wrote.',
      fix: 'Move the nested element out of the invalid parent (e.g. do not put block elements inside <span> or <p>).',
      expected: '<span>Inline text</span>\n<div>Block content</div>',
    },
    es: {
      why: 'Un anidamiento que contradice la especificación HTML hace que el navegador corrija el DOM automáticamente de forma impredecible, de modo que la estructura renderizada (y el árbol de accesibilidad) difiere de lo que escribiste.',
      fix: 'Saca el elemento anidado del padre inválido (p. ej. no pongas elementos de bloque dentro de <span> o <p>).',
      expected: '<span>Texto en línea</span>\n<div>Contenido de bloque</div>',
    },
  },
  context_nesting: {
    en: {
      why: 'This nesting is only valid in certain contexts; browsers may re-parent the element, changing the outline and the accessible structure in ways that are hard to predict.',
      fix: 'Move the element to a context where the nesting is allowed by the HTML spec.',
      expected: '<div>\n  <a href="/page"><span>Link</span></a>\n</div>',
    },
    es: {
      why: 'Este anidamiento solo es válido en determinados contextos; el navegador puede reubicar el elemento, alterando el esquema y la estructura accesible de forma difícil de predecir.',
      fix: 'Mueve el elemento a un contexto donde el anidamiento esté permitido por la especificación HTML.',
      expected: '<div>\n  <a href="/pagina"><span>Enlace</span></a>\n</div>',
    },
  },
};

export function translateIssueWhy(issueType: string): string {
  const entry = ISSUE_FIXES[issueType];
  if (!entry) return '';
  const tr = getLocale().startsWith('es') ? entry.es : entry.en;
  return tr.why;
}

export function translateIssueFix(issueType: string): string {
  const entry = ISSUE_FIXES[issueType];
  if (!entry) return '';
  const tr = getLocale().startsWith('es') ? entry.es : entry.en;
  return tr.fix;
}

export function translateIssueExpected(issueType: string): string {
  const entry = ISSUE_FIXES[issueType];
  if (!entry) return '';
  const tr = getLocale().startsWith('es') ? entry.es : entry.en;
  return tr.expected;
}

export function translateIssueName(issueType: string): string {
  return issueNames[issueType]?.() ?? issueType.replace(/_/g, ' ');
}

export function translateIssueMessage(issueType: string, params?: IssueParams): string {
  return issueMessages[issueType]?.(params) ?? '';
}

export function parseIssueParams(message: string, issueType: string): IssueParams {
  const params: IssueParams = {};
  if (issueType === 'multiple_h1') {
    const m = message.match(/(\d+)\s*<h1>/i);
    if (m) params.count = m[1];
  } else if (issueType === 'heading_skip') {
    const m = message.match(/h(\d+)\s+to\s+h(\d+)/);
    if (m) {
      params.prev = m[1];
      params.level = m[2];
    }
  } else if (issueType === 'invalid_nesting' || issueType === 'context_nesting') {
    const childM = message.match(/<(\w+)>/);
    const parentM = message.match(/<(\w+)>.*?<(\w+)>/);
    if (childM) params.child = childM[1];
    if (parentM) params.parent = parentM[2];
  }
  return params;
}

const severityLabels: Record<string, () => string> = {
  error: () => m['severity.error'](),
  warning: () => m['severity.warning'](),
  info: () => m['severity.info'](),
};

export function translateSeverity(severity: string): string {
  return severityLabels[severity]?.() ?? severity;
}
