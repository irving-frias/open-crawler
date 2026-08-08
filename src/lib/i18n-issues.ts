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
 */
const ISSUE_FIXES: Record<
  string,
  { en: { fix: string; expected: string }; es: { fix: string; expected: string } }
> = {
  missing_html_lang: {
    en: {
      fix: 'Add a lang attribute to the <html> element so search engines and screen readers know the page language.',
      expected: '<html lang="en">',
    },
    es: {
      fix: 'Añade un atributo lang al elemento <html> para que los buscadores y lectores de pantalla conozcan el idioma de la página.',
      expected: '<html lang="es">',
    },
  },
  missing_title: {
    en: {
      fix: 'Add a unique, descriptive <title> in the <head> (30-65 characters).',
      expected: '<head>\n  <title>Unique, descriptive page title</title>\n</head>',
    },
    es: {
      fix: 'Añade un <title> único y descriptivo en el <head> (30-65 caracteres).',
      expected: '<head>\n  <title>Título único y descriptivo de la página</title>\n</head>',
    },
  },
  missing_meta_description: {
    en: {
      fix: 'Add a meta description between 50 and 160 characters summarizing the page.',
      expected:
        '<meta name="description" content="A 50-160 character summary of the page content.">',
    },
    es: {
      fix: 'Añade una meta description de entre 50 y 160 caracteres que resuma la página.',
      expected:
        '<meta name="description" content="Un resumen de 50-160 caracteres del contenido de la página.">',
    },
  },
  missing_canonical: {
    en: {
      fix: 'Point a canonical link at the preferred URL of this page.',
      expected: '<link rel="canonical" href="https://example.com/preferred-url">',
    },
    es: {
      fix: 'Apunta un enlace canonical a la URL preferida de esta página.',
      expected: '<link rel="canonical" href="https://example.com/url-preferida">',
    },
  },
  missing_main: {
    en: {
      fix: 'Wrap the primary page content in a <main> element.',
      expected:
        '<body>\n  <header>…</header>\n  <main>Primary content</main>\n  <footer>…</footer>\n</body>',
    },
    es: {
      fix: 'Envuelve el contenido principal de la página en un elemento <main>.',
      expected:
        '<body>\n  <header>…</header>\n  <main>Contenido principal</main>\n  <footer>…</footer>\n</body>',
    },
  },
  missing_header: {
    en: {
      fix: 'Add a <header> element for the page banner at the top.',
      expected: '<body>\n  <header>Site banner and navigation</header>\n  <main>…</main>\n</body>',
    },
    es: {
      fix: 'Añade un elemento <header> para el banner de la página al inicio.',
      expected:
        '<body>\n  <header>Banner del sitio y navegación</header>\n  <main>…</main>\n</body>',
    },
  },
  missing_footer: {
    en: {
      fix: 'Add a <footer> element with site-wide information.',
      expected: '<body>\n  <main>…</main>\n  <footer>Copyright and contact</footer>\n</body>',
    },
    es: {
      fix: 'Añade un elemento <footer> con la información general del sitio.',
      expected: '<body>\n  <main>…</main>\n  <footer>Copyright y contacto</footer>\n</body>',
    },
  },
  missing_nav: {
    en: {
      fix: 'Wrap the primary navigation links in a <nav> element.',
      expected: '<nav>\n  <ul>\n    <li><a href="/">Home</a></li>\n  </ul>\n</nav>',
    },
    es: {
      fix: 'Envuelve los enlaces de navegación principales en un elemento <nav>.',
      expected: '<nav>\n  <ul>\n    <li><a href="/">Inicio</a></li>\n  </ul>\n</nav>',
    },
  },
  missing_h1: {
    en: {
      fix: 'Add exactly one <h1> summarizing the main topic of the page.',
      expected: '<main>\n  <h1>Main topic of the page</h1>\n</main>',
    },
    es: {
      fix: 'Añade exactamente un <h1> que resuma el tema principal de la página.',
      expected: '<main>\n  <h1>Tema principal de la página</h1>\n</main>',
    },
  },
  multiple_h1: {
    en: {
      fix: 'Keep a single <h1>; convert the extra <h1> elements to <h2> (or lower) in the heading hierarchy.',
      expected: '<h1>Main title</h1>\n<h2>First section</h2>\n<h2>Second section</h2>',
    },
    es: {
      fix: 'Mantén un único <h1>; convierte los <h1> sobrantes en <h2> (o inferiores) en la jerarquía de encabezados.',
      expected: '<h1>Título principal</h1>\n<h2>Primera sección</h2>\n<h2>Segunda sección</h2>',
    },
  },
  heading_skip: {
    en: {
      fix: 'Restore a sequential heading outline (h1 → h2 → h3) without skipping levels.',
      expected: '<h1>Section</h1>\n<h2>Subsection</h2>\n<h3>Detail</h3>',
    },
    es: {
      fix: 'Restaura un esquema de encabezados secuencial (h1 → h2 → h3) sin saltar niveles.',
      expected: '<h1>Sección</h1>\n<h2>Subsección</h2>\n<h3>Detalle</h3>',
    },
  },
  img_no_alt: {
    en: {
      fix: 'Add a descriptive alt attribute to the image, or set alt="" for decorative images.',
      expected: '<img src="photo.jpg" alt="A person walking on the beach at sunset">',
    },
    es: {
      fix: 'Añade un atributo alt descriptivo a la imagen, o usa alt="" para imágenes decorativas.',
      expected: '<img src="foto.jpg" alt="Una persona caminando por la playa al atardecer">',
    },
  },
  img_no_dimensions: {
    en: {
      fix: 'Set explicit width and height attributes (matching the intrinsic size) so the browser reserves layout space and avoids layout shift.',
      expected: '<img src="photo.jpg" alt="Photo" width="1200" height="800">',
    },
    es: {
      fix: 'Define los atributos width y height explícitos (coincidiendo con el tamaño intrínseco) para que el navegador reserve el espacio y evite el desplazamiento de layout.',
      expected: '<img src="foto.jpg" alt="Foto" width="1200" height="800">',
    },
  },
  input_no_id: {
    en: {
      fix: 'Give the form control a unique id so it can be associated with a <label>.',
      expected: '<input type="text" id="email">\n<label for="email">Email</label>',
    },
    es: {
      fix: 'Asigna al control un id único para poder asociarlo con un <label>.',
      expected: '<input type="text" id="email">\n<label for="email">Correo</label>',
    },
  },
  input_no_label: {
    en: {
      fix: 'Associate the control with a <label> using the for/id attributes, or wrap it inside a <label>.',
      expected: '<label for="name">Name</label>\n<input type="text" id="name">',
    },
    es: {
      fix: 'Asocia el control con un <label> mediante los atributos for/id, o envuélvelo dentro de un <label>.',
      expected: '<label for="nombre">Nombre</label>\n<input type="text" id="nombre">',
    },
  },
  empty_link_text: {
    en: {
      fix: 'Add visible link text, or an aria-label describing the destination when a link is icon-only.',
      expected: '<a href="/products">View products</a>',
    },
    es: {
      fix: 'Añade texto visible al enlace, o un aria-label que describa el destino cuando el enlace es solo un icono.',
      expected: '<a href="/productos">Ver productos</a>',
    },
  },
  missing_aria: {
    en: {
      fix: 'Add an accessible name to the control with aria-label, or associate a <label> via for/id.',
      expected: '<select aria-label="Country">\n  <option value="es">Spain</option>\n</select>',
    },
    es: {
      fix: 'Añade un nombre accesible al control con aria-label, o asocia un <label> mediante for/id.',
      expected: '<select aria-label="País">\n  <option value="es">España</option>\n</select>',
    },
  },
  invalid_nesting: {
    en: {
      fix: 'Move the nested element out of the invalid parent (e.g. do not put block elements inside <span> or <p>).',
      expected: '<span>Inline text</span>\n<div>Block content</div>',
    },
    es: {
      fix: 'Saca el elemento anidado del padre inválido (p. ej. no pongas elementos de bloque dentro de <span> o <p>).',
      expected: '<span>Texto en línea</span>\n<div>Contenido de bloque</div>',
    },
  },
  context_nesting: {
    en: {
      fix: 'Move the element to a context where the nesting is allowed by the HTML spec.',
      expected: '<div>\n  <a href="/page"><span>Link</span></a>\n</div>',
    },
    es: {
      fix: 'Mueve el elemento a un contexto donde el anidamiento esté permitido por la especificación HTML.',
      expected: '<div>\n  <a href="/pagina"><span>Enlace</span></a>\n</div>',
    },
  },
};

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
