<script setup lang="ts">
import { NTag } from 'naive-ui'

defineProps<{
  title: string
  summary: string | null
  tags: string[]
}>()
</script>

<template>
  <header class="space-y-2">
    <h1 class="text-xl font-semibold tracking-tight">
      {{ title }}
    </h1>
    <p v-if="summary" class="text-sm text-neutral-600 dark:text-neutral-400">
      {{ summary }}
    </p>
    <div v-if="tags.length" class="flex flex-wrap gap-1">
      <n-tag v-for="t in tags" :key="t" size="small" round :bordered="false">
        {{ t }}
      </n-tag>
    </div>
  </header>
</template>
