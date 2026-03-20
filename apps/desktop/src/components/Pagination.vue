<script setup lang="ts">
import { NPagination } from 'naive-ui'

defineProps<{
  page: number
  pageSize: number
  total: number
}>()

const emit = defineEmits<{
  'update:page': [n: number]
  'update:pageSize': [n: number]
}>()
</script>

<template>
  <div class="flex justify-end">
    <n-pagination
      :page="page"
      :page-size="pageSize"
      :item-count="total"
      :page-sizes="[10, 20, 50, 100]"
      show-size-picker
      size="small"
      @update:page="emit('update:page', $event)"
      @update:page-size="emit('update:pageSize', $event)"
    />
  </div>
</template>
