import fs from 'fs/promises';
import path from 'path';

const DATA_DIR = path.resolve(import.meta.dirname, '..', 'src-tauri', 'data');
const INDEX_FILE = path.join(DATA_DIR, 'caninclude-index.json');
const OUTPUT_FILE = path.resolve(import.meta.dirname, '..', 'src-tauri', 'src', 'nesting_table.rs');

const STATUS_MAP = { can: 0, cant: 1, doubt: 2, error: 2 };

async function main() {
  const index = JSON.parse(await fs.readFile(INDEX_FILE, 'utf-8'));

  const allParents = [];
  const allData = {};

  for (const file of index.files) {
    const chunk = JSON.parse(await fs.readFile(path.join(DATA_DIR, file.filename), 'utf-8'));
    for (const parent of file.parents) {
      allParents.push(parent);
      allData[parent] = chunk[parent];
    }
  }

  allParents.sort();

  const tagSet = new Set();
  for (const parent of allParents) {
    tagSet.add(parent);
    for (const child of Object.keys(allData[parent])) {
      tagSet.add(child);
    }
  }
  const tags = Array.from(tagSet).sort();
  const numParents = allParents.length;
  const numTags = tags.length;

  const matrix = [];
  for (const parent of allParents) {
    const row = [];
    for (const child of tags) {
      const entry = allData[parent]?.[child];
      const status = entry ? (STATUS_MAP[entry.status] ?? 2) : 2;
      row.push(status);
    }
    matrix.push(row);
  }

  let rs = `// Auto-generated from caninclude JSON — DO NOT EDIT
// Generated: ${new Date().toISOString()}
// Source: ${numParents} parents × ${numTags} children = ${numParents * numTags} combinations
// Regenerate: node tools/generate_nesting_table.js

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NestingStatus {
    Can = 0,
    Cant = 1,
    Doubt = 2,
}

impl NestingStatus {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Can,
            1 => Self::Cant,
            _ => Self::Doubt,
        }
    }
}

pub static NESTING_TAGS: &[&str] = &[
`;

  for (const tag of tags) {
    rs += `    "${tag}",\n`;
  }

  rs += `];\n\npub static NESTING_MATRIX: &[[u8; ${numTags}]; ${numParents}] = &[\n`;

  for (let i = 0; i < numParents; i++) {
    rs += `    // ${allParents[i]}\n    [${matrix[i].join(', ')}],\n`;
  }

  rs += `];

fn tag_index(tag: &str) -> Option<usize> {
    NESTING_TAGS.binary_search(&tag).ok()
}

pub fn can_include(parent: &str, child: &str) -> Option<NestingStatus> {
    let pi = tag_index(parent)?;
    let ci = tag_index(child)?;
    Some(NestingStatus::from_u8(NESTING_MATRIX[pi][ci]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_combinations() {
        assert_eq!(can_include("div", "span"), Some(NestingStatus::Cant));
        assert_eq!(can_include("div", "p"), Some(NestingStatus::Can));
        assert_eq!(can_include("ul", "li"), Some(NestingStatus::Can));
        assert_eq!(can_include("table", "tr"), Some(NestingStatus::Can));
        assert_eq!(can_include("tr", "td"), Some(NestingStatus::Can));
        assert_eq!(can_include("select", "option"), Some(NestingStatus::Can));
    }

    #[test]
    fn test_invalid_combinations() {
        assert_eq!(can_include("span", "div"), Some(NestingStatus::Cant));
        assert_eq!(can_include("a", "div"), Some(NestingStatus::Cant));
        assert_eq!(can_include("code", "p"), Some(NestingStatus::Cant));
        assert_eq!(can_include("button", "table"), Some(NestingStatus::Cant));
        assert_eq!(can_include("label", "section"), Some(NestingStatus::Cant));
    }

    #[test]
    fn test_unknown_tag_returns_none() {
        assert_eq!(can_include("div", "unknown-tag"), None);
        assert_eq!(can_include("unknown-tag", "div"), None);
    }

    #[test]
    fn test_matrix_dimensions() {
        assert_eq!(NESTING_TAGS.len(), ${numTags});
        assert_eq!(NESTING_MATRIX.len(), ${numParents});
        for row in NESTING_MATRIX {
            assert_eq!(row.len(), ${numTags});
        }
    }
}
`;

  await fs.writeFile(OUTPUT_FILE, rs);
  console.log(`Generated: ${OUTPUT_FILE}`);
  console.log(`  ${numParents} parents × ${numTags} tags = ${numParents * numTags} entries`);
  console.log(`  File size: ${Math.round(rs.length / 1024)} KB`);
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
