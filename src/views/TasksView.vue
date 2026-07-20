<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { useDownloadStore } from "../lib/store";
import TaskList from "../components/TaskList.vue";
import NewDownloadDialog from "../components/NewDownloadDialog.vue";
import TaskDetailPanel from "../components/TaskDetailPanel.vue";
import { Plus, Play, Pause, Trash2, CheckSquare } from "lucide-vue-next";

const store = useDownloadStore();
const route = useRoute();
const showNewDialog = ref(false);
const droppedUrl = ref("");
const manageMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());
const selectedTask = ref<any | null>(null);

watch(() => route.query.status, (status) => {
  store.activeFilter = ({ completed: "completed", active: "active", paused: "paused", error: "error" })[status as string] ?? "all";
}, { immediate: true });

const allSelected = computed(() =>
  store.filteredTasks.length > 0 && selectedIds.value.size === store.filteredTasks.length
);

function toggleSelectAll() {
  if (allSelected.value) {
    selectedIds.value = new Set();
  } else {
    selectedIds.value = new Set(store.filteredTasks.map(t => t.id));
  }
}

function toggleSelect(id: string) {
  const s = new Set(selectedIds.value);
  if (s.has(id)) s.delete(id); else s.add(id);
  selectedIds.value = s;
}

function batchPause() {
  selectedIds.value.forEach(id => store.pauseTask(id));
  selectedIds.value = new Set();
}

function batchResume() {
  selectedIds.value.forEach(id => store.resumeTask(id));
  selectedIds.value = new Set();
}

function batchRemove() {
  selectedIds.value.forEach(id => store.removeTask(id));
  selectedIds.value = new Set();
}

function onSelect(task: any) {
  selectedTask.value = task;
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

onMounted(() => {
  window.addEventListener('batch-delete', onBatchDelete);
  window.addEventListener('batch-toggle', onBatchToggle);
});

onUnmounted(() => {
  window.removeEventListener('batch-delete', onBatchDelete);
  window.removeEventListener('batch-toggle', onBatchToggle);
});

function onBatchDelete() {
  if (manageMode.value && selectedIds.value.size > 0) {
    batchRemove();
  }
}

function onBatchToggle() {
  if (manageMode.value && selectedIds.value.size > 0) {
    const allPaused = [...selectedIds.value].every(id => {
      const t = store.tasks.find(t => t.id === id);
      return t?.status === 2;
    });
    if (allPaused) batchResume(); else batchPause();
  }
}

function onDrop(e: DragEvent) {
  e.preventDefault();
  const text = e.dataTransfer?.getData('text') || e.dataTransfer?.getData('text/plain');
  if (text && (text.startsWith('http://') || text.startsWith('https://') || text.startsWith('magnet:') || text.startsWith('ftp://'))) {
    droppedUrl.value = text;
    showNewDialog.value = true;
  }
}

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
  if (task.status !== 1 || task.downloaded_bytes <= 0 || task.total_bytes <= 0 || !task.speed || task.speed <= 0) return "—";
  const remaining = task.total_bytes - task.downloaded_bytes;
  const seconds = Math.round(remaining / task.speed);
  if (seconds < 60) return `约 ${seconds} 秒`;
  if (seconds < 3600) return `约 ${Math.round(seconds / 60)} 分钟`;
  return `约 ${(seconds / 3600).toFixed(1)} 小时`;
}
</script>

<template>
  <div class="flex h-full flex-col" @dragover="onDragOver" @drop="onDrop">
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
      <button :class="['rounded p-1.5 transition-colors', manageMode ? 'bg-blue-500/20 text-blue-500' : '']"
        :style="{ color: manageMode ? '#3B82F6' : '#8E8E93' }"
        @click="manageMode = !manageMode; if (!manageMode) selectedIds = new Set()">
        <CheckSquare class="h-4 w-4" />
      </button>
      <button @click="store.pauseAll" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
        <Pause class="h-4 w-4" />
      </button>
      <button @click="store.resumeAll" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
        <Play class="h-4 w-4" />
      </button>
      <button @click="showNewDialog = true; droppedUrl = ''"
        class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
        :style="{ backgroundColor: '#3B82F6', color: '#fff' }">
        <Plus class="h-3.5 w-3.5" /> 新建
      </button>
    </div>

    <!-- Batch action bar -->
    <div v-if="manageMode && selectedIds.size > 0"
      class="flex items-center gap-2 px-4 py-2"
      :style="{ backgroundColor: 'rgba(59,130,246,0.1)', borderBottom: '1px solid #3A3A3C' }"
    >
      <span class="text-xs" :style="{ color: '#A1A1A6' }">已选择 {{ selectedIds.size }} 个任务</span>
      <div class="flex-1"></div>
      <button @click="batchResume" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: '#22C55E' }"><Play class="h-3 w-3 inline" /> 恢复</button>
      <button @click="batchPause" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: '#F59E0B' }"><Pause class="h-3 w-3 inline" /> 暂停</button>
      <button @click="batchRemove" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: '#EF4444' }"><Trash2 class="h-3 w-3 inline" /> 删除</button>
    </div>

    <TaskList
      :tasks="store.filteredTasks"
      :format-bytes="formatBytes"
      :format-speed="formatSpeed"
      :format-eta="formatEta"
      :manage-mode="manageMode"
      :selected-ids="selectedIds"
      @toggle-select="toggleSelect"
      @toggle-select-all="toggleSelectAll"
      @select="onSelect"
      :all-selected="allSelected"
    />

    <TaskDetailPanel
      v-if="selectedTask && !manageMode"
      :task="selectedTask"
      :format-bytes="formatBytes"
      :format-speed="formatSpeed"
      @close="selectedTask = null"
    />

    <NewDownloadDialog v-if="showNewDialog" :initial-url="droppedUrl" @close="showNewDialog = false; droppedUrl = ''" />
  </div>
</template>
