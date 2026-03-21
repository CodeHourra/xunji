<script setup lang="ts">
import TopBar from './TopBar.vue'
import Sidebar from './Sidebar.vue'
import AnalysisQueuePanel from './AnalysisQueuePanel.vue'
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
    <TopBar />
    <div class="flex flex-1 min-h-0">
      <Sidebar />
      <main class="flex-1 min-w-0 min-h-0 overflow-hidden bg-neutral-50/50 dark:bg-neutral-950">
        <router-view />
      </main>
    </div>
    <AnalysisQueuePanel />
  </div>
</template>
