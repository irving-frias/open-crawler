// Auto-generated from caninclude JSON — DO NOT EDIT
// Generated: 2026-08-04T19:57:58.591Z
// Source: 104 elements
// Regenerate: node tools/generate_html_elements.js

export interface ElementRefLink {
  text: string;
  href?: string;
  hashText?: string;
}

export type ElementRefSegment = string | ElementRefLink;

export interface BrowserSupport {
  WebHTMLElement: string;
  WebAPI: string;
  caniuse: string;
}

export interface ElementReference {
  categories: string[];
  contexts: string[];
  contentModel: string[];
  categories_es: string[];
  contexts_es: string[];
  contentModel_es: string[];
  support: Record<string, BrowserSupport>;
  params: ElementRefSegment[];
  params_es: string[];
  rawCategories: ElementRefSegment[][];
  rawContexts: ElementRefSegment[][];
  rawContentModel: ElementRefSegment[][];
}

export const HTML_ELEMENT_REFERENCES: Record<string, ElementReference> = {
  a: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'If the element has an href attribute: Interactive content.',
      'Palpable content.',
    ],
    contexts: ['Where phrasing content is expected.'],
    contentModel: [
      'Transparent, but there must be no interactive content descendant,\n   a element descendant, or descendant with the tabindex attribute specified.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'href',
        href: 'https://html.spec.whatwg.org/#attr-hyperlink-href',
        hashText: '#attr-hyperlink-href',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        'If the element has an ',
        {
          text: 'href',
          href: 'https://html.spec.whatwg.org/#attr-hyperlink-href',
          hashText: '#attr-hyperlink-href',
        },
        ' attribute: ',
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        ', but there must be no ',
        {
          text: 'interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        ' descendant,\n   ',
        {
          text: 'a',
          href: 'https://html.spec.whatwg.org/#the-a-element',
          hashText: '#the-a-element',
        },
        ' element descendant, or descendant with the ',
        {
          text: 'tabindex',
          href: 'https://html.spec.whatwg.org/#attr-tabindex',
          hashText: '#attr-tabindex',
        },
        ' attribute specified.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Si el elemento tiene un atributo href: contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: [
      'Transparente, pero no debe haber contenido interactivo descendiente, elementos a descendientes, ni descendientes con el atributo tabindex especificado.',
    ],
    params_es: ['href'],
  },
  abbr: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '2+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '7+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  address: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: [
      'Flow content, but with no heading content descendants, no sectioning content\n   descendants, and no header, footer, or\n   address element descendants.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no ',
        {
          text: 'heading content',
          href: 'https://html.spec.whatwg.org/#heading-content-2',
          hashText: '#heading-content-2',
        },
        ' descendants, no ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        '\n   descendants, and no ',
        {
          text: 'header',
          href: 'https://html.spec.whatwg.org/#the-header-element',
          hashText: '#the-header-element',
        },
        ', ',
        {
          text: 'footer',
          href: 'https://html.spec.whatwg.org/#the-footer-element',
          hashText: '#the-footer-element',
        },
        ', or\n   ',
        {
          text: 'address',
          href: 'https://html.spec.whatwg.org/#the-address-element',
          hashText: '#the-address-element',
        },
        ' element descendants.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: [
      'Contenido de flujo, pero sin contenido de encabezado descendiente, sin contenido de seccionado descendiente y sin elementos header, footer o address descendientes.',
    ],
    params_es: [],
  },
  area: {
    categories: ['Flow content.', 'Phrasing content.'],
    contexts: ['Where phrasing content is expected, but only if there is a map element ancestor.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected, but only if there is a ',
        {
          text: 'map',
          href: 'https://html.spec.whatwg.org/#the-map-element',
          hashText: '#the-map-element',
        },
        ' element ancestor.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: [
      'Donde se espera contenido de frases, pero solo si hay un ancestro elemento map.',
    ],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  article: {
    categories: ['Flow content.', 'Sectioning content.', 'Palpable content.'],
    contexts: ['Where sectioning content is expected.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4.2+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de seccionado.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de seccionado.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  aside: {
    categories: ['Flow content.', 'Sectioning content.', 'Palpable content.'],
    contexts: ['Where sectioning content is expected.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4.2+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de seccionado.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de seccionado.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  audio: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Embedded content.',
      'If the element has a controls attribute: Interactive content.',
      'If the element has a controls attribute: Palpable content.',
    ],
    contexts: ['Where embedded content is expected.'],
    contentModel: [
      'If the element has a src attribute:\nzero or more track elements, then\ntransparent, but with no media element descendants.',
      'If the element does not have a src attribute: zero or more source elements, then\n zero or more track elements, then\n transparent, but with no media element descendants.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '3.5+',
        WebAPI: '3.5+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '3.1+',
        WebAPI: '3.1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '3+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '10.5+',
        WebAPI: '10.5+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '9+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '2+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '3+',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '11+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'controls',
        href: 'https://html.spec.whatwg.org/#attr-media-controls',
        hashText: '#attr-media-controls',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
      [
        'If the element has a ',
        {
          text: 'controls',
          href: 'https://html.spec.whatwg.org/#attr-media-controls',
          hashText: '#attr-media-controls',
        },
        ' attribute: ',
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        'If the element has a ',
        {
          text: 'controls',
          href: 'https://html.spec.whatwg.org/#attr-media-controls',
          hashText: '#attr-media-controls',
        },
        ' attribute: ',
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'If the element has a ',
        {
          text: 'src',
          href: 'https://html.spec.whatwg.org/#attr-media-src',
          hashText: '#attr-media-src',
        },
        ' attribute:\nzero or more ',
        {
          text: 'track',
          href: 'https://html.spec.whatwg.org/#the-track-element',
          hashText: '#the-track-element',
        },
        ' elements, then\n',
        {
          text: 'transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        ', but with no ',
        {
          text: 'media element',
          href: 'https://html.spec.whatwg.org/#media-element',
          hashText: '#media-element',
        },
        ' descendants.',
      ],
      [
        'If the element does not have a ',
        {
          text: 'src',
          href: 'https://html.spec.whatwg.org/#attr-media-src',
          hashText: '#attr-media-src',
        },
        ' attribute: zero or more ',
        {
          text: 'source',
          href: 'https://html.spec.whatwg.org/#the-source-element',
          hashText: '#the-source-element',
        },
        ' elements, then\n zero or more ',
        {
          text: 'track',
          href: 'https://html.spec.whatwg.org/#the-track-element',
          hashText: '#the-track-element',
        },
        ' elements, then\n ',
        {
          text: 'transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        ', but with no ',
        {
          text: 'media element',
          href: 'https://html.spec.whatwg.org/#media-element',
          hashText: '#media-element',
        },
        ' descendants.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido incrustado.',
      'Si el elemento tiene un atributo controls: contenido interactivo.',
      'Si el elemento tiene un atributo controls: contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: [
      'Si el elemento tiene un atributo src: cero o más elementos track, luego transparente, pero sin elementos de medios (media) descendientes.',
      'Si el elemento no tiene un atributo src: cero o más elementos source, luego cero o más elementos track, luego transparente, pero sin elementos de medios (media) descendientes.',
    ],
    params_es: ['controls'],
  },
  b: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  base: {
    categories: ['Metadata content.'],
    contexts: ['In a head element containing no other base elements.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'In a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element containing no other ',
        {
          text: 'base',
          href: 'https://html.spec.whatwg.org/#the-base-element',
          hashText: '#the-base-element',
        },
        ' elements.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de metadatos.'],
    contexts_es: ['En un elemento head que no contenga otros elementos base.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  bdi: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '10+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '6+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '16+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '10+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '6+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '37+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '14+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  bdo: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  blockquote: {
    categories: ['Flow content.', 'Sectioning root.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '16+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Raíz de seccionado.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  body: {
    categories: ['Sectioning root.'],
    contexts: ['As the second element in an html element.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'As the second element in an ',
        {
          text: 'html',
          href: 'https://html.spec.whatwg.org/#the-html-element',
          hashText: '#the-html-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Raíz de seccionado.'],
    contexts_es: ['Como segundo elemento en un elemento html.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  br: {
    categories: ['Flow content.', 'Phrasing content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  button: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Interactive content.',
      'Listed, labelable, submittable, and autocapitalize-inheriting form-associated element.',
      'Palpable content.',
    ],
    contexts: ['Where phrasing content is expected.'],
    contentModel: [
      'Phrasing content, but there must be no interactive content\n   descendant and no descendant with the tabindex attribute\n   specified.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ', ',
        {
          text: 'labelable',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        ', ',
        {
          text: 'submittable',
          href: 'https://html.spec.whatwg.org/#category-submit',
          hashText: '#category-submit',
        },
        ', and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', but there must be no ',
        {
          text: 'interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '\n   descendant and no descendant with the ',
        {
          text: 'tabindex',
          href: 'https://html.spec.whatwg.org/#attr-tabindex',
          hashText: '#attr-tabindex',
        },
        ' attribute\n   specified.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido interactivo.',
      'Elemento asociado a formulario enumerado, etiquetable, submisible y con herencia de autocapitalización.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: [
      'Contenido de frases, pero no debe haber contenido interactivo descendiente ni descendientes con el atributo tabindex especificado.',
    ],
    params_es: [],
  },
  caption: {
    categories: ['None.'],
    contexts: ['As the first element child of a table element.'],
    contentModel: ['Flow content, but with no descendant table elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As the first element child of a ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no descendant ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como primer hijo de elemento de un elemento table.'],
    contentModel_es: ['Contenido de flujo, pero sin elementos table descendientes.'],
    params_es: [],
  },
  cite: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  code: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  col: {
    categories: ['None.'],
    contexts: ["As a child of a colgroup element that doesn't have\n   a span attribute."],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        " element that doesn't have\n   a ",
        {
          text: 'span',
          href: 'https://html.spec.whatwg.org/#attr-col-span',
          hashText: '#attr-col-span',
        },
        ' attribute.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como hijo de un elemento colgroup que no tiene un atributo span.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  colgroup: {
    categories: ['None.'],
    contexts: [
      'As a child of a table element, after any\n   caption elements and before any thead,\n   tbody, tfoot, and tr\n   elements.',
    ],
    contentModel: [
      'If the span attribute is present: Nothing.',
      'If the span attribute is absent: Zero or more col and template elements.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element, after any\n   ',
        {
          text: 'caption',
          href: 'https://html.spec.whatwg.org/#the-caption-element',
          hashText: '#the-caption-element',
        },
        ' elements and before any ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        ',\n   ',
        {
          text: 'tbody',
          href: 'https://html.spec.whatwg.org/#the-tbody-element',
          hashText: '#the-tbody-element',
        },
        ', ',
        {
          text: 'tfoot',
          href: 'https://html.spec.whatwg.org/#the-tfoot-element',
          hashText: '#the-tfoot-element',
        },
        ', and ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        '\n   elements.',
      ],
    ],
    rawContentModel: [
      [
        'If the ',
        {
          text: 'span',
          href: 'https://html.spec.whatwg.org/#attr-colgroup-span',
          hashText: '#attr-colgroup-span',
        },
        ' attribute is present: ',
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'span',
          href: 'https://html.spec.whatwg.org/#attr-colgroup-span',
          hashText: '#attr-colgroup-span',
        },
        ' attribute is absent: Zero or more ',
        {
          text: 'col',
          href: 'https://html.spec.whatwg.org/#the-col-element',
          hashText: '#the-col-element',
        },
        ' and ',
        {
          text: 'template',
          href: 'https://html.spec.whatwg.org/#the-template-element',
          hashText: '#the-template-element',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento table, después de cualquier elemento caption y antes de cualquier elemento thead, tbody, tfoot y tr.',
    ],
    contentModel_es: [
      'Si el atributo span está presente: nada.',
      'Si el atributo span está ausente: cero o más elementos col y template.',
    ],
    params_es: [],
  },
  data: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '22+',
        WebAPI: '22+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '62+',
        WebAPI: '62+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '49+',
        WebAPI: '49+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '18',
        WebAPI: '14+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '22+',
        WebAPI: '22+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '62+',
        WebAPI: '62+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '62+',
        WebAPI: '62+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '8.0+',
        WebAPI: '8.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '46+',
        WebAPI: '46+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  datalist: {
    categories: ['Flow content.', 'Phrasing content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: [
      'Either: phrasing content.',
      'Or: Zero or more option and script-supporting elements.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '20+',
        WebAPI: '20+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '9.5+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '12.2+',
        WebAPI: '12.2+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '33+',
        WebAPI: '25+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '4.4.3+',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '2.0+',
        WebAPI: '1.5+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '🔰 Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Either: ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        'Or: Zero or more ',
        {
          text: 'option',
          href: 'https://html.spec.whatwg.org/#the-option-element',
          hashText: '#the-option-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: [
      'O bien: contenido de frases.',
      'O bien: cero o más elementos option y de soporte de script.',
    ],
    params_es: [],
  },
  dd: {
    categories: ['None.'],
    contexts: [
      'After dt or dd elements inside dl elements.',
      'After dt or dd elements inside div elements that are children of a dl element.',
    ],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'After ',
        {
          text: 'dt',
          href: 'https://html.spec.whatwg.org/#the-dt-element',
          hashText: '#the-dt-element',
        },
        ' or ',
        {
          text: 'dd',
          href: 'https://html.spec.whatwg.org/#the-dd-element',
          hashText: '#the-dd-element',
        },
        ' elements inside ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' elements.',
      ],
      [
        'After ',
        {
          text: 'dt',
          href: 'https://html.spec.whatwg.org/#the-dt-element',
          hashText: '#the-dt-element',
        },
        ' or ',
        {
          text: 'dd',
          href: 'https://html.spec.whatwg.org/#the-dd-element',
          hashText: '#the-dd-element',
        },
        ' elements inside ',
        {
          text: 'div',
          href: 'https://html.spec.whatwg.org/#the-div-element',
          hashText: '#the-div-element',
        },
        ' elements that are children of a ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Después de elementos dt o dd dentro de elementos dl.',
      'Después de elementos dt o dd dentro de elementos div que son hijos de un elemento dl.',
    ],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  del: {
    categories: ['Flow content.', 'Phrasing content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Transparent.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Transparente.'],
    params_es: [],
  },
  details: {
    categories: ['Flow content.', 'Sectioning root.', 'Interactive content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['One summary element followed by flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '49+',
        WebAPI: '49+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '12+',
        WebAPI: '10+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '15+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '49+',
        WebAPI: '49+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '6.1+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '14+',
        WebAPI: '14+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'One ',
        {
          text: 'summary',
          href: 'https://html.spec.whatwg.org/#the-summary-element',
          hashText: '#the-summary-element',
        },
        ' element followed by ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Raíz de seccionado.',
      'Contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Un elemento summary seguido de contenido de flujo.'],
    params_es: [],
  },
  dfn: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content, but there must be no dfn element descendants.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', but there must be no ',
        {
          text: 'dfn',
          href: 'https://html.spec.whatwg.org/#the-dfn-element',
          hashText: '#the-dfn-element',
        },
        ' element descendants.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases, pero no debe haber elementos dfn descendientes.'],
    params_es: [],
  },
  dialog: {
    categories: ['Flow content.', 'Sectioning root.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '🔰 53+',
        WebAPI: '🔰 53+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '37+',
        WebAPI: '37+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '24+',
        WebAPI: '24+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '🔰 53+',
        WebAPI: '🔰 53+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '37+',
        WebAPI: '37+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '37+',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '3.0+',
        WebAPI: '3.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '24+',
        WebAPI: '24+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Raíz de seccionado.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  div: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.', 'As a child of a dl element.'],
    contentModel: [
      'If the element is a child of a dl element: one or more dt elements followed by one or more dd elements, optionally intermixed with script-supporting elements.',
      'If the element is not a child of a dl element: flow content.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
      [
        'As a child of a ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'If the element is a child of a ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' element: one or more ',
        {
          text: 'dt',
          href: 'https://html.spec.whatwg.org/#the-dt-element',
          hashText: '#the-dt-element',
        },
        ' elements followed by one or more ',
        {
          text: 'dd',
          href: 'https://html.spec.whatwg.org/#the-dd-element',
          hashText: '#the-dd-element',
        },
        ' elements, optionally intermixed with ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
      [
        'If the element is not a child of a ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' element: ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.', 'Como hijo de un elemento dl.'],
    contentModel_es: [
      'Si el elemento es hijo de un elemento dl: uno o más elementos dt seguidos de uno o más elementos dd, opcionalmente intercalados con elementos de soporte de script.',
      'Si el elemento no es hijo de un elemento dl: contenido de flujo.',
    ],
    params_es: [],
  },
  dl: {
    categories: [
      'Flow content.',
      "If the element's children include at least one name-value group: Palpable content.",
    ],
    contexts: ['Where flow content is expected.'],
    contentModel: [
      'Either: Zero or more groups each consisting of one or more dt elements followed by one or more dd elements, optionally intermixed with script-supporting elements.',
      'Or: One or more div elements, optionally intermixed with script-supporting elements.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        "If the element's children include at least one name-value group: ",
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Either: Zero or more groups each consisting of one or more ',
        {
          text: 'dt',
          href: 'https://html.spec.whatwg.org/#the-dt-element',
          hashText: '#the-dt-element',
        },
        ' elements followed by one or more ',
        {
          text: 'dd',
          href: 'https://html.spec.whatwg.org/#the-dd-element',
          hashText: '#the-dd-element',
        },
        ' elements, optionally intermixed with ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
      [
        'Or: One or more ',
        {
          text: 'div',
          href: 'https://html.spec.whatwg.org/#the-div-element',
          hashText: '#the-div-element',
        },
        ' elements, optionally intermixed with ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Si los hijos del elemento incluyen al menos un grupo nombre-valor: contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: [
      'O bien: cero o más grupos, cada uno formado por uno o más elementos dt seguidos de uno o más elementos dd, opcionalmente intercalados con elementos de soporte de script.',
      'O bien: uno o más elementos div, opcionalmente intercalados con elementos de soporte de script.',
    ],
    params_es: [],
  },
  dt: {
    categories: ['None.'],
    contexts: [
      'Before dd or dt elements inside dl elements.',
      'Before dd or dt elements inside div elements that are children of a dl element.',
    ],
    contentModel: [
      'Flow content, but with no header, footer, sectioning content, or heading content descendants.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'Before ',
        {
          text: 'dd',
          href: 'https://html.spec.whatwg.org/#the-dd-element',
          hashText: '#the-dd-element',
        },
        ' or ',
        {
          text: 'dt',
          href: 'https://html.spec.whatwg.org/#the-dt-element',
          hashText: '#the-dt-element',
        },
        ' elements inside ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' elements.',
      ],
      [
        'Before ',
        {
          text: 'dd',
          href: 'https://html.spec.whatwg.org/#the-dd-element',
          hashText: '#the-dd-element',
        },
        ' or ',
        {
          text: 'dt',
          href: 'https://html.spec.whatwg.org/#the-dt-element',
          hashText: '#the-dt-element',
        },
        ' elements inside ',
        {
          text: 'div',
          href: 'https://html.spec.whatwg.org/#the-div-element',
          hashText: '#the-div-element',
        },
        ' elements that are children of a ',
        {
          text: 'dl',
          href: 'https://html.spec.whatwg.org/#the-dl-element',
          hashText: '#the-dl-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no ',
        {
          text: 'header',
          href: 'https://html.spec.whatwg.org/#the-header-element',
          hashText: '#the-header-element',
        },
        ', ',
        {
          text: 'footer',
          href: 'https://html.spec.whatwg.org/#the-footer-element',
          hashText: '#the-footer-element',
        },
        ', ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        ', or ',
        {
          text: 'heading content',
          href: 'https://html.spec.whatwg.org/#heading-content-2',
          hashText: '#heading-content-2',
        },
        ' descendants.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Antes de elementos dd o dt dentro de elementos dl.',
      'Antes de elementos dd o dt dentro de elementos div que son hijos de un elemento dl.',
    ],
    contentModel_es: [
      'Contenido de flujo, pero sin header, footer, contenido de seccionado ni contenido de encabezado descendientes.',
    ],
    params_es: [],
  },
  em: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  embed: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Embedded content.',
      'Interactive content.',
      'Palpable content.',
    ],
    contexts: ['Where embedded content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido incrustado.',
      'Contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  fieldset: {
    categories: [
      'Flow content.',
      'Sectioning root.',
      'Listed and autocapitalize-inheriting form-associated element.',
      'Palpable content.',
    ],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Optionally a legend element, followed by flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
      [
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ' and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Optionally a ',
        {
          text: 'legend',
          href: 'https://html.spec.whatwg.org/#the-legend-element',
          hashText: '#the-legend-element',
        },
        ' element, followed by ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Raíz de seccionado.',
      'Elemento asociado a formulario enumerado y con herencia de autocapitalización.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Opcionalmente un elemento legend, seguido de contenido de flujo.'],
    params_es: [],
  },
  figcaption: {
    categories: ['None.'],
    contexts: ['As the first or last child of a figure element.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '8+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As the first or last child of a ',
        {
          text: 'figure',
          href: 'https://html.spec.whatwg.org/#the-figure-element',
          hashText: '#the-figure-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como primer o último hijo de un elemento figure.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  figure: {
    categories: ['Flow content.', 'Sectioning root.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: [
      'Either: one figcaption element followed by flow content.',
      'Or: flow content followed by one figcaption element.',
      'Or: flow content.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '8+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Either: one ',
        {
          text: 'figcaption',
          href: 'https://html.spec.whatwg.org/#the-figcaption-element',
          hashText: '#the-figcaption-element',
        },
        ' element followed by ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        'Or: ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' followed by one ',
        {
          text: 'figcaption',
          href: 'https://html.spec.whatwg.org/#the-figcaption-element',
          hashText: '#the-figcaption-element',
        },
        ' element.',
      ],
      [
        'Or: ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Raíz de seccionado.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: [
      'O bien: un elemento figcaption seguido de contenido de flujo.',
      'O bien: contenido de flujo seguido de un elemento figcaption.',
      'O bien: contenido de flujo.',
    ],
    params_es: [],
  },
  footer: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Flow content, but with no header or footer element\n   descendants.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4.2+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no ',
        {
          text: 'header',
          href: 'https://html.spec.whatwg.org/#the-header-element',
          hashText: '#the-header-element',
        },
        ' or ',
        {
          text: 'footer',
          href: 'https://html.spec.whatwg.org/#the-footer-element',
          hashText: '#the-footer-element',
        },
        ' element\n   descendants.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de flujo, pero sin elementos header o footer descendientes.'],
    params_es: [],
  },
  form: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Flow content, but with no form element descendants.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '8+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '10.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no ',
        {
          text: 'form',
          href: 'https://html.spec.whatwg.org/#the-form-element',
          hashText: '#the-form-element',
        },
        ' element descendants.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de flujo, pero sin elementos form descendientes.'],
    params_es: [],
  },
  head: {
    categories: ['None.'],
    contexts: ['As the first element in an html element.'],
    contentModel: [
      'If the document is an iframe srcdoc document or if title information is available from a higher-level protocol: Zero or more elements of metadata content, of which no more than one is a title element and no more than one is a base element.',
      'Otherwise: One or more elements of metadata content, of which exactly one is a title element and no more than one is a base element.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As the first element in an ',
        {
          text: 'html',
          href: 'https://html.spec.whatwg.org/#the-html-element',
          hashText: '#the-html-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'If the document is ',
        {
          text: 'an iframe srcdoc document',
          href: 'https://html.spec.whatwg.org/#an-iframe-srcdoc-document',
          hashText: '#an-iframe-srcdoc-document',
        },
        ' or if title information is available from a higher-level protocol: Zero or more elements of ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ', of which no more than one is a ',
        {
          text: 'title',
          href: 'https://html.spec.whatwg.org/#the-title-element',
          hashText: '#the-title-element',
        },
        ' element and no more than one is a ',
        {
          text: 'base',
          href: 'https://html.spec.whatwg.org/#the-base-element',
          hashText: '#the-base-element',
        },
        ' element.',
      ],
      [
        'Otherwise: One or more elements of ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ', of which exactly one is a ',
        {
          text: 'title',
          href: 'https://html.spec.whatwg.org/#the-title-element',
          hashText: '#the-title-element',
        },
        ' element and no more than one is a ',
        {
          text: 'base',
          href: 'https://html.spec.whatwg.org/#the-base-element',
          hashText: '#the-base-element',
        },
        ' element.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como primer elemento en un elemento html.'],
    contentModel_es: [
      'Si el documento es un documento srcdoc de un iframe o si la información de title está disponible desde un protocolo de nivel superior: cero o más elementos de contenido de metadatos, de los cuales no más de uno es un elemento title y no más de uno es un elemento base.',
      'De otro modo: uno o más elementos de contenido de metadatos, de los cuales exactamente uno es un elemento title y no más de uno es un elemento base.',
    ],
    params_es: [],
  },
  header: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Flow content, but with no header or footer element\n   descendants.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4.2+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no ',
        {
          text: 'header',
          href: 'https://html.spec.whatwg.org/#the-header-element',
          hashText: '#the-header-element',
        },
        ' or ',
        {
          text: 'footer',
          href: 'https://html.spec.whatwg.org/#the-footer-element',
          hashText: '#the-footer-element',
        },
        ' element\n   descendants.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de flujo, pero sin elementos header o footer descendientes.'],
    params_es: [],
  },
  hr: {
    categories: ['Flow content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '3+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5.5+',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  html: {
    categories: ['None.'],
    contexts: [
      "As document's document element.",
      'Wherever a subdocument fragment is allowed in a compound document.',
    ],
    contentModel: ['A head element followed by a body element.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        "As document's ",
        {
          text: 'document element',
          href: 'https://dom.spec.whatwg.org/#document-element',
          hashText: '#document-element',
        },
        '.',
      ],
      ['Wherever a subdocument fragment is allowed in a compound document.'],
    ],
    rawContentModel: [
      [
        'A ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element followed by a ',
        {
          text: 'body',
          href: 'https://html.spec.whatwg.org/#the-body-element',
          hashText: '#the-body-element',
        },
        ' element.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como elemento document del documento.',
      'Dondequiera que se permita un fragmento de subdocumento en un documento compuesto.',
    ],
    contentModel_es: ['Un elemento head seguido de un elemento body.'],
    params_es: [],
  },
  i: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  iframe: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Embedded content.',
      'Interactive content.',
      'Palpable content.',
    ],
    contexts: ['Where embedded content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido incrustado.',
      'Contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  img: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Embedded content.',
      'Form-associated element.',
      'If the element has a usemap attribute: Interactive content.',
      'Palpable content.',
    ],
    contexts: ['Where embedded content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '8+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '10.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'usemap',
        href: 'https://html.spec.whatwg.org/#attr-hyperlink-usemap',
        hashText: '#attr-hyperlink-usemap',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
      [
        {
          text: 'Form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        'If the element has a ',
        {
          text: 'usemap',
          href: 'https://html.spec.whatwg.org/#attr-hyperlink-usemap',
          hashText: '#attr-hyperlink-usemap',
        },
        ' attribute: ',
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido incrustado.',
      'Elemento asociado a formulario.',
      'Si el elemento tiene un atributo usemap: contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: ['Nada.'],
    params_es: ['usemap'],
  },
  input: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'If the type attribute is not in the Hidden state: Interactive content.',
      'If the type attribute is not in the Hidden state: Listed, labelable, submittable, resettable, and autocapitalize-inheriting form-associated element.',
      'If the type attribute is in the Hidden state: Listed, submittable, resettable, and autocapitalize-inheriting form-associated element.',
      'If the type attribute is not in the Hidden state: Palpable content.',
    ],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '8+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '10.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'Hidden',
        href: 'https://html.spec.whatwg.org/#hidden-state-(type=hidden)',
        hashText: '#hidden-state-(type=hidden)',
      },
      {
        text: 'type',
        href: 'https://html.spec.whatwg.org/#attr-input-type',
        hashText: '#attr-input-type',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'type',
          href: 'https://html.spec.whatwg.org/#attr-input-type',
          hashText: '#attr-input-type',
        },
        ' attribute is ',
        'not',
        ' in the ',
        {
          text: 'Hidden',
          href: 'https://html.spec.whatwg.org/#hidden-state-(type=hidden)',
          hashText: '#hidden-state-(type=hidden)',
        },
        ' state: ',
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'type',
          href: 'https://html.spec.whatwg.org/#attr-input-type',
          hashText: '#attr-input-type',
        },
        ' attribute is ',
        'not',
        ' in the ',
        {
          text: 'Hidden',
          href: 'https://html.spec.whatwg.org/#hidden-state-(type=hidden)',
          hashText: '#hidden-state-(type=hidden)',
        },
        ' state: ',
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ', ',
        {
          text: 'labelable',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        ', ',
        {
          text: 'submittable',
          href: 'https://html.spec.whatwg.org/#category-submit',
          hashText: '#category-submit',
        },
        ', ',
        {
          text: 'resettable',
          href: 'https://html.spec.whatwg.org/#category-reset',
          hashText: '#category-reset',
        },
        ', and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'type',
          href: 'https://html.spec.whatwg.org/#attr-input-type',
          hashText: '#attr-input-type',
        },
        ' attribute is in the ',
        {
          text: 'Hidden',
          href: 'https://html.spec.whatwg.org/#hidden-state-(type=hidden)',
          hashText: '#hidden-state-(type=hidden)',
        },
        ' state: ',
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ', ',
        {
          text: 'submittable',
          href: 'https://html.spec.whatwg.org/#category-submit',
          hashText: '#category-submit',
        },
        ', ',
        {
          text: 'resettable',
          href: 'https://html.spec.whatwg.org/#category-reset',
          hashText: '#category-reset',
        },
        ', and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'type',
          href: 'https://html.spec.whatwg.org/#attr-input-type',
          hashText: '#attr-input-type',
        },
        ' attribute is ',
        'not',
        ' in the ',
        {
          text: 'Hidden',
          href: 'https://html.spec.whatwg.org/#hidden-state-(type=hidden)',
          hashText: '#hidden-state-(type=hidden)',
        },
        ' state: ',
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Si el atributo type no está en el estado Hidden: contenido interactivo.',
      'Si el atributo type no está en el estado Hidden: elemento asociado a formulario enumerado, etiquetable, submisible, restablecible y con herencia de autocapitalización.',
      'Si el atributo type está en el estado Hidden: elemento asociado a formulario enumerado, submisible, restablecible y con herencia de autocapitalización.',
      'Si el atributo type no está en el estado Hidden: contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Nada.'],
    params_es: ['Oculto', 'type'],
  },
  ins: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Transparent.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Transparente.'],
    params_es: [],
  },
  kbd: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  label: {
    categories: ['Flow content.', 'Phrasing content.', 'Interactive content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: [
      "Phrasing content, but with no descendant labelable elements unless it is the element's labeled control, and no descendant label elements.",
    ],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', but with no descendant ',
        {
          text: 'labelable elements',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        " unless it is the element's ",
        {
          text: 'labeled control',
          href: 'https://html.spec.whatwg.org/#labeled-control',
          hashText: '#labeled-control',
        },
        ', and no descendant ',
        {
          text: 'label',
          href: 'https://html.spec.whatwg.org/#the-label-element',
          hashText: '#the-label-element',
        },
        ' elements.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: [
      'Contenido de frases, pero sin elementos etiquetables descendientes a menos que sea el control etiquetado del elemento, y sin elementos label descendientes.',
    ],
    params_es: [],
  },
  legend: {
    categories: ['None.'],
    contexts: ['As the first child of a fieldset element.'],
    contentModel: ['Phrasing content, optionally intermixed with heading content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '3+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As the ',
        {
          text: 'first child',
          href: 'https://dom.spec.whatwg.org/#concept-tree-first-child',
          hashText: '#concept-tree-first-child',
        },
        ' of a ',
        {
          text: 'fieldset',
          href: 'https://html.spec.whatwg.org/#the-fieldset-element',
          hashText: '#the-fieldset-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', optionally intermixed with ',
        {
          text: 'heading content',
          href: 'https://html.spec.whatwg.org/#heading-content-2',
          hashText: '#heading-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como primer hijo de un elemento fieldset.'],
    contentModel_es: [
      'Contenido de frases, opcionalmente intercalado con contenido de encabezado.',
    ],
    params_es: [],
  },
  li: {
    categories: ['None.'],
    contexts: ['Inside ol elements.', 'Inside ul elements.', 'Inside menu elements.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '3+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5.5+',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'Inside ',
        {
          text: 'ol',
          href: 'https://html.spec.whatwg.org/#the-ol-element',
          hashText: '#the-ol-element',
        },
        ' elements.',
      ],
      [
        'Inside ',
        {
          text: 'ul',
          href: 'https://html.spec.whatwg.org/#the-ul-element',
          hashText: '#the-ul-element',
        },
        ' elements.',
      ],
      [
        'Inside ',
        {
          text: 'menu',
          href: 'https://html.spec.whatwg.org/#the-menu-element',
          hashText: '#the-menu-element',
        },
        ' elements.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Dentro de elementos ol.',
      'Dentro de elementos ul.',
      'Dentro de elementos menu.',
    ],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  link: {
    categories: [
      'Metadata content.',
      'If the element is allowed in the body: flow content.',
      'If the element is allowed in the body: phrasing content.',
    ],
    contexts: [
      'Where metadata content is expected.',
      'In a noscript element that is a child of a head element.',
      'If the element is allowed in the body: where phrasing content is expected.',
    ],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'allowed in the body',
        href: 'https://html.spec.whatwg.org/#allowed-in-the-body',
        hashText: '#allowed-in-the-body',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
      [
        'If the element is ',
        {
          text: 'allowed in the body',
          href: 'https://html.spec.whatwg.org/#allowed-in-the-body',
          hashText: '#allowed-in-the-body',
        },
        ': ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        'If the element is ',
        {
          text: 'allowed in the body',
          href: 'https://html.spec.whatwg.org/#allowed-in-the-body',
          hashText: '#allowed-in-the-body',
        },
        ': ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ' is expected.',
      ],
      [
        'In a ',
        {
          text: 'noscript',
          href: 'https://html.spec.whatwg.org/#the-noscript-element',
          hashText: '#the-noscript-element',
        },
        ' element that is a child of a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element.',
      ],
      [
        'If the element is ',
        {
          text: 'allowed in the body',
          href: 'https://html.spec.whatwg.org/#allowed-in-the-body',
          hashText: '#allowed-in-the-body',
        },
        ': where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de metadatos.',
      'Si el elemento está permitido en el body: contenido de flujo.',
      'Si el elemento está permitido en el body: contenido de frases.',
    ],
    contexts_es: [
      'Donde se espera contenido de metadatos.',
      'En un elemento noscript que es hijo de un elemento head.',
      'Si el elemento está permitido en el body: donde se espera contenido de frases.',
    ],
    contentModel_es: ['Nada.'],
    params_es: ['permitido en el body'],
  },
  main: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: [
      'Where flow content is expected, but only if it is a hierarchically correct main element.',
    ],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '21+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '7+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '26+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '16+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '21+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '7+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected, but only if it is a ',
        {
          text: 'hierarchically correct main element',
          href: 'https://html.spec.whatwg.org/#hierarchically-correct-main-element',
          hashText: '#hierarchically-correct-main-element',
        },
        '.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: [
      'Donde se espera contenido de flujo, pero solo si es un elemento main jerárquicamente correcto.',
    ],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  map: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Transparent.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '1+',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Transparente.'],
    params_es: [],
  },
  mark: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  menu: {
    categories: [
      'Flow content.',
      "If the element's children include at least one li element: Palpable content.",
    ],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Zero or more li and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '3+',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '12.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'li',
        href: 'https://html.spec.whatwg.org/#the-li-element',
        hashText: '#the-li-element',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        "If the element's children include at least one ",
        {
          text: 'li',
          href: 'https://html.spec.whatwg.org/#the-li-element',
          hashText: '#the-li-element',
        },
        ' element: ',
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'li',
          href: 'https://html.spec.whatwg.org/#the-li-element',
          hashText: '#the-li-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Si los hijos del elemento incluyen al menos un elemento li: contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Cero o más elementos li y de soporte de script.'],
    params_es: ['li'],
  },
  meta: {
    categories: [
      'Metadata content.',
      'If the itemprop attribute is present: flow content.',
      'If the itemprop attribute is present: phrasing content.',
    ],
    contexts: [
      "If the charset attribute is present, or if the element's http-equiv attribute is in the Encoding declaration state: in a head element.",
      'If the http-equiv attribute is present but not in the Encoding declaration state: in a head element.',
      'If the http-equiv attribute is present but not in the Encoding declaration state: in a noscript element that is a child of a head element.',
      'If the name attribute is present: where metadata content is expected.',
      'If the itemprop attribute is present: where metadata content is expected.',
      'If the itemprop attribute is present: where phrasing content is expected.',
    ],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'itemprop',
        href: 'https://html.spec.whatwg.org/#names:-the-itemprop-attribute',
        hashText: '#names:-the-itemprop-attribute',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'itemprop',
          href: 'https://html.spec.whatwg.org/#names:-the-itemprop-attribute',
          hashText: '#names:-the-itemprop-attribute',
        },
        ' attribute is present: ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        'If the ',
        {
          text: 'itemprop',
          href: 'https://html.spec.whatwg.org/#names:-the-itemprop-attribute',
          hashText: '#names:-the-itemprop-attribute',
        },
        ' attribute is present: ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'If the ',
        {
          text: 'charset',
          href: 'https://html.spec.whatwg.org/#attr-meta-charset',
          hashText: '#attr-meta-charset',
        },
        " attribute is present, or if the element's ",
        {
          text: 'http-equiv',
          href: 'https://html.spec.whatwg.org/#attr-meta-http-equiv',
          hashText: '#attr-meta-http-equiv',
        },
        ' attribute is in the ',
        {
          text: 'Encoding declaration state',
          href: 'https://html.spec.whatwg.org/#attr-meta-http-equiv-content-type',
          hashText: '#attr-meta-http-equiv-content-type',
        },
        ': in a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element.',
      ],
      [
        'If the ',
        {
          text: 'http-equiv',
          href: 'https://html.spec.whatwg.org/#attr-meta-http-equiv',
          hashText: '#attr-meta-http-equiv',
        },
        ' attribute is present but not in the ',
        {
          text: 'Encoding declaration state',
          href: 'https://html.spec.whatwg.org/#attr-meta-http-equiv-content-type',
          hashText: '#attr-meta-http-equiv-content-type',
        },
        ': in a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element.',
      ],
      [
        'If the ',
        {
          text: 'http-equiv',
          href: 'https://html.spec.whatwg.org/#attr-meta-http-equiv',
          hashText: '#attr-meta-http-equiv',
        },
        ' attribute is present but not in the ',
        {
          text: 'Encoding declaration state',
          href: 'https://html.spec.whatwg.org/#attr-meta-http-equiv-content-type',
          hashText: '#attr-meta-http-equiv-content-type',
        },
        ': in a ',
        {
          text: 'noscript',
          href: 'https://html.spec.whatwg.org/#the-noscript-element',
          hashText: '#the-noscript-element',
        },
        ' element that is a child of a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element.',
      ],
      [
        'If the ',
        {
          text: 'name',
          href: 'https://html.spec.whatwg.org/#attr-meta-name',
          hashText: '#attr-meta-name',
        },
        ' attribute is present: where ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ' is expected.',
      ],
      [
        'If the ',
        {
          text: 'itemprop',
          href: 'https://html.spec.whatwg.org/#names:-the-itemprop-attribute',
          hashText: '#names:-the-itemprop-attribute',
        },
        ' attribute is present: where ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ' is expected.',
      ],
      [
        'If the ',
        {
          text: 'itemprop',
          href: 'https://html.spec.whatwg.org/#names:-the-itemprop-attribute',
          hashText: '#names:-the-itemprop-attribute',
        },
        ' attribute is present: where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de metadatos.',
      'Si el atributo itemprop está presente: contenido de flujo.',
      'Si el atributo itemprop está presente: contenido de frases.',
    ],
    contexts_es: [
      'Si el atributo charset está presente, o si el atributo http-equiv del elemento está en el estado de declaración de codificación: en un elemento head.',
      'Si el atributo http-equiv está presente pero no en el estado de declaración de codificación: en un elemento head.',
      'Si el atributo http-equiv está presente pero no en el estado de declaración de codificación: en un elemento noscript que es hijo de un elemento head.',
      'Si el atributo name está presente: donde se espera contenido de metadatos.',
      'Si el atributo itemprop está presente: donde se espera contenido de metadatos.',
      'Si el atributo itemprop está presente: donde se espera contenido de frases.',
    ],
    contentModel_es: ['Nada.'],
    params_es: ['itemprop'],
  },
  meter: {
    categories: ['Flow content.', 'Phrasing content.', 'Labelable element.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content, but there must be no meter element descendants.'],
    support: {
      Firefox: {
        WebHTMLElement: '16+',
        WebAPI: '16+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11+',
        WebAPI: '11+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '18',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '16+',
        WebAPI: '16+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '10.3+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'No',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11+',
        WebAPI: '11+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Labelable element',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', but there must be no ',
        {
          text: 'meter',
          href: 'https://html.spec.whatwg.org/#the-meter-element',
          hashText: '#the-meter-element',
        },
        ' element descendants.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Elemento etiquetable.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases, pero no debe haber elementos meter descendientes.'],
    params_es: [],
  },
  nav: {
    categories: ['Flow content.', 'Sectioning content.', 'Palpable content.'],
    contexts: ['Where sectioning content is expected.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4.2+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de seccionado.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de seccionado.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  noscript: {
    categories: ['Metadata content.', 'Flow content.', 'Phrasing content.'],
    contexts: [
      'In a head element of an HTML document, if there are no ancestor noscript elements.',
      'Where phrasing content is expected in HTML documents, if there are no ancestor noscript elements.',
    ],
    contentModel: [
      'When scripting is disabled, in a head element: in any order, zero or more link elements, zero or more style elements, and zero or more meta elements.',
      'When scripting is disabled, not in a head element: transparent, but there must be no noscript element descendants.',
      'Otherwise: text that conforms to the requirements given in the prose.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'In a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element of an ',
        {
          text: 'HTML document',
          href: 'https://dom.spec.whatwg.org/#html-document',
          hashText: '#html-document',
        },
        ', if there are no ancestor ',
        {
          text: 'noscript',
          href: 'https://html.spec.whatwg.org/#the-noscript-element',
          hashText: '#the-noscript-element',
        },
        ' elements.',
      ],
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected in ',
        {
          text: 'HTML documents',
          href: 'https://dom.spec.whatwg.org/#html-document',
          hashText: '#html-document',
        },
        ', if there are no ancestor ',
        {
          text: 'noscript',
          href: 'https://html.spec.whatwg.org/#the-noscript-element',
          hashText: '#the-noscript-element',
        },
        ' elements.',
      ],
    ],
    rawContentModel: [
      [
        'When ',
        {
          text: 'scripting is disabled',
          href: 'https://html.spec.whatwg.org/#concept-n-noscript',
          hashText: '#concept-n-noscript',
        },
        ', in a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element: in any order, zero or more ',
        {
          text: 'link',
          href: 'https://html.spec.whatwg.org/#the-link-element',
          hashText: '#the-link-element',
        },
        ' elements, zero or more ',
        {
          text: 'style',
          href: 'https://html.spec.whatwg.org/#the-style-element',
          hashText: '#the-style-element',
        },
        ' elements, and zero or more ',
        {
          text: 'meta',
          href: 'https://html.spec.whatwg.org/#the-meta-element',
          hashText: '#the-meta-element',
        },
        ' elements.',
      ],
      [
        'When ',
        {
          text: 'scripting is disabled',
          href: 'https://html.spec.whatwg.org/#concept-n-noscript',
          hashText: '#concept-n-noscript',
        },
        ', not in a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element: ',
        {
          text: 'transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        ', but there must be no ',
        {
          text: 'noscript',
          href: 'https://html.spec.whatwg.org/#the-noscript-element',
          hashText: '#the-noscript-element',
        },
        ' element descendants.',
      ],
      ['Otherwise: text that conforms to the requirements given in the prose.'],
    ],
    categories_es: ['Contenido de metadatos.', 'Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: [
      'En un elemento head de un documento HTML, si no hay elementos noscript ancestros.',
      'Donde se espera contenido de frases en documentos HTML, si no hay elementos noscript ancestros.',
    ],
    contentModel_es: [
      'Cuando los scripts están deshabilitados, en un elemento head: en cualquier orden, cero o más elementos link, cero o más elementos style y cero o más elementos meta.',
      'Cuando los scripts están deshabilitados, fuera de un elemento head: transparente, pero no debe haber elementos noscript descendientes.',
      'De otro modo: texto que cumpla los requisitos dados en la prosa.',
    ],
    params_es: [],
  },
  object: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Embedded content.',
      'Listed form-associated element.',
      'Palpable content.',
    ],
    contexts: ['Where embedded content is expected.'],
    contentModel: ['Zero or more param elements, then, transparent.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
      [
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'param',
          href: 'https://html.spec.whatwg.org/#the-param-element',
          hashText: '#the-param-element',
        },
        ' elements, then, ',
        {
          text: 'transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido incrustado.',
      'Elemento asociado a formulario enumerado.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: ['Cero o más elementos param, luego, transparente.'],
    params_es: [],
  },
  ol: {
    categories: [
      'Flow content.',
      "If the element's children include at least one li element: Palpable content.",
    ],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Zero or more li and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'li',
        href: 'https://html.spec.whatwg.org/#the-li-element',
        hashText: '#the-li-element',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        "If the element's children include at least one ",
        {
          text: 'li',
          href: 'https://html.spec.whatwg.org/#the-li-element',
          hashText: '#the-li-element',
        },
        ' element: ',
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'li',
          href: 'https://html.spec.whatwg.org/#the-li-element',
          hashText: '#the-li-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Si los hijos del elemento incluyen al menos un elemento li: contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Cero o más elementos li y de soporte de script.'],
    params_es: ['li'],
  },
  optgroup: {
    categories: ['None.'],
    contexts: ['As a child of a select element.'],
    contentModel: ['Zero or more option and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5.5+',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'select',
          href: 'https://html.spec.whatwg.org/#the-select-element',
          hashText: '#the-select-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'option',
          href: 'https://html.spec.whatwg.org/#the-option-element',
          hashText: '#the-option-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como hijo de un elemento select.'],
    contentModel_es: ['Cero o más elementos option y de soporte de script.'],
    params_es: [],
  },
  option: {
    categories: ['None.'],
    contexts: [
      'As a child of a select element.',
      'As a child of a datalist element.',
      'As a child of an optgroup element.',
    ],
    contentModel: [
      'If the element has a label attribute and a value attribute: Nothing.',
      'If the element has a label attribute but no value attribute: Text.',
      'If the element has no label attribute and is not a\n   child of a datalist element: Text that is not\n   inter-element whitespace.',
      'If the element has no label attribute and is a child\n   of a datalist element: Text.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1.2+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'select',
          href: 'https://html.spec.whatwg.org/#the-select-element',
          hashText: '#the-select-element',
        },
        ' element.',
      ],
      [
        'As a child of a ',
        {
          text: 'datalist',
          href: 'https://html.spec.whatwg.org/#the-datalist-element',
          hashText: '#the-datalist-element',
        },
        ' element.',
      ],
      [
        'As a child of an ',
        {
          text: 'optgroup',
          href: 'https://html.spec.whatwg.org/#the-optgroup-element',
          hashText: '#the-optgroup-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'If the element has a ',
        {
          text: 'label',
          href: 'https://html.spec.whatwg.org/#attr-option-label',
          hashText: '#attr-option-label',
        },
        ' attribute and a ',
        {
          text: 'value',
          href: 'https://html.spec.whatwg.org/#attr-option-value',
          hashText: '#attr-option-value',
        },
        ' attribute: ',
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
      [
        'If the element has a ',
        {
          text: 'label',
          href: 'https://html.spec.whatwg.org/#attr-option-label',
          hashText: '#attr-option-label',
        },
        ' attribute but no ',
        {
          text: 'value',
          href: 'https://html.spec.whatwg.org/#attr-option-value',
          hashText: '#attr-option-value',
        },
        ' attribute: ',
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        '.',
      ],
      [
        'If the element has no ',
        {
          text: 'label',
          href: 'https://html.spec.whatwg.org/#attr-option-label',
          hashText: '#attr-option-label',
        },
        ' attribute and is not a\n   child of a ',
        {
          text: 'datalist',
          href: 'https://html.spec.whatwg.org/#the-datalist-element',
          hashText: '#the-datalist-element',
        },
        ' element: ',
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        ' that is not\n   ',
        {
          text: 'inter-element whitespace',
          href: 'https://html.spec.whatwg.org/#inter-element-whitespace',
          hashText: '#inter-element-whitespace',
        },
        '.',
      ],
      [
        'If the element has no ',
        {
          text: 'label',
          href: 'https://html.spec.whatwg.org/#attr-option-label',
          hashText: '#attr-option-label',
        },
        ' attribute and is a child\n   of a ',
        {
          text: 'datalist',
          href: 'https://html.spec.whatwg.org/#the-datalist-element',
          hashText: '#the-datalist-element',
        },
        ' element: ',
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento select.',
      'Como hijo de un elemento datalist.',
      'Como hijo de un elemento optgroup.',
    ],
    contentModel_es: [
      'Si el elemento tiene un atributo label y un atributo value: nada.',
      'Si el elemento tiene un atributo label pero no un atributo value: texto.',
      'Si el elemento no tiene atributo label y no es hijo de un elemento datalist: texto que no es espacio en blanco entre elementos.',
      'Si el elemento no tiene atributo label y es hijo de un elemento datalist: texto.',
    ],
    params_es: [],
  },
  output: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Listed, labelable, resettable, and autocapitalize-inheriting form-associated element.',
      'Palpable content.',
    ],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '7+',
        WebAPI: '5.1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '10+',
        WebAPI: '9+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '18',
        WebAPI: '14+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '5+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '?',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ', ',
        {
          text: 'labelable',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        ', ',
        {
          text: 'resettable',
          href: 'https://html.spec.whatwg.org/#category-reset',
          hashText: '#category-reset',
        },
        ', and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Elemento asociado a formulario enumerado, etiquetable, restablecible y con herencia de autocapitalización.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  p: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  param: {
    categories: ['None.'],
    contexts: ['As a child of an object element, before any flow content.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of an ',
        {
          text: 'object',
          href: 'https://html.spec.whatwg.org/#the-object-element',
          hashText: '#the-object-element',
        },
        ' element, before any ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como hijo de un elemento object, antes de cualquier contenido de flujo.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  picture: {
    categories: ['Flow content.', 'Phrasing content.', 'Embedded content.'],
    contexts: ['Where embedded content is expected.'],
    contentModel: [
      'Zero or more source elements, followed by one img element,\n   optionally intermixed with script-supporting elements.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '38+',
        WebAPI: '38+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '9.1+',
        WebAPI: '9.1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '38+',
        WebAPI: '38+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '25+',
        WebAPI: '25+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '13+',
        WebAPI: '13+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '38+',
        WebAPI: '38+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '9.3+',
        WebAPI: '9.3+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '38+',
        WebAPI: '38+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '38+',
        WebAPI: '38+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '3.0+',
        WebAPI: '3.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '25+',
        WebAPI: '25+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'source',
          href: 'https://html.spec.whatwg.org/#the-source-element',
          hashText: '#the-source-element',
        },
        ' elements, followed by one ',
        {
          text: 'img',
          href: 'https://html.spec.whatwg.org/#the-img-element',
          hashText: '#the-img-element',
        },
        ' element,\n   optionally intermixed with ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido incrustado.'],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: [
      'Cero o más elementos source, seguidos de un elemento img, opcionalmente intercalados con elementos de soporte de script.',
    ],
    params_es: [],
  },
  pre: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  progress: {
    categories: ['Flow content.', 'Phrasing content.', 'Labelable element.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content, but there must be no progress element descendants.'],
    support: {
      Firefox: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '7+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Labelable element',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', but there must be no ',
        {
          text: 'progress',
          href: 'https://html.spec.whatwg.org/#the-progress-element',
          hashText: '#the-progress-element',
        },
        ' element descendants.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Elemento etiquetable.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases, pero no debe haber elementos progress descendientes.'],
    params_es: [],
  },
  q: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  rp: {
    categories: ['None.'],
    contexts: [
      'As a child of a ruby element, either immediately before or immediately after an rt element.',
    ],
    contentModel: ['Text.'],
    support: {
      Firefox: {
        WebHTMLElement: '38+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '38+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '14+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'ruby',
          href: 'https://html.spec.whatwg.org/#the-ruby-element',
          hashText: '#the-ruby-element',
        },
        ' element, either immediately before or immediately after an ',
        {
          text: 'rt',
          href: 'https://html.spec.whatwg.org/#the-rt-element',
          hashText: '#the-rt-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento ruby, inmediatamente antes o inmediatamente después de un elemento rt.',
    ],
    contentModel_es: ['Texto.'],
    params_es: [],
  },
  rt: {
    categories: ['None.'],
    contexts: ['As a child of a ruby element.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '38+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '38+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '14+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'ruby',
          href: 'https://html.spec.whatwg.org/#the-ruby-element',
          hashText: '#the-ruby-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como hijo de un elemento ruby.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  ruby: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['See prose.'],
    support: {
      Firefox: {
        WebHTMLElement: '38+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '38+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '14+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [['See prose.']],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Ver la prosa.'],
    params_es: [],
  },
  s: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  samp: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  script: {
    categories: [
      'Metadata content.',
      'Flow content.',
      'Phrasing content.',
      'Script-supporting element.',
    ],
    contexts: [
      'Where metadata content is expected.',
      'Where phrasing content is expected.',
      'Where script-supporting elements are expected.',
    ],
    contentModel: [
      'If there is no src\n   attribute, depends on the value of the type attribute, but must match\n   script content restrictions.',
      'If there is a src\n   attribute, the element must be either empty or contain only\n   script documentation that also matches script content restrictions.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Script-supporting element',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ' is expected.',
      ],
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
      [
        'Where ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' are expected.',
      ],
    ],
    rawContentModel: [
      [
        'If there is no ',
        {
          text: 'src',
          href: 'https://html.spec.whatwg.org/#attr-script-src',
          hashText: '#attr-script-src',
        },
        '\n   attribute, depends on the value of the ',
        {
          text: 'type',
          href: 'https://html.spec.whatwg.org/#attr-script-type',
          hashText: '#attr-script-type',
        },
        ' attribute, but must match\n   ',
        {
          text: 'script content restrictions',
          href: 'https://html.spec.whatwg.org/#restrictions-for-contents-of-script-elements',
          hashText: '#restrictions-for-contents-of-script-elements',
        },
        '.',
      ],
      [
        'If there ',
        'is',
        ' a ',
        {
          text: 'src',
          href: 'https://html.spec.whatwg.org/#attr-script-src',
          hashText: '#attr-script-src',
        },
        '\n   attribute, the element must be either empty or contain only\n   ',
        {
          text: 'script documentation',
          href: 'https://html.spec.whatwg.org/#inline-documentation-for-external-scripts',
          hashText: '#inline-documentation-for-external-scripts',
        },
        ' that also matches ',
        {
          text: 'script content restrictions',
          href: 'https://html.spec.whatwg.org/#restrictions-for-contents-of-script-elements',
          hashText: '#restrictions-for-contents-of-script-elements',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de metadatos.',
      'Contenido de flujo.',
      'Contenido de frases.',
      'Elemento de soporte de script.',
    ],
    contexts_es: [
      'Donde se espera contenido de metadatos.',
      'Donde se espera contenido de frases.',
      'Donde se esperan elementos de soporte de script.',
    ],
    contentModel_es: [
      'Si no hay atributo src, depende del valor del atributo type, pero debe cumplir las restricciones de contenido de script.',
      'Si hay un atributo src, el elemento debe estar vacío o contener solo documentación de script que también cumpla las restricciones de contenido de script.',
    ],
    params_es: [],
  },
  section: {
    categories: ['Flow content.', 'Sectioning content.', 'Palpable content.'],
    contexts: ['Where sectioning content is expected.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '5+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4.2+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '11.1+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de seccionado.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de seccionado.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  select: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Interactive content.',
      'Listed, labelable, submittable, resettable, and autocapitalize-inheriting form-associated element.',
      'Palpable content.',
    ],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Zero or more option, optgroup, and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '2+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '10.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ', ',
        {
          text: 'labelable',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        ', ',
        {
          text: 'submittable',
          href: 'https://html.spec.whatwg.org/#category-submit',
          hashText: '#category-submit',
        },
        ', ',
        {
          text: 'resettable',
          href: 'https://html.spec.whatwg.org/#category-reset',
          hashText: '#category-reset',
        },
        ', and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'option',
          href: 'https://html.spec.whatwg.org/#the-option-element',
          hashText: '#the-option-element',
        },
        ', ',
        {
          text: 'optgroup',
          href: 'https://html.spec.whatwg.org/#the-optgroup-element',
          hashText: '#the-optgroup-element',
        },
        ', and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido interactivo.',
      'Elemento asociado a formulario enumerado, etiquetable, submisible, restablecible y con herencia de autocapitalización.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Cero o más elementos option, optgroup y de soporte de script.'],
    params_es: [],
  },
  slot: {
    categories: ['Flow content.', 'Phrasing content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Transparent'],
    support: {
      Firefox: {
        WebHTMLElement: '63+',
        WebAPI: '63+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '53+',
        WebAPI: '53+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '40+',
        WebAPI: '40+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '63+',
        WebAPI: '63+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '53+',
        WebAPI: '53+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '53+',
        WebAPI: '53+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '6.0+',
        WebAPI: '6.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '41+',
        WebAPI: '41+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Transparente'],
    params_es: [],
  },
  small: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  source: {
    categories: ['None.'],
    contexts: [
      'As a child of a picture element, before the img element.',
      'As a child of a media element, before any flow content or\n   track elements.',
    ],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '3.5+',
        WebAPI: '3.5+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3.1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '9+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '2+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'picture',
          href: 'https://html.spec.whatwg.org/#the-picture-element',
          hashText: '#the-picture-element',
        },
        ' element, before the ',
        {
          text: 'img',
          href: 'https://html.spec.whatwg.org/#the-img-element',
          hashText: '#the-img-element',
        },
        ' element.',
      ],
      [
        'As a child of a ',
        {
          text: 'media element',
          href: 'https://html.spec.whatwg.org/#media-element',
          hashText: '#media-element',
        },
        ', before any ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' or\n   ',
        {
          text: 'track',
          href: 'https://html.spec.whatwg.org/#the-track-element',
          hashText: '#the-track-element',
        },
        ' elements.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento picture, antes del elemento img.',
      'Como hijo de un elemento de medios (media), antes de cualquier contenido de flujo o elemento track.',
    ],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  span: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '6+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '15+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '15+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '14+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  strong: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  style: {
    categories: ['Metadata content.'],
    contexts: [
      'Where metadata content is expected.',
      'In a noscript element that is a child of a head element.',
    ],
    contentModel: ['Text that gives a conformant style sheet.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '1+',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '3.5+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '3+',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '10.1+',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ' is expected.',
      ],
      [
        'In a ',
        {
          text: 'noscript',
          href: 'https://html.spec.whatwg.org/#the-noscript-element',
          hashText: '#the-noscript-element',
        },
        ' element that is a child of a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        ' that gives a ',
        {
          text: 'conformant style sheet',
          href: 'https://drafts.csswg.org/css-syntax/#conform-classes',
          hashText: '#conform-classes',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de metadatos.'],
    contexts_es: [
      'Donde se espera contenido de metadatos.',
      'En un elemento noscript que es hijo de un elemento head.',
    ],
    contentModel_es: ['Texto que constituye una hoja de estilo conforme.'],
    params_es: [],
  },
  sub: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  summary: {
    categories: ['None.'],
    contexts: ['As the first child of a details element.'],
    contentModel: ['Phrasing content, optionally intermixed with heading content.'],
    support: {
      Firefox: {
        WebHTMLElement: '49+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '6+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '49+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '14+',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As the ',
        {
          text: 'first child',
          href: 'https://dom.spec.whatwg.org/#concept-tree-first-child',
          hashText: '#concept-tree-first-child',
        },
        ' of a ',
        {
          text: 'details',
          href: 'https://html.spec.whatwg.org/#the-details-element',
          hashText: '#the-details-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ', optionally intermixed with ',
        {
          text: 'heading content',
          href: 'https://html.spec.whatwg.org/#heading-content-2',
          hashText: '#heading-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como primer hijo de un elemento details.'],
    contentModel_es: [
      'Contenido de frases, opcionalmente intercalado con contenido de encabezado.',
    ],
    params_es: [],
  },
  sup: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  table: {
    categories: ['Flow content.', 'Palpable content.'],
    contexts: ['Where flow content is expected.'],
    contentModel: [
      'In this order: optionally a caption element, followed by zero or more\n   colgroup elements, followed optionally by a thead element, followed by\n   either zero or more tbody elements or one or more tr elements, followed\n   optionally by a tfoot element, optionally intermixed with one or more\n   script-supporting elements.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '18+',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.0+',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'In this order: optionally a ',
        {
          text: 'caption',
          href: 'https://html.spec.whatwg.org/#the-caption-element',
          hashText: '#the-caption-element',
        },
        ' element, followed by zero or more\n   ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        ' elements, followed optionally by a ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        ' element, followed by\n   either zero or more ',
        {
          text: 'tbody',
          href: 'https://html.spec.whatwg.org/#the-tbody-element',
          hashText: '#the-tbody-element',
        },
        ' elements or one or more ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' elements, followed\n   optionally by a ',
        {
          text: 'tfoot',
          href: 'https://html.spec.whatwg.org/#the-tfoot-element',
          hashText: '#the-tfoot-element',
        },
        ' element, optionally intermixed with one or more\n   ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: [
      'En este orden: opcionalmente un elemento caption, seguido de cero o más elementos colgroup, seguido opcionalmente de un elemento thead, seguido de cero o más elementos tbody o uno o más elementos tr, seguido opcionalmente de un elemento tfoot, opcionalmente intercalado con uno o más elementos de soporte de script.',
    ],
    params_es: [],
  },
  tbody: {
    categories: ['None.'],
    contexts: [
      'As a child of a table element, after any\n   caption, colgroup, and\n   thead elements, but only if there are no\n   tr elements that are children of the\n   table element.',
    ],
    contentModel: ['Zero or more tr and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element, after any\n   ',
        {
          text: 'caption',
          href: 'https://html.spec.whatwg.org/#the-caption-element',
          hashText: '#the-caption-element',
        },
        ', ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        ', and\n   ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        ' elements, but only if there are no\n   ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' elements that are children of the\n   ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento table, después de cualquier elemento caption, colgroup y thead, pero solo si no hay elementos tr que sean hijos del elemento table.',
    ],
    contentModel_es: ['Cero o más elementos tr y de soporte de script.'],
    params_es: [],
  },
  td: {
    categories: ['Sectioning root.'],
    contexts: ['As a child of a tr element.'],
    contentModel: ['Flow content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Sectioning root',
          href: 'https://html.spec.whatwg.org/#sectioning-root',
          hashText: '#sectioning-root',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Raíz de seccionado.'],
    contexts_es: ['Como hijo de un elemento tr.'],
    contentModel_es: ['Contenido de flujo.'],
    params_es: [],
  },
  template: {
    categories: [
      'Metadata content.',
      'Flow content.',
      'Phrasing content.',
      'Script-supporting element.',
    ],
    contexts: [
      'Where metadata content is expected.',
      'Where phrasing content is expected.',
      'Where script-supporting elements are expected.',
      "As a child of a colgroup element that doesn't have a span attribute.",
    ],
    contentModel: ['Nothing (for clarification, see example).'],
    support: {
      Firefox: {
        WebHTMLElement: '22+',
        WebAPI: '22+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '8+',
        WebAPI: '8+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '26+',
        WebAPI: '26+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '15+',
        WebAPI: '15+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '13+',
        WebAPI: '13+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '22+',
        WebAPI: '22+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '8+',
        WebAPI: '8+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '26+',
        WebAPI: '26+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.5+',
        WebAPI: '1.5+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '?',
        WebAPI: 'Yes',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Script-supporting element',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        ' is expected.',
      ],
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
      [
        'Where ',
        {
          text: 'script-supporting elements',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' are expected.',
      ],
      [
        'As a child of a ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        " element that doesn't have a ",
        {
          text: 'span',
          href: 'https://html.spec.whatwg.org/#attr-colgroup-span',
          hashText: '#attr-colgroup-span',
        },
        ' attribute.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        ' (for clarification, ',
        {
          text: 'see example',
          href: 'https://html.spec.whatwg.org/#template-example',
          hashText: '#template-example',
        },
        ').',
      ],
    ],
    categories_es: [
      'Contenido de metadatos.',
      'Contenido de flujo.',
      'Contenido de frases.',
      'Elemento de soporte de script.',
    ],
    contexts_es: [
      'Donde se espera contenido de metadatos.',
      'Donde se espera contenido de frases.',
      'Donde se esperan elementos de soporte de script.',
      'Como hijo de un elemento colgroup que no tiene un atributo span.',
    ],
    contentModel_es: ['Nada (para aclaración, ver el ejemplo).'],
    params_es: [],
  },
  textarea: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Interactive content.',
      'Listed, labelable, submittable, resettable, and autocapitalize-inheriting form-associated element.',
      'Palpable content.',
    ],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Text.'],
    support: {
      Firefox: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '8+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '10.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Listed',
          href: 'https://html.spec.whatwg.org/#category-listed',
          hashText: '#category-listed',
        },
        ', ',
        {
          text: 'labelable',
          href: 'https://html.spec.whatwg.org/#category-label',
          hashText: '#category-label',
        },
        ', ',
        {
          text: 'submittable',
          href: 'https://html.spec.whatwg.org/#category-submit',
          hashText: '#category-submit',
        },
        ', ',
        {
          text: 'resettable',
          href: 'https://html.spec.whatwg.org/#category-reset',
          hashText: '#category-reset',
        },
        ', and ',
        {
          text: 'autocapitalize-inheriting',
          href: 'https://html.spec.whatwg.org/#category-autocapitalize',
          hashText: '#category-autocapitalize',
        },
        ' ',
        {
          text: 'form-associated element',
          href: 'https://html.spec.whatwg.org/#form-associated-element',
          hashText: '#form-associated-element',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        '.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido interactivo.',
      'Elemento asociado a formulario enumerado, etiquetable, submisible, restablecible y con herencia de autocapitalización.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Texto.'],
    params_es: [],
  },
  tfoot: {
    categories: ['None.'],
    contexts: [
      'As a child of a table element, after any\n   caption, colgroup, thead,\n   tbody, and tr elements, but only if there\n   are no other tfoot elements that are children of the\n   table element.',
    ],
    contentModel: ['Zero or more tr and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element, after any\n   ',
        {
          text: 'caption',
          href: 'https://html.spec.whatwg.org/#the-caption-element',
          hashText: '#the-caption-element',
        },
        ', ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        ', ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        ',\n   ',
        {
          text: 'tbody',
          href: 'https://html.spec.whatwg.org/#the-tbody-element',
          hashText: '#the-tbody-element',
        },
        ', and ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' elements, but only if there\n   are no other ',
        {
          text: 'tfoot',
          href: 'https://html.spec.whatwg.org/#the-tfoot-element',
          hashText: '#the-tfoot-element',
        },
        ' elements that are children of the\n   ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento table, después de cualquier elemento caption, colgroup, thead, tbody y tr, pero solo si no hay otros elementos tfoot que sean hijos del elemento table.',
    ],
    contentModel_es: ['Cero o más elementos tr y de soporte de script.'],
    params_es: [],
  },
  th: {
    categories: ['None.'],
    contexts: ['As a child of a tr element.'],
    contentModel: [
      'Flow content, but with no header, footer,\n   sectioning content, or heading content descendants.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ', but with no ',
        {
          text: 'header',
          href: 'https://html.spec.whatwg.org/#the-header-element',
          hashText: '#the-header-element',
        },
        ', ',
        {
          text: 'footer',
          href: 'https://html.spec.whatwg.org/#the-footer-element',
          hashText: '#the-footer-element',
        },
        ',\n   ',
        {
          text: 'sectioning content',
          href: 'https://html.spec.whatwg.org/#sectioning-content-2',
          hashText: '#sectioning-content-2',
        },
        ', or ',
        {
          text: 'heading content',
          href: 'https://html.spec.whatwg.org/#heading-content-2',
          hashText: '#heading-content-2',
        },
        ' descendants.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: ['Como hijo de un elemento tr.'],
    contentModel_es: [
      'Contenido de flujo, pero sin header, footer, contenido de seccionado ni contenido de encabezado descendientes.',
    ],
    params_es: [],
  },
  thead: {
    categories: ['None.'],
    contexts: [
      'As a child of a table element, after any\n   caption, and colgroup\n   elements and before any tbody, tfoot, and\n   tr elements, but only if there are no other\n   thead elements that are children of the\n   table element.',
    ],
    contentModel: ['Zero or more tr and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element, after any\n   ',
        {
          text: 'caption',
          href: 'https://html.spec.whatwg.org/#the-caption-element',
          hashText: '#the-caption-element',
        },
        ', and ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        '\n   elements and before any ',
        {
          text: 'tbody',
          href: 'https://html.spec.whatwg.org/#the-tbody-element',
          hashText: '#the-tbody-element',
        },
        ', ',
        {
          text: 'tfoot',
          href: 'https://html.spec.whatwg.org/#the-tfoot-element',
          hashText: '#the-tfoot-element',
        },
        ', and\n   ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' elements, but only if there are no other\n   ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        ' elements that are children of the\n   ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'tr',
          href: 'https://html.spec.whatwg.org/#the-tr-element',
          hashText: '#the-tr-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento table, después de cualquier elemento caption y colgroup y antes de cualquier elemento tbody, tfoot y tr, pero solo si no hay otros elementos thead que sean hijos del elemento table.',
    ],
    contentModel_es: ['Cero o más elementos tr y de soporte de script.'],
    params_es: [],
  },
  time: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: [
      'If the element has a datetime attribute: Phrasing content.',
      'Otherwise: Text, but must match requirements described in prose below.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '22+',
        WebAPI: '22+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '7+',
        WebAPI: '10+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '62+',
        WebAPI: '62+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '49+',
        WebAPI: '49+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '18',
        WebAPI: '14+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'No',
        WebAPI: 'No',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '22+',
        WebAPI: '22+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '4+',
        WebAPI: '10+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '62+',
        WebAPI: '62+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: '62+',
        WebAPI: '62+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '8.0+',
        WebAPI: '8.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '46+',
        WebAPI: '46+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'If the element has a ',
        {
          text: 'datetime',
          href: 'https://html.spec.whatwg.org/#attr-time-datetime',
          hashText: '#attr-time-datetime',
        },
        ' attribute: ',
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        'Otherwise: ',
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        ', but must match requirements described in prose below.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: [
      'Si el elemento tiene un atributo datetime: contenido de frases.',
      'De otro modo: texto, pero debe cumplir los requisitos descritos en la prosa a continuación.',
    ],
    params_es: [],
  },
  title: {
    categories: ['Metadata content.'],
    contexts: ['In a head element containing no other title elements.'],
    contentModel: ['Text that is not inter-element whitespace.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '1+',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '1+',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Metadata content',
          href: 'https://html.spec.whatwg.org/#metadata-content-2',
          hashText: '#metadata-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'In a ',
        {
          text: 'head',
          href: 'https://html.spec.whatwg.org/#the-head-element',
          hashText: '#the-head-element',
        },
        ' element containing no other ',
        {
          text: 'title',
          href: 'https://html.spec.whatwg.org/#the-title-element',
          hashText: '#the-title-element',
        },
        ' elements.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Text',
          href: 'https://html.spec.whatwg.org/#text-content',
          hashText: '#text-content',
        },
        ' that is not ',
        {
          text: 'inter-element whitespace',
          href: 'https://html.spec.whatwg.org/#inter-element-whitespace',
          hashText: '#inter-element-whitespace',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de metadatos.'],
    contexts_es: ['En un elemento head que no contenga otros elementos title.'],
    contentModel_es: ['Texto que no es espacio en blanco entre elementos.'],
    params_es: [],
  },
  tr: {
    categories: ['None.'],
    contexts: [
      'As a child of a thead element.',
      'As a child of a tbody element.',
      'As a child of a tfoot element.',
      'As a child of a table element, after any\n   caption, colgroup, and thead\n   elements, but only if there are no tbody elements that\n   are children of the table element.',
    ],
    contentModel: ['Zero or more td, th, and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        ' element.',
      ],
      [
        'As a child of a ',
        {
          text: 'tbody',
          href: 'https://html.spec.whatwg.org/#the-tbody-element',
          hashText: '#the-tbody-element',
        },
        ' element.',
      ],
      [
        'As a child of a ',
        {
          text: 'tfoot',
          href: 'https://html.spec.whatwg.org/#the-tfoot-element',
          hashText: '#the-tfoot-element',
        },
        ' element.',
      ],
      [
        'As a child of a ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element, after any\n   ',
        {
          text: 'caption',
          href: 'https://html.spec.whatwg.org/#the-caption-element',
          hashText: '#the-caption-element',
        },
        ', ',
        {
          text: 'colgroup',
          href: 'https://html.spec.whatwg.org/#the-colgroup-element',
          hashText: '#the-colgroup-element',
        },
        ', and ',
        {
          text: 'thead',
          href: 'https://html.spec.whatwg.org/#the-thead-element',
          hashText: '#the-thead-element',
        },
        '\n   elements, but only if there are no ',
        {
          text: 'tbody',
          href: 'https://html.spec.whatwg.org/#the-tbody-element',
          hashText: '#the-tbody-element',
        },
        ' elements that\n   are children of the ',
        {
          text: 'table',
          href: 'https://html.spec.whatwg.org/#the-table-element',
          hashText: '#the-table-element',
        },
        ' element.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'td',
          href: 'https://html.spec.whatwg.org/#the-td-element',
          hashText: '#the-td-element',
        },
        ', ',
        {
          text: 'th',
          href: 'https://html.spec.whatwg.org/#the-th-element',
          hashText: '#the-th-element',
        },
        ', and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento thead.',
      'Como hijo de un elemento tbody.',
      'Como hijo de un elemento tfoot.',
      'Como hijo de un elemento table, después de cualquier elemento caption, colgroup y thead, pero solo si no hay elementos tbody que sean hijos del elemento table.',
    ],
    contentModel_es: ['Cero o más elementos td, th y de soporte de script.'],
    params_es: [],
  },
  track: {
    categories: ['None.'],
    contexts: ['As a child of a media element, before any flow content.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '31+',
        WebAPI: '31+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '23+',
        WebAPI: '23+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '12.1+',
        WebAPI: '12+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '10+',
        WebAPI: '10+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '31+',
        WebAPI: '31+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '6+',
        WebAPI: '6+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: '25+',
        WebAPI: '25+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: '1.5+',
        WebAPI: '1.5+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '?',
        WebAPI: '12+',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [['None.']],
    rawContexts: [
      [
        'As a child of a ',
        {
          text: 'media element',
          href: 'https://html.spec.whatwg.org/#media-element',
          hashText: '#media-element',
        },
        ', before any ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Ninguno.'],
    contexts_es: [
      'Como hijo de un elemento de medios (media), antes de cualquier contenido de flujo.',
    ],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
  u: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  ul: {
    categories: [
      'Flow content.',
      "If the element's children include at least one li element: Palpable content.",
    ],
    contexts: ['Where flow content is expected.'],
    contentModel: ['Zero or more li and script-supporting elements.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '3+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '5.5+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '1+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '12.1+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'li',
        href: 'https://html.spec.whatwg.org/#the-li-element',
        hashText: '#the-li-element',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        "If the element's children include at least one ",
        {
          text: 'li',
          href: 'https://html.spec.whatwg.org/#the-li-element',
          hashText: '#the-li-element',
        },
        ' element: ',
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'Zero or more ',
        {
          text: 'li',
          href: 'https://html.spec.whatwg.org/#the-li-element',
          hashText: '#the-li-element',
        },
        ' and ',
        {
          text: 'script-supporting',
          href: 'https://html.spec.whatwg.org/#script-supporting-elements-2',
          hashText: '#script-supporting-elements-2',
        },
        ' elements.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Si los hijos del elemento incluyen al menos un elemento li: contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido de flujo.'],
    contentModel_es: ['Cero o más elementos li y de soporte de script.'],
    params_es: ['li'],
  },
  var: {
    categories: ['Flow content.', 'Phrasing content.', 'Palpable content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Phrasing content.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.', 'Contenido palpable.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Contenido de frases.'],
    params_es: [],
  },
  video: {
    categories: [
      'Flow content.',
      'Phrasing content.',
      'Embedded content.',
      'If the element has a controls attribute: Interactive content.',
      'Palpable content.',
    ],
    contexts: ['Where embedded content is expected.'],
    contentModel: [
      'If the element has a src attribute:\n zero or more track elements, then\n transparent, but with no media element descendants.',
      'If the element does not have a src attribute: zero or more source elements, then\n zero or more track elements, then\n transparent, but with no media element descendants.',
    ],
    support: {
      Firefox: {
        WebHTMLElement: '3.5+',
        WebAPI: '3.5+',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '3.1+',
        WebAPI: '3.1+',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '3+',
        WebAPI: '1+',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '10.5+',
        WebAPI: '10.5+',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '79+',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: '12+',
        WebAPI: '12+',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '9+',
        WebAPI: '9+',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '4+',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: 'Yes',
        WebAPI: '2+',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '18+',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '37+',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '1.0+',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '11+',
        caniuse: '--',
      },
    },
    params: [
      {
        text: 'controls',
        href: 'https://html.spec.whatwg.org/#attr-media-controls',
        hashText: '#attr-media-controls',
      },
    ],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        '.',
      ],
      [
        'If the element has a ',
        {
          text: 'controls',
          href: 'https://html.spec.whatwg.org/#attr-media-controls',
          hashText: '#attr-media-controls',
        },
        ' attribute: ',
        {
          text: 'Interactive content',
          href: 'https://html.spec.whatwg.org/#interactive-content-2',
          hashText: '#interactive-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Palpable content',
          href: 'https://html.spec.whatwg.org/#palpable-content-2',
          hashText: '#palpable-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'embedded content',
          href: 'https://html.spec.whatwg.org/#embedded-content-category',
          hashText: '#embedded-content-category',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        'If the element has a ',
        {
          text: 'src',
          href: 'https://html.spec.whatwg.org/#attr-media-src',
          hashText: '#attr-media-src',
        },
        ' attribute:\n zero or more ',
        {
          text: 'track',
          href: 'https://html.spec.whatwg.org/#the-track-element',
          hashText: '#the-track-element',
        },
        ' elements, then\n ',
        {
          text: 'transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        ', but with no ',
        {
          text: 'media element',
          href: 'https://html.spec.whatwg.org/#media-element',
          hashText: '#media-element',
        },
        ' descendants.',
      ],
      [
        'If the element does not have a ',
        {
          text: 'src',
          href: 'https://html.spec.whatwg.org/#attr-media-src',
          hashText: '#attr-media-src',
        },
        ' attribute: zero or more ',
        {
          text: 'source',
          href: 'https://html.spec.whatwg.org/#the-source-element',
          hashText: '#the-source-element',
        },
        ' elements, then\n zero or more ',
        {
          text: 'track',
          href: 'https://html.spec.whatwg.org/#the-track-element',
          hashText: '#the-track-element',
        },
        ' elements, then\n ',
        {
          text: 'transparent',
          href: 'https://html.spec.whatwg.org/#transparent',
          hashText: '#transparent',
        },
        ', but with no ',
        {
          text: 'media element',
          href: 'https://html.spec.whatwg.org/#media-element',
          hashText: '#media-element',
        },
        ' descendants.',
      ],
    ],
    categories_es: [
      'Contenido de flujo.',
      'Contenido de frases.',
      'Contenido incrustado.',
      'Si el elemento tiene un atributo controls: contenido interactivo.',
      'Contenido palpable.',
    ],
    contexts_es: ['Donde se espera contenido incrustado.'],
    contentModel_es: [
      'Si el elemento tiene un atributo src: cero o más elementos track, luego transparente, pero sin elementos de medios (media) descendientes.',
      'Si el elemento no tiene un atributo src: cero o más elementos source, luego cero o más elementos track, luego transparente, pero sin elementos de medios (media) descendientes.',
    ],
    params_es: ['controls'],
  },
  wbr: {
    categories: ['Flow content.', 'Phrasing content.'],
    contexts: ['Where phrasing content is expected.'],
    contentModel: ['Nothing.'],
    support: {
      Firefox: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Safari: {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      Chrome: {
        WebHTMLElement: '1+',
        WebAPI: '--',
        caniuse: '--',
      },
      Opera: {
        WebHTMLElement: '11.6+',
        WebAPI: '--',
        caniuse: '--',
      },
      Edge: {
        WebHTMLElement: '79+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Edge (Legacy)': {
        WebHTMLElement: 'No',
        WebAPI: '--',
        caniuse: '--',
      },
      'Internet Explorer': {
        WebHTMLElement: '5.5–7',
        WebAPI: '--',
        caniuse: '--',
      },
      'Firefox Android': {
        WebHTMLElement: '4+',
        WebAPI: '--',
        caniuse: '--',
      },
      'Safari iOS': {
        WebHTMLElement: '?',
        WebAPI: '--',
        caniuse: '--',
      },
      'Chrome Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'WebView Android': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Samsung Internet': {
        WebHTMLElement: 'Yes',
        WebAPI: '--',
        caniuse: '--',
      },
      'Opera Android': {
        WebHTMLElement: '?',
        WebAPI: '--',
        caniuse: '--',
      },
    },
    params: [],
    rawCategories: [
      [
        {
          text: 'Flow content',
          href: 'https://html.spec.whatwg.org/#flow-content-2',
          hashText: '#flow-content-2',
        },
        '.',
      ],
      [
        {
          text: 'Phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        '.',
      ],
    ],
    rawContexts: [
      [
        'Where ',
        {
          text: 'phrasing content',
          href: 'https://html.spec.whatwg.org/#phrasing-content-2',
          hashText: '#phrasing-content-2',
        },
        ' is expected.',
      ],
    ],
    rawContentModel: [
      [
        {
          text: 'Nothing',
          href: 'https://html.spec.whatwg.org/#concept-content-nothing',
          hashText: '#concept-content-nothing',
        },
        '.',
      ],
    ],
    categories_es: ['Contenido de flujo.', 'Contenido de frases.'],
    contexts_es: ['Donde se espera contenido de frases.'],
    contentModel_es: ['Nada.'],
    params_es: [],
  },
};

export function getElementReference(tag: string): ElementReference | undefined {
  return HTML_ELEMENT_REFERENCES[tag.toLowerCase()];
}
