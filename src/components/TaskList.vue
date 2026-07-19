<script setup lang="ts">
import { ref } from "vue";
import { useDownloadStore } from "../lib/store";
import { Play, Pause, Music, Film, File, Trash2 } from "lucide-vue-next";

const props = defineProps<{
  tasks: any[];
  formatBytes: (bytes: number) => string;
  formatSpeed: (bytes: number) => string;
  formatEta: (task: any) => string;
}>();

const store = useDownloadStore();
const selectedId = ref<string | null>(null);

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
      <div
        v-for="task in tasks" :key="task.id"
        @click="selectedId = task.id"
        @contextmenu.prevent
        :class="selectedId === task.id ? 'flex cursor-pointer items-center gap-0 px-4 transition-colors bg-selected' : 'flex cursor-pointer items-center gap-0 px-4 transition-colors hover-bg'"
        :style="{
          height: '64px',
          borderBottom: '1px solid #3A3A3C',
          backgroundColor: selectedId === task.id ? '#3A3A3C' : 'transparent',
        }"
      >
        <!-- Selected indicator -->
        <div v-if="selectedId === task.id" class="mr-3 shrink-0 rounded-sm" :style="{ width: '3px', height: '28px', backgroundColor: '#3B82F6' }"></div>

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
          {{ task.status === 1 ? '1.2 MB/s' : '—' }}
        </div>

        <!-- ETA (~80px) -->
        <div class="shrink-0 text-xs tabular-nums text-center" style="width: 80px;" :style="{ color: task.status === 1 ? '#A1A1A6' : '#8E8E93' }">
          {{ task.status === 1 ? '约 2 分钟' : '—' }}
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
  </div>
</template>
