<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useDownloadStore } from "../lib/store";
import type { Task } from "../lib/store";
import { Play, Pause, Music, Film, File, Trash2, FolderOpen, Copy, ArrowUp, SlidersHorizontal, Star, ArrowUpDown, ArrowUpWideNarrow, ArrowDownWideNarrow, List } from "lucide-vue-next";
import EditThreadsDialog from "./EditThreadsDialog.vue";

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
  (e: 'select', task: any): void;
}>();

const store = useDownloadStore();
const selectedId = ref<string | null>(null);
const contextMenu = ref({ show: false, x: 0, y: 0, task: null as Task | null });
const showEditThreads = ref(false);
const showQueueSubmenu = ref(false);
const deleteConfirmTask = ref<Task | null>(null);

function closeContextMenu() {
  contextMenu.value.show = false;
  contextMenu.value.task = null;
}

function onDeleteClick(task: any) {
  if (task.status === 3) {
    deleteConfirmTask.value = task;
  } else {
    store.removeTask(task.id);
  }
}

function confirmDeleteTask(deleteFiles: boolean) {
  if (deleteConfirmTask.value) {
    store.removeTask(deleteConfirmTask.value.id, deleteFiles);
    deleteConfirmTask.value = null;
  }
}

function cancelDeleteTask() {
  deleteConfirmTask.value = null;
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
  return ['var(--gold)', 'var(--accent)', 'var(--gold)', 'var(--success)', '#D64531', 'var(--accent)'][status] || 'var(--text-muted)';
}

function statusText(status: number): string {
  return ['等待中', '下载中', '已暂停', '已完成', '出错', '准备中'][status] || '未知';
}

