<script lang="ts">
  import { pages } from '$lib/i18n';
</script>

<svelte:head>
  <title>404 · NanoPrint</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<main
  class="container-page flex min-h-screen flex-col items-center justify-center gap-6 text-center"
>
  <img src="/brand/isologo-nanoprint.svg" alt="" width="64" height="80" class="h-20 w-auto" />
  <h1 class="text-3xl font-semibold">404</h1>
  <p class="text-muted" lang="es">Esta página no existe.</p>
  <p class="text-muted" lang="en">This page does not exist.</p>
  <p class="flex gap-6">
    <a href={pages.home.es} class="link" lang="es">Ir al inicio</a>
    <a href={pages.home.en} class="link" lang="en">Go to the home page</a>
  </p>
</main>
