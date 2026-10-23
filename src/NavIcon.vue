<script setup lang="ts">
defineProps<{ name: string }>();
// Small local line icons; no external icon font or runtime dependency.
const paths: Record<string, string> = {
  window: 'M4 4h16v16H4z M4 8h16 M7 6h.01 M10 6h.01',
  sidebar: 'M4 4h16v16H4z M9 4v16 M14 9l-3 3 3 3',
  mouse: 'M12 3a6 6 0 0 0-6 6v6a6 6 0 0 0 12 0V9a6 6 0 0 0-6-6z M12 3v7 M6 10h12',
  timer: 'M9 2h6 M12 2v3 M18 5l2 2 M12 9v5l3 2 M20 14a8 8 0 1 1-16 0 8 8 0 0 1 16 0z',
  settings: 'M4 6h16 M4 12h16 M4 18h16 M8 3v6 M16 9v6 M10 15v6',
  info: 'M12 11v6 M12 7h.01 M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
  refresh: 'M20 4v6h-6 M20 10a8 8 0 1 0-1 8',
  play: 'M8 5l11 7-11 7z',
  stop: 'M6 6h12v12H6z',
  help: 'M9.1 9a3 3 0 0 1 5.8 1c0 2-3 2-3 4 M12 17h.01 M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
  search: 'M20 20l-5-5 M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0z',
  chevron: 'M9 5l7 7-7 7',
  check: 'M5 12l4 4L19 6',
  more: 'M5 12h.01 M12 12h.01 M19 12h.01',
  activity: 'M3 12h4l3-8 4 16 3-8h4',
  close: 'M6 6l12 12 M18 6L6 18',
};
</script>
<template><svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path :d="paths[name] ?? paths.info"/></svg></template>
