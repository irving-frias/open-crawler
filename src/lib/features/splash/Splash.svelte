<script lang="ts">
  import { m } from '$lib/paraglide/messages.js';
  import { Radar } from 'lucide-svelte';

  let { visible = true }: { visible?: boolean } = $props();
</script>

<div class="splash" class:exit={!visible} aria-hidden={!visible}>
  <div class="splash-center">
    <div class="splash-ring" aria-hidden="true"></div>
    <div class="splash-badge" aria-hidden="true">
      <Radar class="splash-icon" />
    </div>
    <h1 class="splash-title">{m['app.title']()}</h1>
    <div class="splash-dots" aria-hidden="true">
      <span class="dot"></span>
      <span class="dot"></span>
      <span class="dot"></span>
    </div>
  </div>
</div>

<style>
  .splash {
    position: fixed;
    inset: 0;
    z-index: 999;
    display: grid;
    place-items: center;
    background: var(--bg-deep);
    transition: opacity 0.45s ease, visibility 0.45s ease;
  }

  .splash.exit {
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
  }

  .splash-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .splash-badge {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 96px;
    height: 96px;
    border-radius: 50%;
    background: var(--bg-card);
    box-shadow: var(--neu-raised-lg);
    animation: splash-breathe 2.4s ease-in-out infinite;
  }

  :global(.splash-icon) {
    width: 44px;
    height: 44px;
    color: var(--accent);
    animation: splash-spin 5s linear infinite;
  }

  .splash-ring {
    position: absolute;
    width: 136px;
    height: 136px;
    border-radius: 50%;
    border: 2px solid transparent;
    box-shadow: 0 0 0 0 var(--accent-subtle);
    animation: splash-ring 2.4s ease-out infinite;
    pointer-events: none;
  }

  .splash-title {
    margin: 0;
    font-size: 1.6rem;
    font-weight: 800;
    letter-spacing: -0.02em;
    background: var(--accent-gradient);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .splash-dots {
    display: flex;
    gap: 8px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--bg-card);
    box-shadow: var(--neu-raised-sm);
    animation: splash-dot 1.2s ease-in-out infinite;
  }

  .dot:nth-child(2) {
    animation-delay: 0.15s;
  }

  .dot:nth-child(3) {
    animation-delay: 0.3s;
  }

  @keyframes splash-breathe {
    0%, 100% {
      transform: scale(1);
      box-shadow: var(--neu-raised-lg);
    }
    50% {
      transform: scale(1.04);
      box-shadow: var(--neu-float);
    }
  }

  @keyframes splash-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes splash-ring {
    0% {
      box-shadow: 0 0 0 0 var(--accent-subtle);
      opacity: 1;
    }
    100% {
      box-shadow: 0 0 0 22px transparent;
      opacity: 0;
    }
  }

  @keyframes splash-dot {
    0%, 100% {
      transform: translateY(0);
      opacity: 0.5;
    }
    50% {
      transform: translateY(-5px);
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .splash-badge,
    :global(.splash-icon),
    .splash-ring,
    .dot {
      animation: none;
    }
  }
</style>
