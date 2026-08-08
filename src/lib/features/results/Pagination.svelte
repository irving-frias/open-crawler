<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-svelte';
  import { Button } from '$lib/components/ui/button/index.js';

  let {
    currentPage,
    totalPages,
    total,
    pageSize,
    onGoToPage,
  }: {
    currentPage: number;
    totalPages: number;
    total: number;
    pageSize: number;
    onGoToPage: (page: number) => void;
  } = $props();

  function getPageNumbers(): (number | '...')[] {
    const pages: (number | '...')[] = [];
    const total = totalPages;
    const current = currentPage;

    if (total <= 7) {
      for (let i = 1; i <= total; i++) pages.push(i);
    } else {
      pages.push(1);
      if (current > 3) pages.push('...');
      const start = Math.max(2, current - 1);
      const end = Math.min(total - 1, current + 1);
      for (let i = start; i <= end; i++) pages.push(i);
      if (current < total - 2) pages.push('...');
      pages.push(total);
    }

    return pages;
  }
</script>

<div class="pagination">
  <span class="pagination-info">
    {m['results.showing']({
      from: ((currentPage - 1) * pageSize + 1).toString(),
      to: Math.min(currentPage * pageSize, total).toString(),
      total: total.toLocaleString(),
    })}
  </span>
  <div class="pagination-controls">
    <Button
      variant="outline"
      size="icon"
      class="btn-edge size-9"
      onclick={() => onGoToPage(1)}
      disabled={currentPage === 1}
      aria-label="First page"
    >
      <ChevronsLeft class="size-4" />
    </Button>
    <Button
      variant="outline"
      size="icon"
      class="size-9"
      onclick={() => onGoToPage(currentPage - 1)}
      disabled={currentPage === 1}
      aria-label="Previous page"
    >
      <ChevronLeft class="size-4" />
    </Button>
    {#each getPageNumbers() as pageNum (pageNum)}
      {#if pageNum === '...'}
        <span class="page-ellipsis">&hellip;</span>
      {:else}
        <Button
          variant={pageNum === currentPage ? 'default' : 'outline'}
          size="icon"
          class="btn-page size-9"
          onclick={() => onGoToPage(pageNum)}
          aria-current={pageNum === currentPage ? 'page' : undefined}
        >
          {pageNum}
        </Button>
      {/if}
    {/each}
    <Button
      variant="outline"
      size="icon"
      class="size-9"
      onclick={() => onGoToPage(currentPage + 1)}
      disabled={currentPage === totalPages}
      aria-label="Next page"
    >
      <ChevronRight class="size-4" />
    </Button>
    <Button
      variant="outline"
      size="icon"
      class="btn-edge size-9"
      onclick={() => onGoToPage(totalPages)}
      disabled={currentPage === totalPages}
      aria-label="Last page"
    >
      <ChevronsRight class="size-4" />
    </Button>
  </div>
  <span class="pagination-page">
    {m['results.page_of']({ current: currentPage.toString(), total: totalPages.toString() })}
  </span>
</div>

<style>
  .pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  .pagination-info {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .pagination-controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .page-ellipsis {
    padding: 0 4px;
    color: var(--text-muted);
  }

  .pagination-page {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  @media (max-width: 767px) {
    .pagination {
      flex-direction: column;
      gap: var(--space-md);
      align-items: center;
    }

    .pagination-controls {
      flex-wrap: wrap;
      justify-content: center;
    }

    :global(.btn-edge) {
      display: none;
    }
  }

  @media (min-width: 768px) {
    .pagination {
      flex-direction: row;
      justify-content: space-between;
    }

    :global(.btn-edge) {
      display: inline-flex;
    }
  }
</style>
