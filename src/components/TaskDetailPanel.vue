<script setup lang="ts">
import { ref, computed } from "vue";
import { X, Info, FileIcon, GitBranch, Users, Circle } from "lucide-vue-next";

const props = defineProps<{
  task: any;
  formatBytes: (bytes: number) => string;
  formatSpeed: (bytes: number) => string;
}>();

const emit = defineEmits<{ close: [] }>();

const activeTab = ref("info");

const tabs = [
  { id: "info", icon: Info, label: "信息" },
  { id: "files", icon: FileIcon, label: "文件" },
  { id: "segments", icon: GitBranch, label: "分段" },
];

const panelHeight = ref(240);

const progressPercent = computed(() =>
  props.task.total_bytes > 0
    ? Math.round((props.task.downloaded_bytes / props.task.total_bytes) * 100)
    : 0
);

const statusMap: Record<number, string> = {
  0: "等待中", 1: "下载中", 2: "已暂停",
  3: "已完成", 4: "出错", 5: "准备中",
};

const totalBlocks = 160;

function segmentColor(row: number, col: number): string {
  const idx = (row - 1) * 40 + col;
  const progress = props.task.total_bytes > 0
    ? props.task.downloaded_bytes / props.task.total_bytes
    : 0;
  // Simulate: first `progress * totalBlocks` blocks are done
  // A few blocks after that are "active" (downloading)
  const doneThreshold = Math.floor(progress * totalBlocks);
  const activeZone = 3;
  if (idx <= doneThreshold - activeZone) return 'var(--accent)';
  if (idx <= doneThreshold) return 'var(--success)';
  if (idx <= doneThreshold + 2 && props.task.status === 1) return 'var(--accent)';
  return 'var(--surface-bg2)';
}

let dragStartY = 0;
let dragStartHeight = 0;

function onDragStart(e: MouseEvent) {
  dragStartY = e.clientY;
  dragStartHeight = panelHeight.value;
  document.addEventListener('mousemove', onDragMove);
  document.addEventListener('mouseup', onDragEnd);
  e.preventDefault();
}

function onDragMove(e: MouseEvent) {
  const delta = dragStartY - e.clientY;
  panelHeight.value = Math.max(120, Math.min(600, dragStartHeight + delta));
}

function onDragEnd() {
  document.removeEventListener('mousemove', onDragMove);
  document.removeEventListener('mouseup', onDragEnd);
}
</script>

