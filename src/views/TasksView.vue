<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { useDownloadStore } from "../lib/store";
import TaskList from "../components/TaskList.vue";
import NewDownloadDialog from "../components/NewDownloadDialog.vue";
import TaskDetailPanel from "../components/TaskDetailPanel.vue";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Play, Pause, Trash2, CheckSquare, SlidersHorizontal } from "lucide-vue-next";

const store = useDownloadStore();
const route = useRoute();
const showNewDialog = ref(false);
const droppedUrl = ref("");
const showConfirmDelete = ref(false);
const showBatchThreads = ref(false);
const batchThreadsValue = ref(4);
const batchQueueId = ref("");

function batchMoveToQueue() {
  if (!batchQueueId.value) return;
  selectedIds.value.forEach(id => store.moveTaskToQueue(id, batchQueueId.value));
  batchQueueId.value = "";
}

const batchResult = ref("");
let batchResultTimer: ReturnType<typeof setTimeout> | null = null;

function showBatchResult(msg: string) {
  batchResult.value = msg;
  if (batchResultTimer) clearTimeout(batchResultTimer);
  batchResultTimer = setTimeout(() => batchResult.value = "", 3000);
}

async function applyBatchThreads() {
  const count = Math.max(1, Math.min(128, batchThreadsValue.value));
  const ids = [...selectedIds.value];
  showBatchThreads.value = false;
  const results = await Promise.allSettled(
    ids.map(id => invoke("set_task_segments", { taskId: id, segments: count }))
  );
  const failed = results.filter(r => r.status === "rejected").length;
  if (failed === 0) {
    showBatchResult(`已为 ${ids.length} 个任务设置 ${count} 线程`);
  } else {
    showBatchResult(`${failed}/${ids.length} 个任务设置线程失败`);
  }
}
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
  window.addEventListener('open-new-download', onOpenNewDownload);
});

onUnmounted(() => {
  window.removeEventListener('batch-delete', onBatchDelete);
  window.removeEventListener('batch-toggle', onBatchToggle);
  window.removeEventListener('open-new-download', onOpenNewDownload);
});

function onOpenNewDownload() {
  showNewDialog.value = true;
  droppedUrl.value = '';
}

function onBatchDelete() {
  if (manageMode.value && selectedIds.value.size > 0) {
    showConfirmDelete.value = true;
  }
}

