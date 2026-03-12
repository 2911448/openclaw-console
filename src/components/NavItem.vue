<template>
  <div
    class="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer select-none transition-colors border"
    :class="active ? 'bg-white/12 border-white/20 shadow-[inset_0_1px_0_rgba(255,255,255,0.08)]' : 'border-transparent hover:bg-white/6'"
    @click="$emit('select')"
  >
    <div class="h-2 w-2 rounded-full" :class="active ? 'bg-white/80' : 'bg-white/25'"></div>
    <div class="text-sm" :class="active ? 'text-white font-medium' : 'text-subtext'">{{ label }}</div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ label: string; active?: boolean }>()
defineEmits<{ select: [] }>()
</script>
