<script setup lang="ts">
import { ref, computed } from "vue";
import { X, Download, FileIcon, CheckSquare, Square, HardDrive } from "lucide-vue-next";

interface FileEntry {
  index: number;
  path: string;
  size: number;
}

interface TorrentMeta {
  name: string;
  total_bytes: number;
  files: FileEntry[];
}

const props = defineProps<{
  meta: TorrentMeta;
  formatBytes: (bytes: number) => string;
}>();

const emit = defineEmits<{ close: []; confirm: [selectedIndices: number[]] }>();

const selected = ref<Set<number>>(new Set(props.meta.files.map((_, i) => i)));
const allSelected = computed(() => selected.value.size === props.meta.files.length);

function toggleAll() {
  if (allSelected.value) {
    selected.value = new Set();
  } else {
    selected.value = new Set(props.meta.files.map((_, i) => i));
  }
}

function toggleFile(index: number) {
  const s = new Set(selected.value);
  if (s.has(index)) s.delete(index); else s.add(index);
  selected.value = s;
}

const selectedBytes = computed(() =>
  props.meta.files.reduce((acc, f) => selected.value.has(f.index) ? acc + f.size : acc, 0)
);
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center" :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }" @click.self="emit('close')">
    <div class="flex w-full max-w-lg flex-col rounded-xl shadow-2xl" :style="{ backgroundColor: 'var(--surface-bg)', border: '1px solid var(--surface-border)' }">
      <div class="flex items-center justify-between px-5 py-4" :style="{ borderBottom: '1px solid var(--surface-border)' }">
        <div class="flex items-center gap-2.5 min-w-0">
          <div class="flex h-7 w-7 items-center justify-center rounded shrink-0" :style="{ backgroundColor: 'rgba(var(--accent-rgb),0.18)' }">
            <HardDrive class="h-3.5 w-3.5" :style="{ color: 'var(--accent)' }" />
          </div>
          <div class="min-w-0">
            <h2 class="text-sm font-semibold truncate" :style="{ color: 'var(--text-primary)' }">{{ meta.name }}</h2>
            <span class="text-2xs" :style="{ color: 'var(--text-muted)' }">{{ formatBytes(meta.total_bytes) }} · {{ meta.files.length }} 个文件</span>
          </div>
        </div>
        <button @click="emit('close')" class="rounded p-1 transition-colors shrink-0" :style="{ color: 'var(--text-muted)' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="overflow-y-auto px-4 py-3" style="max-height: 50vh;">
        <div class="flex items-center gap-2 px-2 py-1.5 rounded mb-1 cursor-pointer" :style="{ color: 'var(--text-secondary)' }"
          @click="toggleAll">
          <component :is="allSelected ? CheckSquare : Square" class="h-4 w-4" />
          <span class="text-xs font-medium">全选/取消</span>
          <span class="ml-auto text-2xs tabular-nums">{{ formatBytes(selectedBytes) }} / {{ formatBytes(meta.total_bytes) }}</span>
        </div>
        <div :style="{ borderBottom: '1px solid var(--surface-border)', margin: '4px 0' }"></div>
        <div v-for="file in meta.files" :key="file.index"
          class="flex items-center gap-2 rounded px-2 py-1.5 cursor-pointer transition-colors"
          :style="{ color: selected.has(file.index) ? 'var(--text-primary)' : 'var(--text-muted)' }"
          @click="toggleFile(file.index)"
          @mouseenter="$event.currentTarget.style.backgroundColor='var(--surface-bg2)'"
          @mouseleave="$event.currentTarget.style.backgroundColor='transparent'"
        >
          <component :is="selected.has(file.index) ? CheckSquare : Square" class="h-3.5 w-3.5 shrink-0" />
          <span class="text-xs truncate flex-1">{{ file.path }}</span>
          <span class="text-2xs tabular-nums shrink-0">{{ formatBytes(file.size) }}</span>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 px-5 py-3" :style="{ borderTop: '1px solid var(--surface-border)' }">
        <span v-if="selected.size > 0" class="text-2xs mr-auto" :style="{ color: 'var(--text-muted)' }">
          已选择 {{ selected.size }} 个文件 ({{ formatBytes(selectedBytes) }})
        </span>
        <button @click="emit('close')" class="rounded-md px-4 py-1.5 text-xs transition-colors"
          :style="{ color: 'var(--text-secondary)' }"
          @mouseenter="$event.target.style.backgroundColor='var(--surface-bg2)'"
          @mouseleave="$event.target.style.backgroundColor='transparent'"
        >取消</button>
        <button @click="emit('confirm', Array.from(selected))" class="rounded-md px-4 py-1.5 text-xs transition-colors"
          :style="{ backgroundColor: 'var(--accent)', color: 'var(--text-primary)' }"
          @mouseenter="$event.target.style.opacity='0.9'"
          @mouseleave="$event.target.style.opacity='1'"
        >
          <Download class="h-3 w-3 inline mr-1" />下载选中文件
        </button>
      </div>
    </div>
  </div>
</template>
