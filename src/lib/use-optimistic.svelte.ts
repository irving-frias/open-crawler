export type OptimisticAction<T> = { id: string; type: string } & Partial<T>;

/**
 * Svelte 5 equivalent of Next.js `useOptimistic`.
 *
 * Returns a derived list computed from a "server truth" base plus a stack of
 * pending (optimistic) actions. Call `add` to apply an optimistic change
 * instantly; call `settle` once the real async operation finishes — on
 * success commit the new server truth to `base`, on failure the derived list
 * automatically reverts because the pending action is dropped.
 */
export function useOptimistic<T extends { id: string }, A extends OptimisticAction<T>>(
  getBase: () => T[],
  apply: (base: T[], action: A) => T[]
) {
  let pending = $state<A[]>([]);
  const optimistic = $derived(pending.reduce((list, action) => apply(list, action), getBase()));
  const isPending = $derived(pending.length > 0);

  function add(action: A) {
    pending = [...pending, action];
  }

  function settle(action: A) {
    pending = pending.filter((a) => a !== action);
  }

  return {
    get optimistic() {
      return optimistic;
    },
    get pending() {
      return pending;
    },
    get isPending() {
      return isPending;
    },
    add,
    settle,
  };
}
