<script setup lang="ts">
</script>

<template>
  <div class="h-screen flex flex-col items-center justify-center bg-neutral-50 dark:bg-neutral-900">
    <h1 class="text-3xl font-bold text-neutral-800 dark:text-neutral-100">
      寻迹 XunJi
    </h1>
    <p class="mt-2 text-neutral-500">
      AI 编程知识管理平台
    </p>
    <div class="mt-4 flex gap-2">
      <span class="px-3 py-1 rounded-full text-sm bg-value-high text-white">high</span>
      <span class="px-3 py-1 rounded-full text-sm bg-value-medium text-white">medium</span>
      <span class="px-3 py-1 rounded-full text-sm bg-value-low text-neutral-600">low</span>
    </div>
  </div>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
}
</style>
