<script lang="ts">
  export let view: "create" | "compose" | "library" | "engine";
  export let refresh: () => void;

  $: title =
    view === "create"
      ? "Generate"
      : view === "compose"
        ? "Compose"
        : view === "library"
          ? "Library"
          : "Engine setup";
</script>

<header class="topbar">
  <div>
    <p class="eyebrow">OFFLINE CREATIVE PRODUCTION</p>
    <h1>{title}</h1>
  </div>
  <div class="topbar-actions">
    <span class="local-badge">LOCAL ONLY</span>
    <button class="ghost-button" onclick={refresh}>Refresh</button>
  </div>
</header>