function urlFilename(url: string): string {
  return url.split(/[?#]/)[0].split('/').filter(Boolean).pop() || url;
}

function extBadge(name: string): string {
  if (!name) return '';
  // 只取文件名最后一段，避免把 URL 路径当作扩展名
  const filename = name.split(/[?#]/)[0].split('/').pop() || name;
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  if (!ext || ext === filename.toLowerCase()) return '';
  const map: Record<string, string> = {
    mp3: 'MP3', flac: 'FLAC', wav: 'WAV', aac: 'AAC', ogg: 'OGG',
    mp4: 'MP4', mkv: 'MKV', avi: 'AVI', mov: 'MOV', webm: 'WEBM',
    zip: 'ZIP', rar: 'RAR', '7z': '7Z', tar: 'TAR', gz: 'GZ',
    pdf: 'PDF', epub: 'EPUB', mobi: 'MOBI',
    jpg: 'JPG', png: 'PNG', gif: 'GIF', webp: 'WEBP',
    exe: 'EXE', dmg: 'DMG', deb: 'DEB', appimage: 'APP',
  };
  return map[ext] || ext.toUpperCase();
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div v-if="tasks.length === 0" class="flex h-full items-center justify-center" :style="{ color: 'var(--text-muted)' }">
      暂无任务
    </div>
    <div v-else>
      <!-- Select All header -->
      <div v-if="manageMode" class="flex items-center gap-3 px-4 py-1.5" :style="{ borderBottom: '1px solid var(--surface-border)', backgroundColor: 'var(--surface-bg)' }">
        <label class="flex cursor-pointer items-center gap-2 text-xs" :style="{ color: 'var(--text-secondary)' }">
          <input type="checkbox" :checked="allSelected" @change="emit('toggle-select-all')"
            class="h-3.5 w-3.5 rounded"
            :style="{ accentColor: 'var(--accent)' }"
          />
          全选
        </label>
      </div>
      <table :style="{ width: '100%', tableLayout: 'fixed', borderCollapse: 'collapse' }">
        <thead class="text-2xs font-medium uppercase tracking-wider select-none"
          :style="{ color: 'var(--text-muted)', backgroundColor: 'var(--surface-dark)' }"
        >
          <tr :style="{ height: '28px' }">
            <th :style="{ width: '200px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', verticalAlign: 'middle', fontWeight: 'inherit', textAlign: 'left' }">
              <button @click="store.setSort('file_name')" class="flex items-center gap-1 hover-text transition-colors w-full text-left truncate">
                文件
                <ArrowUpDown v-if="store.sortField !== 'file_name'" class="h-2.5 w-2.5 shrink-0 opacity-40" />
                <ArrowUpWideNarrow v-else-if="store.sortOrder === 'asc'" class="h-2.5 w-2.5 shrink-0" />
                <ArrowDownWideNarrow v-else class="h-2.5 w-2.5 shrink-0" />
              </button>
            </th>
            <th :style="{ width: '150px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', textAlign: 'center', verticalAlign: 'middle', fontWeight: 'inherit' }">
              <button @click="store.setSort('downloaded_bytes')" class="inline-flex items-center justify-center gap-1 hover-text transition-colors">
                进度
                <ArrowUpDown v-if="store.sortField !== 'downloaded_bytes'" class="h-2.5 w-2.5 opacity-40" />
                <ArrowUpWideNarrow v-else-if="store.sortOrder === 'asc'" class="h-2.5 w-2.5" />
                <ArrowDownWideNarrow v-else class="h-2.5 w-2.5" />
              </button>
            </th>
            <th :style="{ width: '85px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', textAlign: 'center', verticalAlign: 'middle', fontWeight: 'inherit' }">
              <span class="text-2xs font-medium uppercase tracking-wider" :style="{ color: 'var(--text-muted)' }">已下载</span>
            </th>
            <th :style="{ width: '85px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', textAlign: 'center', verticalAlign: 'middle', fontWeight: 'inherit' }">
              <span class="text-2xs font-medium uppercase tracking-wider" :style="{ color: 'var(--text-muted)' }">大小</span>
            </th>
            <th :style="{ width: '90px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', textAlign: 'center', verticalAlign: 'middle', fontWeight: 'inherit' }">
              <button @click="store.setSort('speed')" class="inline-flex items-center justify-center gap-1 hover-text transition-colors">
                速度
                <ArrowUpDown v-if="store.sortField !== 'speed'" class="h-2.5 w-2.5 opacity-40" />
                <ArrowUpWideNarrow v-else-if="store.sortOrder === 'asc'" class="h-2.5 w-2.5" />
                <ArrowDownWideNarrow v-else class="h-2.5 w-2.5" />
              </button>
            </th>
            <th :style="{ width: '80px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', textAlign: 'center', verticalAlign: 'middle', fontWeight: 'inherit' }">
              <span class="text-2xs font-medium uppercase tracking-wider" :style="{ color: 'var(--text-muted)' }">剩余</span>
            </th>
            <th :style="{ width: '60px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', textAlign: 'center', verticalAlign: 'middle', fontWeight: 'inherit' }">
              <button @click="store.setSort('status')" class="inline-flex items-center justify-center gap-1 hover-text transition-colors">
                状态
                <ArrowUpDown v-if="store.sortField !== 'status'" class="h-2.5 w-2.5 opacity-40" />
                <ArrowUpWideNarrow v-else-if="store.sortOrder === 'asc'" class="h-2.5 w-2.5" />
                <ArrowDownWideNarrow v-else class="h-2.5 w-2.5" />
              </button>
            </th>
            <th :style="{ width: '72px', padding: '0 4px', borderBottom: '1px solid var(--surface-border)', verticalAlign: 'middle', fontWeight: 'inherit' }"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="task in tasks" :key="task.id"
            @click="manageMode ? emit('toggle-select', task.id) : (selectedId = task.id, emit('select', task))"
            @contextmenu="onContextMenu($event, task)"
            :class="selectedId === task.id && !manageMode ? 'cursor-pointer bg-selected' : 'cursor-pointer hover-bg'"
            :style="{
              height: '64px',
              borderBottom: '1px solid var(--surface-border)',
              backgroundColor: selectedId === task.id && !manageMode ? 'var(--surface-bg2)' : 'transparent',
            }"
          >
            <!-- File Info -->
            <td :style="{ width: '200px', padding: '0 4px', verticalAlign: 'middle' }">
              <div class="flex items-center gap-2 overflow-hidden">
                <input v-if="manageMode" type="checkbox"
                  :checked="selectedIds?.has(task.id)"
                  @click.stop="emit('toggle-select', task.id)"
                  class="h-4 w-4 shrink-0"
                  :style="{ accentColor: 'var(--accent)' }"
                />
                <div v-if="!manageMode && selectedId === task.id" class="shrink-0 rounded-sm" :style="{ width: '3px', height: '28px', backgroundColor: 'var(--accent)' }"></div>
                <div class="min-w-0 flex-1 overflow-hidden">
                  <div class="flex items-center gap-1.5">
                    <Star v-if="store.isPriorityTask(task.id)" class="h-3 w-3 shrink-0 fill-current" :style="{ color: 'var(--gold)' }" />
                    <span v-if="extBadge(task.file_name || task.url)" class="shrink-0 text-2xs font-semibold tabular-nums font-song" :style="{ color: 'var(--text-secondary)' }">{{ extBadge(task.file_name || task.url) }}</span>
                    <span class="truncate text-sm font-medium" :style="{ color: 'var(--text-primary)' }" :title="task.file_name || task.url || '(准备中)'">{{ task.file_name || urlFilename(task.url) || '(准备中)' }}</span>
                  </div>
                  <div class="truncate text-2xs" :style="{ color: 'var(--text-muted)' }" :title="task.url || ''">{{ task.url || '' }}</div>
                </div>
              </div>
            </td>

            <!-- Progress -->
            <td :style="{ width: '150px', padding: '0 4px', verticalAlign: 'middle' }">
              <div class="flex items-center gap-1.5">
                <div class="h-1 rounded-full flex-1 min-w-0" :style="{ backgroundColor: 'var(--surface-bg2)' }">
                  <div class="h-full rounded-full transition-all duration-300"
                    :style="{
                      width: `${task.total_bytes > 0 ? Math.round((task.downloaded_bytes / task.total_bytes) * 100) : 0}%`,
                      backgroundColor: progressColor(task.status),
                    }"
                  />
                </div>
                <span class="shrink-0 text-xs tabular-nums text-right" style="min-width: 36px;" :style="{ color: 'var(--text-secondary)' }">
                  {{ task.total_bytes > 0 ? Math.round((task.downloaded_bytes / task.total_bytes) * 100) : 0 }}%
                </span>
              </div>
            </td>

            <!-- Downloaded size -->
            <td :style="{ width: '85px', padding: '0 4px', textAlign: 'center', verticalAlign: 'middle', color: 'var(--text-secondary)' }" class="text-xs tabular-nums">
              {{ props.formatBytes(task.downloaded_bytes) }}
            </td>

            <!-- Total size -->
            <td :style="{ width: '85px', padding: '0 4px', textAlign: 'center', verticalAlign: 'middle', color: 'var(--text-secondary)' }" class="text-xs tabular-nums">
              {{ task.total_bytes > 0 ? props.formatBytes(task.total_bytes) : '—' }}
            </td>

            <!-- Speed -->
            <td :style="{ width: '90px', padding: '0 4px', textAlign: 'center', verticalAlign: 'middle', color: task.status === 1 ? 'var(--success)' : 'var(--text-muted)' }" class="text-xs tabular-nums">
              {{ task.status === 1 && task.speed > 0 ? props.formatSpeed(task.speed) : '—' }}
            </td>

            <!-- ETA -->
            <td :style="{ width: '80px', padding: '0 4px', textAlign: 'center', verticalAlign: 'middle', color: task.status === 1 ? 'var(--text-secondary)' : 'var(--text-muted)' }" class="text-xs tabular-nums">
              {{ task.status === 1 && task.speed > 0 && task.total_bytes > 0 ? props.formatEta(task) : '—' }}
            </td>

            <!-- Status -->
            <td :style="{ width: '60px', padding: '0 4px', textAlign: 'center', verticalAlign: 'middle', color: progressColor(task.status) }" class="text-xs">
              {{ statusText(task.status) }}
            </td>

            <!-- Actions -->
            <td :style="{ width: '72px', padding: '0 4px', verticalAlign: 'middle' }">
              <div class="flex items-center justify-center gap-0.5">
                <button v-if="task.status === 2" @click.stop="store.resumeTask(task.id)" class="rounded p-1.5 transition-colors" :style="{ color: 'var(--text-muted)' }">
                  <Play class="h-3.5 w-3.5" />
                </button>
                <button v-else-if="task.status === 1" @click.stop="store.pauseTask(task.id)" class="rounded p-1.5 transition-colors" :style="{ color: 'var(--text-muted)' }">
                  <Pause class="h-3.5 w-3.5" />
                </button>
                <button @click.stop="onDeleteClick(task)" class="rounded p-1.5 transition-colors" :style="{ color: 'var(--text-muted)' }">
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Context Menu -->
    <div v-if="contextMenu.show"
      id="task-context-menu"
      :style="{
        position: 'fixed',
        left: contextMenu.x + 'px',
        top: contextMenu.y + 'px',
        zIndex: 9999,
        backgroundColor: 'var(--surface-bg)',
        border: '1px solid var(--surface-border)',
        borderRadius: '8px',
        padding: '4px',
        minWidth: '180px',
        boxShadow: '0 8px 24px rgba(0,0,0,0.6)',
      }"
    >
      <button v-if="contextMenu.task?.status === 1"
        @click="store.pauseTask(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: 'var(--text-primary)' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Pause class="h-3.5 w-3.5" /> 暂停
      </button>
      <button v-if="contextMenu.task?.status === 2"
        @click="store.resumeTask(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: 'var(--text-primary)' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Play class="h-3.5 w-3.5" /> 恢复
      </button>
      <button
        @click="store.setTaskPriority(contextMenu.task.id); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: 'var(--text-primary)' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <ArrowUp class="h-3.5 w-3.5" /> 优先下载
      </button>
      <button
        @click="showEditThreads = true; closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: 'var(--text-primary)' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <SlidersHorizontal class="h-3.5 w-3.5" /> 编辑线程数
      </button>
      <div :style="{ borderBottom: '1px solid var(--surface-border)', margin: '4px 8px' }"></div>
      <div class="relative">
        <button
          @click="showQueueSubmenu = !showQueueSubmenu"
          class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
          :style="{ color: 'var(--text-primary)' }"
          @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
          @mouseleave="$event.target.style.backgroundColor='transparent'"
        >
          <List class="h-3.5 w-3.5" /> 移动到队列 <span class="ml-auto" :style="{ color: 'var(--text-muted)' }">▸</span>
        </button>
        <!-- Queue submenu -->
        <div v-if="showQueueSubmenu"
          :style="{
            position: 'absolute',
            left: '100%',
            top: '0',
            zIndex: 10000,
            backgroundColor: 'var(--surface-bg)',
            border: '1px solid var(--surface-border)',
            borderRadius: '8px',
            padding: '4px',
            minWidth: '140px',
            boxShadow: '0 8px 24px rgba(0,0,0,0.6)',
          }"
        >
          <button
            v-for="(_, qid) in store.queueStates" :key="qid"
            @click="store.moveTaskToQueue(contextMenu.task.id, qid); showQueueSubmenu = false; closeContextMenu()"
            class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
            :style="{ color: 'var(--text-primary)' }"
            @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
            @mouseleave="$event.target.style.backgroundColor='transparent'"
          >
            {{ store.queueLabels[qid] || qid }}
          </button>
        </div>
      </div>
      <button
        @click="navigator.clipboard.writeText(contextMenu.task.url); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: 'var(--text-primary)' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Copy class="h-3.5 w-3.5" /> 复制链接
      </button>
      <button
        @click="store.revealInFolder(contextMenu.task.save_dir + '/' + contextMenu.task.file_name); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: 'var(--text-primary)' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <FolderOpen class="h-3.5 w-3.5" /> 打开文件位置
      </button>
      <div :style="{ borderBottom: '1px solid var(--surface-border)', margin: '4px 8px' }"></div>
      <button
        @click="onDeleteClick(contextMenu.task); closeContextMenu()"
        class="flex w-full items-center gap-2 rounded px-3 py-1.5 text-sm transition-colors"
        :style="{ color: '#D64531' }"
        @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
        @mouseleave="$event.target.style.backgroundColor='transparent'"
      >
        <Trash2 class="h-3.5 w-3.5" /> 删除任务
      </button>
    </div>

    <!-- Delete confirmation dialog for completed tasks -->
    <div v-if="deleteConfirmTask"
      class="fixed inset-0 z-50 flex items-center justify-center"
      :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
      @click.self="cancelDeleteTask"
    >
      <div class="w-80 rounded-xl p-5 shadow-2xl" :style="{ backgroundColor: 'var(--surface-bg)', border: '1px solid var(--surface-border)' }">
        <p class="text-sm font-medium" :style="{ color: 'var(--text-primary)' }">删除已完成的任务</p>
        <p class="mt-2 text-xs" :style="{ color: 'var(--text-secondary)' }">
          是否同时删除已下载的文件？
        </p>
        <div class="mt-5 flex justify-end gap-2">
          <button @click="cancelDeleteTask"
            class="rounded-lg px-4 py-2 text-xs font-medium transition-colors"
            :style="{ backgroundColor: 'var(--surface-bg2)', color: 'var(--text-secondary)' }"
            @mouseenter="$event.target.style.backgroundColor='var(--surface-bg3)'"
            @mouseleave="$event.target.style.backgroundColor='var(--surface-bg2)'"
          >取消</button>
          <button @click="confirmDeleteTask(false)"
            class="rounded-lg px-4 py-2 text-xs font-medium transition-colors"
            :style="{ backgroundColor: 'var(--surface-bg2)', color: 'var(--text-primary)' }"
            @mouseenter="$event.target.style.backgroundColor='var(--surface-bg3)'"
            @mouseleave="$event.target.style.backgroundColor='var(--surface-bg2)'"
          >仅删除任务</button>
          <button @click="confirmDeleteTask(true)"
            class="rounded-lg px-4 py-2 text-xs font-medium transition-colors"
            :style="{ backgroundColor: '#D64531', color: 'var(--text-primary)' }"
            @mouseenter="$event.target.style.opacity='0.9'"
            @mouseleave="$event.target.style.opacity='1'"
          >删除任务及文件</button>
        </div>
      </div>
    </div>

    <EditThreadsDialog v-if="showEditThreads" :task="contextMenu.task" @close="showEditThreads = false" />
  </div>
</template>