<template>
  <div
    class="flex flex-col shrink-0 overflow-hidden transition-all duration-200"
    :style="{
      borderTop: '1px solid var(--surface-border)',
      backgroundColor: 'var(--surface-bg)',
      height: panelHeight + 'px',
    }"
  >
    <!-- Drag handle -->
    <div
      class="flex cursor-row-resize items-center justify-center py-0.5 shrink-0"
      :style="{ backgroundColor: 'var(--surface-dark)' }"
      @mousedown="onDragStart"
    >
      <div class="h-0.5 w-8 rounded-full" :style="{ backgroundColor: 'var(--surface-border)' }"></div>
    </div>

    <!-- Title bar -->
    <div class="flex items-center justify-between px-4 py-2 shrink-0" :style="{ borderBottom: '1px solid var(--surface-border)' }">
      <div class="flex items-center gap-3 min-w-0">
        <span class="text-sm font-medium truncate" :style="{ color: 'var(--text-primary)' }">
          {{ task.file_name || task.url.split('/').pop() || task.url }}
        </span>
        <span class="rounded px-1.5 py-0.5 text-2xs" :style="{
          backgroundColor: task.status === 1 ? 'rgba(78,122,90,0.18)' : task.status === 3 ? 'rgba(78,122,90,0.18)' : 'rgba(156,130,96,0.18)',
          color: task.status === 1 ? 'var(--success)' : task.status === 3 ? 'var(--success)' : 'var(--text-muted)',
        }">{{ statusMap[task.status] || '未知' }}</span>
      </div>
      <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: 'var(--text-muted)' }">
        <X class="h-4 w-4" />
      </button>
    </div>

    <!-- Tab bar -->
    <div class="flex items-center gap-1 px-3 pt-2 shrink-0" :style="{ borderBottom: '1px solid var(--surface-border)' }">
      <button
        v-for="tab in tabs" :key="tab.id"
        @click="activeTab = tab.id"
        class="flex items-center gap-1.5 rounded-t-md px-3 py-1.5 text-xs transition-colors"
        :style="activeTab === tab.id
          ? { backgroundColor: 'var(--surface-bg2)', color: 'var(--text-primary)', borderBottom: '2px solid var(--accent)' }
          : { color: 'var(--text-muted)' }"
      >
        <component :is="tab.icon" class="h-3.5 w-3.5" />
        {{ tab.label }}
      </button>
    </div>

    <!-- Tab content -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Info tab -->
      <template v-if="activeTab === 'info'">
        <div class="grid grid-cols-2 gap-x-8 gap-y-3 text-sm">
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">文件大小</div>
            <div :style="{ color: 'var(--text-primary)' }">{{ formatBytes(task.total_bytes) }}</div>
          </div>
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">已下载</div>
            <div :style="{ color: 'var(--text-primary)' }">{{ formatBytes(task.downloaded_bytes) }} ({{ progressPercent }}%)</div>
          </div>
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">下载速度</div>
            <div :style="{ color: task.speed > 0 ? 'var(--success)' : 'var(--text-muted)' }">{{ task.speed > 0 ? formatSpeed(task.speed) : '—' }}</div>
          </div>
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">上传速度</div>
            <div :style="{ color: task.upload_speed > 0 ? 'var(--success)' : 'var(--text-muted)' }">{{ task.upload_speed > 0 ? formatSpeed(task.upload_speed) : '—' }}</div>
          </div>
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">分段数</div>
            <div :style="{ color: 'var(--text-primary)' }">{{ task.segments || '—' }}</div>
          </div>
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">保存位置</div>
            <div :style="{ color: 'var(--text-secondary)' }" class="truncate">{{ task.save_dir || '—' }}</div>
          </div>
          <div>
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: 'var(--text-muted)' }">下载链接</div>
            <div :style="{ color: 'var(--text-secondary)' }" class="truncate">{{ task.url || '—' }}</div>
          </div>
          <div v-if="task.error_message">
            <div class="text-2xs font-medium uppercase tracking-wider mb-0.5" :style="{ color: '#D64531' }">错误信息</div>
            <div :style="{ color: '#D64531' }">{{ task.error_message }}</div>
          </div>
        </div>
      </template>

      <!-- Files tab -->
      <template v-if="activeTab === 'files'">
        <div v-if="task.file_name" class="flex items-center gap-3 rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--app-bg)' }">
          <FileIcon class="h-5 w-5 shrink-0" :style="{ color: 'var(--accent)' }" />
          <div class="min-w-0 flex-1">
            <div class="text-sm truncate" :style="{ color: 'var(--text-primary)' }">{{ task.file_name }}</div>
            <div class="text-2xs" :style="{ color: 'var(--text-muted)' }">{{ formatBytes(task.total_bytes) }}</div>
          </div>
          <div class="shrink-0">
            <span class="text-xs tabular-nums" :style="{ color: 'var(--success)' }">{{ progressPercent }}%</span>
          </div>
        </div>
        <div v-else class="flex items-center justify-center h-full" :style="{ color: 'var(--text-muted)' }">
          <span class="text-sm">无文件信息</span>
        </div>
      </template>

      <!-- Segments tab -->
      <template v-if="activeTab === 'segments'">
        <div class="flex flex-col gap-3">
          <!-- Stats row -->
          <div class="flex items-center gap-4 text-xs">
            <div class="flex items-center gap-1.5">
              <Circle class="h-2.5 w-2.5 fill-current" :style="{ color: 'var(--accent)' }" />
              <span :style="{ color: 'var(--text-secondary)' }">已完成</span>
            </div>
            <div class="flex items-center gap-1.5">
              <Circle class="h-2.5 w-2.5 fill-current" :style="{ color: 'var(--success)' }" />
              <span :style="{ color: 'var(--text-secondary)' }">下载中</span>
            </div>
            <div class="flex items-center gap-1.5">
              <Circle class="h-2.5 w-2.5 fill-current" :style="{ color: 'var(--surface-bg2)' }" />
              <span :style="{ color: 'var(--text-secondary)' }">等待中</span>
            </div>
            <span class="ml-auto tabular-nums" :style="{ color: 'var(--text-muted)' }">
              {{ task.segments || '—' }} 分段 · {{ task.speed > 0 ? '活跃' : '空闲' }}
            </span>
          </div>

          <!-- Segment grid: simulate segments based on overall progress -->
          <div v-if="task.total_bytes > 0" class="space-y-0.5">
            <!-- Generate segment-like blocks -->
            <div
              v-for="row in 4" :key="'row-'+row"
              class="flex gap-0.5"
            >
              <div
                v-for="col in 40" :key="'seg-'+row+'-'+col"
                class="flex-1 rounded-sm transition-colors duration-300"
                :style="{
                  height: '12px',
                  backgroundColor: segmentColor(row, col),
                }"
                :title="`分段 ${(row-1)*40+col}`"
              ></div>
            </div>
          </div>

          <!-- Progress overview bar -->
          <div class="mt-1">
            <div class="flex items-center justify-between text-2xs mb-1" :style="{ color: 'var(--text-muted)' }">
              <span>{{ formatBytes(task.downloaded_bytes) }} / {{ formatBytes(task.total_bytes) }}</span>
              <span>{{ progressPercent }}%</span>
            </div>
            <div class="h-2 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--surface-bg2)' }">
              <div class="h-full rounded-full transition-all duration-300"
                :style="{
                  width: progressPercent + '%',
                  background: 'linear-gradient(90deg, var(--accent), var(--success))',
                }"
              ></div>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
