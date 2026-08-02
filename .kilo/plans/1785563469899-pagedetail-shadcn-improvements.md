# Plan: PageDetailPanel shadcn/ui Improvements

## Goal
Upgrade the PageDetailPanel component to leverage more shadcn/ui components for better visual hierarchy, scrolling, and UX patterns.

## Context
- Current PageDetailPanel uses: Tabs, Card, Button, Skeleton
- Available shadcn/ui components: alert-dialog, badge, button, card, checkbox, dialog, input, label, popover, progress, scroll-area, select, separator, skeleton, sonner, tabs
- The panel has 3 tabs: Overview, Links (and could add Preview)
- Data comes from `get_page_detail` command returning `[CrawlResult, PageLink[]]`
- `get_page_html` and `capture_page_screenshot` commands exist for preview/screenshot

## Changes

### 1. Add ScrollArea to issue list and links tab
- Wrap the issue list in `ScrollArea` with a fixed max height
- Wrap the links table in `ScrollArea` for overflow handling
- Import from `$lib/components/ui/scroll-area/index.js`

### 2. Add Badge for status codes and severity
- Replace custom status-code spans with `<Badge>` variant styling
- Use `variant="destructive"` for 5xx, `variant="warning"` for 4xx, `variant="default"` for 2xx/3xx
- Use Badge for severity labels (error/warning/info) instead of custom CSS classes
- Import from `$lib/components/ui/badge/index.js`

### 3. Add Separator between card sections
- Add `<Separator>` between SEO meta fields and crawl info card
- Add `<Separator>` between issue cards
- Import from `$lib/components/ui/separator/index.js`

### 4. Add Tooltip for truncated text and copy buttons
- Wrap the URL in the header with a `<Tooltip>` showing full URL
- Wrap copy buttons with `<Tooltip>` showing "Copy" / "Copied"
- Import from `$lib/components/ui/tooltip/index.js` (need to create if not exists)

### 5. Add Accordion for collapsible overview sections
- Replace stacked Cards with an `<Accordion>` for SEO Meta, Crawl Info, Hreflang, Semantic Issues
- Each section collapsible, default open for SEO Meta and Issues
- Import from `$lib/components/ui/accordion/index.js` (need to create if not exists)

### 6. Add Table for links tab
- Replace custom grid links list with `<Table>` component
- Columns: Type (Badge), URL (truncated with Tooltip), Anchor Text
- Import from `$lib/components/ui/table/index.js` (need to create if not exists)

### 7. Add Preview tab with HTML preview and screenshot
- Add "Preview" tab alongside Overview and Links
- Use `get_page_html` command to fetch inline HTML
- Display HTML in an `<iframe>` with sandboxed srcdoc
- Show screenshot if available (from `capture_page_screenshot` or stored data)
- Use `Skeleton` loading state for preview
- Import from existing components

### 8. Add Alert for error state
- Replace inline error `<div>` with `<Alert variant="destructive">`
- Import from `$lib/components/ui/alert/index.js`

### 9. Add Progress bar for page load time
- Show load time as a `<Progress>` bar (normalized to max 10s)
- Color: green (<1s), yellow (1-3s), red (>3s)
- Import from `$lib/components/ui/progress/index.js`

### 10. Add Popover for issue actions
- Replace inline copy button with a `<Popover>` dropdown on each issue
- Actions: Copy XPath, Copy Element, Copy Issue Message
- Import from `$lib/components/ui/popover/index.js`

## Files to Modify
- `src/lib/components/PageDetailPanel.svelte` — main component rewrite
- `src/lib/components/ui/scroll-area/` — create if missing
- `src/lib/components/ui/accordion/` — create if missing
- `src/lib/components/ui/table/` — create if missing
- `src/lib/components/ui/tooltip/` — create if missing
- `src/lib/components/ui/alert/` — create if missing (may already exist)

## Validation
- `pnpm check` must pass with 0 errors
- Component must render correctly with all tabs
- ScrollArea must enable scrolling for long issue lists
- Badge must display correct variants for status codes and severities
- Preview tab must show HTML and screenshot when available
- Accordion must collapse/expand sections correctly
- Table must display links with proper columns

## Risks
- Creating new shadcn/ui component wrappers (scroll-area, accordion, table, tooltip, alert) requires following existing patterns from other components
- The Preview tab requires fetching HTML via `invoke('get_page_html')` which adds async complexity
- Screenshot display requires base64 data URL handling
- Accordion + ScrollArea nesting may have layout issues
