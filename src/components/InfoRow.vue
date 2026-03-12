<template>
  <div class="flex items-center justify-between gap-3">
    <div class="text-[11px] text-subtext">{{ label }}</div>
    <div class="text-sm font-medium truncate">{{ value }}</div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ label: string; value: string }>()
</script>
