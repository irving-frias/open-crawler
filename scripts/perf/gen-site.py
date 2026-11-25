#!/usr/bin/env python3
"""Genera un sitio de prueba con N páginas interconectadas para medir
rendimiento (T10). Uso:

    python3 gen-site.py <salida> <n_paginas>

Cada página tiene título, h1, meta description, párrafos, enlaces internos y
algunos problemas reales (sin título, sin descripción, 404) para que el crawl
produzca issues. Determinista (semilla fija) para que las mediciones sean
reproducibles. El index enlaza a lotes de 100 páginas; cada página enlaza a
las 5 siguientes, así el crawler las descubre todas por enlaces.
"""

import random
import sys
from pathlib import Path

WORDS = (
    "auditoria seo rastreo enlaces contenido pagina titulo descripcion "
    "encabezado imagen video tabla formulario navegacion accesibilidad "
    "rendimiento velocidad movil escritorio palabras clave busqueda google "
    "analisis tecnico estructura datos schema jsonld canonical robots sitemap"
).split()


def paragraph(rng: random.Random, n_words: int) -> str:
    return " ".join(rng.choice(WORDS) for _ in range(n_words)) + "."


def page_html(i: int, n: int, rng: random.Random, port: int) -> str:
    # ~1% de páginas sin título, ~2% sin descripción, ~1% con título largo
    no_title = i % 97 == 0
    no_desc = i % 53 == 0
    long_title = i % 71 == 0
    title = "" if no_title else (
        f"Página de prueba número {i} sobre auditoría SEO y contenido"
        + (" con un título deliberadamente demasiado largo para fallar el "
           "check de longitud máxima de título" if long_title else "")
    )
    desc = "" if no_desc else paragraph(rng, 22)
    links = []
    for j in range(i + 1, min(i + 6, n)):
        links.append(f'<a href="/p{j:06d}.html">Página {j}</a>')
    # ~0.5% de enlaces rotos
    if i % 211 == 0:
        links.append('<a href="/no-existe.html">Enlace roto</a>')
    links_html = " ".join(f"<li>{a}</li>" for a in links)
    return f"""<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="utf-8">
<title>{title}</title>
<meta name="description" content="{desc}">
<link rel="canonical" href="http://127.0.0.1:{port}/p{i:06d}.html">
</head>
<body>
<main>
<h1>Página {i} — auditoría SEO</h1>
<p>{paragraph(rng, 60)}</p>
<p>{paragraph(rng, 45)}</p>
<h2>Sección de contenido {i}</h2>
<p>{paragraph(rng, 50)}</p>
<nav><ul>{links_html}</ul></nav>
</main>
</body>
</html>
"""


def main() -> None:
    out_dir = Path(sys.argv[1])
    n = int(sys.argv[2])
    port = int(sys.argv[3]) if len(sys.argv) > 3 else 8899
    rng = random.Random(42)
    out_dir.mkdir(parents=True, exist_ok=True)
    # index con enlaces a lotes de 100 páginas
    batch_links = "".join(
        f'<li><a href="/p{b:06d}.html">Lote {b}</a></li>'
        for b in range(0, n, 100)
    )
    (out_dir / "index.html").write_text(
        "<!DOCTYPE html>\n<html lang=\"es\">\n<head><meta charset=\"utf-8\">"
        "<title>Sitio de prueba</title></head>\n<body><main><h1>Sitio de prueba"
        f"</h1><ul>{batch_links}</ul></main></body></html>\n",
        encoding="utf-8",
    )
    for i in range(n):
        (out_dir / f"p{i:06d}.html").write_text(
            page_html(i, n, rng, port), encoding="utf-8"
        )
    print(f"generadas {n} páginas en {out_dir}")


if __name__ == "__main__":
    main()
