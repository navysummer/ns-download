<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useDownloadStore } from "../lib/store";
import { Play, Pause, Music, Film, File, Trash2, FolderOpen, Copy, ArrowUp, List } from "lucide-vue-next";

const props = defineProps<{
  tasks: any[];
  formatBytes: (bytes: number) => string;
  formatSpeed: (bytes: number) => string;
  formatEta: (task: any) => string;
  manageMode?: boolean;
  selectedIds?: Set<string>;
  allSelected?: boolean;
}>();

const emit = defineEmits<{
  (e: 'toggle-select', id: string): void;
  (e: 'toggle-select-all'): void;
}>();

const store = useDownloadStore();
const selectedId = ref<string | null>(null);
const contextMenu = ref({ show: false, x: 0, y: 0, task: null as any | null });

function closeContextMenu() {
  contextMenu.value.show = false;
  contextMenu.value.task = null;
}

function onContextMenu(e: MouseEvent, task: any) {
  e.preventDefault();
  selectedId.value = task.id;
  contextMenu.value = { show: true, x: e.clientX, y: e.clientY, task };
}

function onClickOutside(e: MouseEvent) {
  if (contextMenu.value.show) {
    const el = document.getElementById('task-context-menu');
    if (el && !el.contains(e.target as Node)) closeContextMenu();
  }
}

onMounted(() => document.addEventListener('click', onClickOutside));
onUnmounted(() => document.removeEventListener('click', onClickOutside));

function progressColor(status: number): string {
  return ['#F59E0B', '#3B82F6', '#F59E0B', '#22C55E', '#EF4444', '#3B82F6'][status] || '#8E8E93';
}

function statusText(status: number): string {
  return ['等待中', '下载中', '已暂停', '已完成', '出错', '准备中'][status] || '未知';
}

