import fs from 'fs/promises';
import path from 'path';

const DATA_DIR = path.resolve(import.meta.dirname, '..', 'src-tauri', 'data');
const INDEX_FILE = path.join(DATA_DIR, 'caninclude-index.json');
const DICT_FILE = path.resolve(import.meta.dirname, 'spec-es.json');
const OUTPUT_FILE = path.resolve(import.meta.dirname, '..', 'src', 'lib', 'data', 'html-elements.ts');

const normalize = (s) => s.replace(/\s+/g, ' ').trim();

async function main() {
  const index = JSON.parse(await fs.readFile(INDEX_FILE, 'utf-8'));
  const dict = JSON.parse(await fs.readFile(DICT_FILE, 'utf-8'));
  const dictBySection = {};
  for (const section of ['categories', 'contexts', 'contentModel', 'params']) {
    dictBySection[section] = new Map();
    for (const [en, es] of Object.entries(dict[section] ?? {})) {
      dictBySection[section].set(normalize(en), es);
    }
  }

  const missing = new Set();
  const translate = (section, text) => {
    const es = dictBySection[section].get(normalize(text));
    if (es === undefined) missing.add(`${section}: ${text}`);
    return es ?? text;
  };

  const byTag = new Map();

  for (const file of index.files) {
    const chunk = JSON.parse(await fs.readFile(path.join(DATA_DIR, file.filename), 'utf-8'));
    for (const parent of file.parents) {
      for (const childTag of Object.keys(chunk[parent])) {
        const child = chunk[parent][childTag]?.child;
        if (!child?.tag) continue;
        if (!byTag.has(child.tag)) {
          const { tag: _tag, ...reference } = child;
          reference.categories_es = reference.categories.map((s) => translate('categories', s));
          reference.contexts_es = reference.contexts.map((s) => translate('contexts', s));
          reference.contentModel_es = reference.contentModel.map((s) => translate('contentModel', s));
          reference.params_es = reference.params.map((s) => translate('params', typeof s === 'string' ? s : s.text));
          byTag.set(child.tag, reference);
        }
      }
    }
  }

  const tags = Array.from(byTag.keys()).sort();

  const lines = tags.map((tag) => {
    const value = JSON.stringify(byTag.get(tag), null, 2)
      .split('\n')
      .map((line, i) => (i === 0 ? `  ${JSON.stringify(tag)}: ${line}` : `  ${line}`))
      .join('\n');
    return `${value},`;
  });

  let ts = `// Auto-generated from caninclude JSON — DO NOT EDIT
// Generated: ${new Date().toISOString()}
// Source: ${tags.length} elements
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
${lines.join('\n')}
};

export function getElementReference(tag: string): ElementReference | undefined {
  return HTML_ELEMENT_REFERENCES[tag.toLowerCase()];
}
`;

  await fs.writeFile(OUTPUT_FILE, ts);
  console.log(`Generated: ${OUTPUT_FILE}`);
  console.log(`  ${tags.length} elements`);
  console.log(`  File size: ${Math.round(ts.length / 1024)} KB`);
  if (missing.size) {
    console.warn(`WARNING: ${missing.size} untranslated string(s):`);
    for (const s of missing) console.warn(`  - ${s}`);
  }
}

main().catch((err) => {
  console.error('Error:', err);
  process.exit(1);
});
