<script lang="ts">
  import PageDetailPanel from '$lib/features/page-detail/PageDetailPanel.svelte';

  let pageId = $state('mock-page');
</script>

<svelte:head>
  <script>
    (() => {
      const sample = {
        id: 'r1',
        config_id: 'c1',
        project_id: 'p1',
        url: 'https://example.com/products/foo',
        status_code: 200,
        title: 'Producto Foo | Ejemplo',
        meta_description: 'Descripcion del producto foo.',
        h1: 'Producto Foo',
        canonical: 'https://example.com/products/foo',
        size_bytes: 15234,
        load_time_ms: 431,
        is_indexable: true,
        depth: 1,
        parent_url: 'https://example.com/',
        crawl_timestamp: '2026-08-02T12:00:00Z',
        html_lang: 'es',
        hreflang_json: JSON.stringify([]),
        semantic_issues_json: JSON.stringify([
          {
            issue_type: 'multiple_h1',
            severity: 'warning',
            message: 'Multiple H1 (2)',
            element: 'h1',
            xpath: '/html/body/div[1]',
          },
          {
            issue_type: 'img_no_alt',
            severity: 'error',
            message: '3 images without alt',
            element: 'img',
            xpath: '/html/body/img[1]',
          },
        ]),
        html_body:
          '<!DOCTYPE html><html lang="es"><head><style>body{font-family:sans-serif;margin:0;padding:24px}h1{color:#222}</style><script>window.bad=1<\/script></head><body><div style="height:120px;background:#eee"><h1 style="height:80px">Titulo de ejemplo</h1></div><img src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" width="200" height="100" alt=""><div style="height:1800px"></div><h2>Fin</h2></body></html>',
      };

      window.__TAURI_INTERNALS__ = {
        metadata: { __windows: [] },
        transformCallback: () => {},
        invoke: async (cmd, args) => {
          if (cmd === 'get_page_detail') return [sample, []];
          if (cmd === 'get_page_html') return sample.html_body;
          if (cmd === 'inline_assets') return args.html;
          if (cmd === 'capture_page_screenshot') return null;
          throw new Error('unmocked invoke: ' + cmd);
        },
      };
    })();
  </script>
</svelte:head>

<PageDetailPanel bind:pageId onClose={() => { pageId = ''; }} />
