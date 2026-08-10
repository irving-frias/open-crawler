export type SiteMapNavAction = 'graph' | 'tree';

type SiteMapNavState = {
  seq: number;
  action: SiteMapNavAction | null;
  projectId: string | null;
  url: string | null;
};

// Cross-navigation between the site tree and the site graph. A component
// publishes a focus request and the results view switches to the matching tab,
// where the target component expands/scrolls to (tree) or centers/selects
// (graph) the requested page.
export const siteMapNav = $state<SiteMapNavState>({
  seq: 0,
  action: null,
  projectId: null,
  url: null,
});

export function requestFocusInGraph(projectId: string, url: string): void {
  siteMapNav.seq += 1;
  siteMapNav.action = 'graph';
  siteMapNav.projectId = projectId;
  siteMapNav.url = url;
}

export function requestFocusInTree(projectId: string, url: string): void {
  siteMapNav.seq += 1;
  siteMapNav.action = 'tree';
  siteMapNav.projectId = projectId;
  siteMapNav.url = url;
}
