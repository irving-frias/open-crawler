<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import type { PageLink } from '$lib/api/types';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import {
    Tooltip,
    TooltipTrigger,
    TooltipContent,
    TooltipProvider,
  } from '$lib/components/ui/tooltip/index.js';
  import {
    Table,
    TableHeader,
    TableBody,
    TableRow,
    TableHead,
    TableCell,
  } from '$lib/components/ui/table/index.js';

  let {
    links,
  }: {
    links: PageLink[];
  } = $props();

  function truncateUrl(url: string, maxLen: number = 80): string {
    if (url.length <= maxLen) return url;
    return url.slice(0, maxLen - 3) + '...';
  }
</script>

<div class="fullpage-body">
  {#if links.length > 0}
    <ScrollArea class="h-full">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{m['detail.links_type']()}</TableHead>
            <TableHead>{m['detail.links_url']()}</TableHead>
            <TableHead>{m['detail.links_anchor']()}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {#each links as link (link.to_url)}
            <TableRow>
              <TableCell>
                <Badge variant="secondary" class="text-xs">{link.link_type}</Badge>
              </TableCell>
              <TableCell>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger>
                      <a href={link.to_url} target="_blank" class="link-url">
                        {truncateUrl(link.to_url)}
                      </a>
                    </TooltipTrigger>
                    <TooltipContent>
                      {link.to_url}
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </TableCell>
              <TableCell>
                <span class="link-anchor">{link.anchor_text || '-'}</span>
              </TableCell>
            </TableRow>
          {/each}
        </TableBody>
      </Table>
    </ScrollArea>
  {:else}
    <div class="empty-tab">{m['detail.no_links']()}</div>
  {/if}
</div>

<style>
  .fullpage-body {
    height: 100%;
    overflow-y: auto;
    padding: 24px;
    padding-bottom: calc(24px + env(safe-area-inset-bottom, 0px));
    overscroll-behavior: contain;
  }

  .link-url {
    color: var(--text-secondary);
    text-decoration: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .link-url:hover {
    color: var(--accent);
  }
  .link-url:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 2px;
  }

  .link-anchor {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-tab {
    text-align: center;
    color: var(--text-muted);
    padding: 60px 20px;
    font-size: 0.9rem;
  }

  @media (max-width: 767px) {
    .fullpage-body {
      padding: 16px;
      padding-bottom: calc(16px + env(safe-area-inset-bottom, 0px));
    }
  }
</style>