function extBadge(name: string): string {
  const ext = name?.split('.').pop()?.toLowerCase() || '';
  const map: Record<string, string> = {
    mp3: 'MP3', flac: 'FLAC', wav: 'WAV', aac: 'AAC', ogg: 'OGG',
    mp4: 'MP4', mkv: 'MKV', avi: 'AVI', mov: 'MOV', webm: 'WEBM',
    zip: 'ZIP', rar: 'RAR', '7z': '7Z', tar: 'TAR', gz: 'GZ',
    pdf: 'PDF', epub: 'EPUB', mobi: 'MOBI',
    jpg: 'JPG', png: 'PNG', gif: 'GIF', webp: 'WEBP',
    exe: 'EXE', dmg: 'DMG', deb: 'DEB', appimage: 'APP',
  };
  return map[ext] || ext.toUpperCase() || 'FILE';
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div v-if="tasks.length === 0" class="flex h-full items-center justify-center" :style="{ color: '#8E8E93' }">
      暂无任务
    </div>
    <div v-else>
      <!-- Select All header -->
      <div v-if="manageMode" class="flex items-center gap-3 px-4 py-1.5" :style="{ borderBottom: '1px solid #3A3A3C', backgroundColor: '#2C2C2E' }">
        <label class="flex cursor-pointer items-center gap-2 text-xs" :style="{ color: '#A1A1A6' }">
          <input type="checkbox" :checked="allSelected" @change="emit('toggle-select-all')"
            class="h-3.5 w-3.5 rounded"
            :style="{ accentColor: '#3B82F6' }"
          />
          全选
        </label>
      </div>
      <div
        v-for="task in tasks" :key="task.id"
        @click="manageMode ? emit('toggle-select', task.id) : (selectedId = task.id)"
        @contextmenu="onContextMenu($event, task)"
        :class="selectedId === task.id ? 'flex cursor-pointer items-center gap-0 px-4 transition-colors bg-selected' : 'flex cursor-pointer items-center gap-0 px-4 transition-colors hover-bg'"
        :style="{
          height: '64px',
          borderBottom: '1px solid #3A3A3C',
          backgroundColor: selectedId === task.id && !manageMode ? '#3A3A3C' : 'transparent',
        }"
      >
        <!-- Checkbox in manage mode -->
        <input v-if="manageMode" type="checkbox"
          :checked="selectedIds?.has(task.id)"
          @click.stop="emit('toggle-select', task.id)"
          class="mr-3 h-4 w-4 shrink-0"
          :style="{ accentColor: '#3B82F6' }"
        />
        <!-- Selected indicator -->
        <div v-if="!manageMode && selectedId === task.id" class="mr-3 shrink-0 rounded-sm" :style="{ width: '3px', height: '28px', backgroundColor: '#3B82F6' }"></div>

        <!-- File Info (flex: 1) -->
        <div class="flex min-w-0 flex-1 items-center gap-3">
          <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md" :style="{ backgroundColor: '#2C2C2E' }">
            <span class="text-2xs font-semibold tabular-nums" :style="{ color: '#A1A1A6' }">{{ extBadge(task.file_name || task.url) }}</span>
          </div>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium" :style="{ color: '#F5F5F7' }">{{ task.file_name || task.url.split('/').pop() || task.url }}</div>
            <div class="truncate text-2xs" :style="{ color: '#8E8E93' }">{{ task.url }}</div>
          </div>
        </div>

        <!-- Progress (~150px) -->
        <div class="shrink-0 px-3" style="width: 150px;">
          <div class="flex items-center gap-2">
            <div class="flex-1 h-1 rounded-full" :style="{ backgroundColor: '#3A3A3C' }">
              <div class="h-full rounded-full transition-all duration-300"
                :style="{
                  width: `${task.total_bytes > 0 ? Math.round((task.downloaded_bytes / task.total_bytes) * 100) : 0}%`,
                  backgroundColor: progressColor(task.status),
                }"
              />
            </div>
            <span class="shrink-0 text-xs tabular-nums" :style="{ color: '#A1A1A6' }">
              {{ task.total_bytes > 0 ? Math.round((task.downloaded_bytes / task.total_bytes) * 100) : 0 }}%
            </span>
          </div>
        </div>

        <!-- Speed (~90px) -->
        <div class="shrink-0 text-xs tabular-nums text-center" style="width: 90px;" :style="{ color: task.status === 1 ? '#22C55E' : '#8E8E93' }">
          {{ task.status === 1 && task.speed > 0 ? props.formatSpeed(task.speed) : '—' }}
        </div>

        <!-- ETA (~80px) -->
        <div class="shrink-0 text-xs tabular-nums text-center" style="width: 80px;" :style="{ color: task.status === 1 ? '#A1A1A6' : '#8E8E93' }">
          {{ task.status === 1 && task.speed > 0 && task.total_bytes > 0 ? props.formatEta(task) : '—' }}
        </div>

        <!-- Status (~60px) -->
        <div class="shrink-0 text-center text-xs" style="width: 60px;" :style="{ color: progressColor(task.status) }">
          {{ statusText(task.status) }}
        </div>

        <!-- Actions -->
        <div class="flex shrink-0 items-center gap-0.5 pl-2" style="width: 72px;">
          <button v-if="task.status === 2" @click.stop="store.resumeTask(task.id)" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
            <Play class="h-3.5 w-3.5" />
          </button>
          <button v-else-if="task.status === 1" @click.stop="store.pauseTask(task.id)" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
            <Pause class="h-3.5 w-3.5" />
          </button>
          <button @click.stop="store.removeTask(task.id)" class="rounded p-1.5 transition-colors" :style="{ color: '#8E8E93' }">
            <Trash2 class="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <div v-if="contextMenu.show"
      id="task-context-menu"
      :style="{
        position: 'fixed',
        left: contextMenu.x + 'px',
        top: contextMenu.y + 'px',
        zIndex: 9999,
        backgroundColor: '#2C2C2E',
        border: '1px solid #48484A',
        borderRadius: '8px',
        padding: '4px',
        minWidth: '180px',
        boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
      }"
    >
      <button v-if="contextMenu.task?.status === 1"
        @click="store.pauseTask(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#F5F5F7' }"
        @mouseenter="$event.target.style.backgroundColor='#3A3A3C'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Pause class="h-3.5 w-3.5" /> 暂停
      </button>
      <button v-if="contextMenu.task?.status === 2"
        @click="store.resumeTask(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#F5F5F7' }"
        @mouseenter="$event.target.style.backgroundColor='#3A3A3C'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Play class="h-3.5 w-3.5" /> 恢复
      </button>
      <button
        @click="store.setTaskPriority(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#F5F5F7' }"
        @mouseenter="$event.target.style.backgroundColor='#3A3A3C'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <ArrowUp class="h-3.5 w-3.5" /> 优先下载
      </button>
      <div :style="{ borderBottom: '1px solid #3A3A3C', margin: '4px 8px' }"></div>
      <button
        @click="navigator.clipboard.writeText(contextMenu.task.url); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#F5F5F7' }"
        @mouseenter="$event.target.style.backgroundColor='#3A3A3C'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Copy class="h-3.5 w-3.5" /> 复制链接
      </button>
      <button
        @click="store.revealInFolder(contextMenu.task.save_dir + '/' + contextMenu.task.file_name); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#F5F5F7' }"
        @mouseenter="$event.target.style.backgroundColor='#3A3A3C'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <FolderOpen class="h-3.5 w-3.5" /> 打开文件位置
      </button>
      <div :style="{ borderBottom: '1px solid #3A3A3C', margin: '4px 8px' }"></div>
      <button
        @click="store.removeTask(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#EF4444' }"
        @mouseenter="$event.target.style.backgroundColor='#3A3A3C'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Trash2 class="h-3.5 w-3.5" /> 删除任务
      </button>
    </div>
  </div>
</template>
