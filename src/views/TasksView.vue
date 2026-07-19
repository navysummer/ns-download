<script setup lang="ts">
import { ref, watch } from "vue";
import { useRoute } from "vue-router";
import { useDownloadStore } from "../lib/store";
import TaskList from "../components/TaskList.vue";
import NewDownloadDialog from "../components/NewDownloadDialog.vue";
import { Plus, Play, Pause, Trash2 } from "lucide-vue-next";

const store = useDownloadStore();
const route = useRoute();
const showNewDialog = ref(false);

watch(() => route.query.status, (status) => {
  store.activeFilter = ({ completed: "completed", active: "active", paused: "paused", error: "error" })[status as string] ?? "all";
}, { immediate: true });

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + ["B", "KB", "MB", "GB", "TB"][i];
}

function formatSpeed(bytes: number): string {
  if (bytes <= 0) return "—";
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
  return `${Math.round(bytes / 1024)} KB/s`;
}

function formatEta(task: any): string {
  if (task.status !== 1 || task.downloaded_bytes <= 0 || task.total_bytes <= 0) return "—";
  const remaining = task.total_bytes - task.downloaded_bytes;
  const speed = 1_200_000; // placeholder — real speed from engine
  const seconds = Math.round(remaining / speed);
  if (seconds < 60) return `约 ${seconds} 秒`;
  if (seconds < 3600) return `约 ${Math.round(seconds / 60)} 分钟`;
  return `约 ${(seconds / 3600).toFixed(1)} 小时`;
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-2 px-4 py-2.5" :style="{ borderBottom: '1px solid #3A3A3C' }">
      <button
        v-for="tab in store.filterTabs" :key="tab.id"
        @click="store.activeFilter = tab.id"
        :class="['rounded-md px-3 py-1 text-sm transition-colors', store.activeFilter === tab.id ? '' : 'hover-bg']"
        :style="store.activeFilter === tab.id
          ? { backgroundColor: '#3B82F6', color: '#fff' }
          : { color: '#8E8E93', backgroundColor: 'transparent' }"
      >
        {{ tab.label }} ({{ tab.count }})
      </button>
      <div class="flex-1" />
      <button @click="store.pauseAll" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
        <Pause class="h-4 w-4" />
      </button>
      <button @click="store.resumeAll" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
        <Play class="h-4 w-4" />
      </button>
      <button @click="showNewDialog = true"
        class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
        :style="{ backgroundColor: '#3B82F6', color: '#fff' }">
        <Plus class="h-3.5 w-3.5" /> 新建
      </button>
    </div>

    <TaskList
      :tasks="store.filteredTasks"
      :format-bytes="formatBytes"
      :format-speed="formatSpeed"
      :format-eta="formatEta"
    />

    <NewDownloadDialog v-if="showNewDialog" @close="showNewDialog = false" />
  </div>
</template>