function confirmBatchDelete() {
  batchRemove();
  showConfirmDelete.value = false;
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
    <div class="flex items-center gap-2 px-4 py-2.5" :style="{ borderBottom: '1px solid var(--surface-border)' }">
      <button
        v-for="tab in store.filterTabs" :key="tab.id"
        @click="store.activeFilter = tab.id"
        :class="['rounded-md px-3 py-1 text-sm transition-colors', store.activeFilter === tab.id ? '' : 'hover-bg']"
        :style="store.activeFilter === tab.id
          ? { backgroundColor: 'var(--accent)', color: 'var(--text-primary)' }
          : { color: 'var(--text-muted)', backgroundColor: 'transparent' }"
      >
        {{ tab.label }} ({{ tab.count }})
      </button>
      <div class="flex-1" />
      <button data-manage-mode :class="['rounded p-1.5 transition-colors', manageMode ? 'bg-blue-500/20 text-blue-500' : '']"
        :style="{ color: manageMode ? 'var(--accent)' : 'var(--text-muted)' }"
        @click="manageMode = !manageMode; if (!manageMode) selectedIds = new Set()">
        <CheckSquare class="h-4 w-4" />
      </button>
      <button v-if="store.settings.showTitlebarPauseAll" @click="store.pauseAll" class="rounded p-1.5 transition-colors" :style="{ color: 'var(--text-muted)' }">
        <Pause class="h-4 w-4" />
      </button>
      <button v-if="store.settings.showTitlebarResumeAll" @click="store.resumeAll" class="rounded p-1.5 transition-colors" :style="{ color: 'var(--text-muted)' }">
        <Play class="h-4 w-4" />
      </button>
      <button @click="showNewDialog = true; droppedUrl = ''"
        class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
        :style="{ backgroundColor: 'var(--accent)', color: 'var(--text-primary)' }">
        <Plus class="h-3.5 w-3.5" /> 新建
      </button>
    </div>

    <!-- Batch action bar -->
    <!-- Batch result toast -->
    <div v-if="batchResult"
      class="flex items-center justify-center px-4 py-1.5"
      :style="{ backgroundColor: 'rgba(var(--accent-rgb),0.1)', borderBottom: '1px solid var(--surface-border)' }"
    >
      <span class="text-xs" :style="{ color: 'var(--accent)' }">{{ batchResult }}</span>
    </div>

    <div v-if="manageMode && selectedIds.size > 0"
      class="flex items-center gap-2 px-4 py-2"
      :style="{ backgroundColor: 'rgba(var(--accent-rgb),0.1)', borderBottom: '1px solid var(--surface-border)' }"
    >
      <span class="text-xs" :style="{ color: 'var(--text-secondary)' }">已选择 {{ selectedIds.size }} 个任务</span>
      <div class="flex-1"></div>
      <button @click="batchResume" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: 'var(--success)' }"><Play class="h-3 w-3 inline" /> 恢复</button>
      <button @click="batchPause" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: 'var(--gold)' }"><Pause class="h-3 w-3 inline" /> 暂停</button>
      <button @click="showConfirmDelete = true" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: '#D64531' }"><Trash2 class="h-3 w-3 inline" /> 删除</button>
      <div v-if="showBatchThreads" class="flex items-center gap-1">
        <input v-model.number="batchThreadsValue" type="number" min="1" max="128"
          class="w-14 rounded px-1.5 py-0.5 text-xs outline-none tabular-nums"
          :style="{ backgroundColor: 'var(--app-bg)', border: '1px solid var(--surface-border)', color: 'var(--text-primary)' }"
          @keydown.enter="applyBatchThreads"
        />
        <button @click="applyBatchThreads" class="rounded px-1.5 py-0.5 text-2xs transition-colors"
          :style="{ backgroundColor: 'var(--accent)', color: 'var(--text-primary)' }">应用</button>
        <button @click="showBatchThreads = false" class="rounded px-1 py-0.5 text-2xs transition-colors"
          :style="{ color: 'var(--text-muted)' }">取消</button>
      </div>
      <button v-else @click="showBatchThreads = true" class="rounded px-2 py-1 text-xs transition-colors"
        :style="{ color: 'var(--text-secondary)' }"><SlidersHorizontal class="h-3 w-3 inline" /> 线程</button>
      <select v-if="Object.keys(store.queueStates).length > 0" v-model="batchQueueId"
        class="rounded-md px-2 py-1 text-xs outline-none"
        :style="{ backgroundColor: 'var(--app-bg)', border: '1px solid var(--surface-border)', color: 'var(--text-secondary)' }"
        @change="batchMoveToQueue"
      >
        <option value="">移动到队列...</option>
        <option v-for="(_, qid) in store.queueStates" :key="qid" :value="qid">{{ store.queueLabels[qid] || qid }}</option>
      </select>
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

    <!-- Batch delete confirmation -->
    <div v-if="showConfirmDelete"
      class="fixed inset-0 z-50 flex items-center justify-center"
      :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
      @click.self="showConfirmDelete = false"
    >
      <div class="w-80 rounded-xl p-5 shadow-2xl" :style="{ backgroundColor: 'var(--surface-bg)', border: '1px solid var(--surface-border)' }">
        <h3 class="text-sm font-semibold mb-2 font-kai" :style="{ color: 'var(--text-primary)' }">确认删除</h3>
        <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">确定要删除选中的 {{ selectedIds.size }} 个任务吗？此操作不可撤销。</p>
        <div class="flex justify-end gap-2">
          <button @click="showConfirmDelete = false" class="rounded-md px-4 py-1.5 text-xs transition-colors"
            :style="{ color: 'var(--text-secondary)' }"
            @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
            @mouseleave="$event.target.style.backgroundColor='transparent'"
          >取消</button>
          <button @click="confirmBatchDelete" class="rounded-md px-4 py-1.5 text-xs transition-colors"
            :style="{ backgroundColor: '#D64531', color: 'var(--text-primary)' }"
            @mouseenter="$event.target.style.opacity='0.9'"
            @mouseleave="$event.target.style.opacity='1'"
          >删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
