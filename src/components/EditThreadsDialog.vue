<script setup lang="ts">
import { ref } from "vue";
import { X } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{ task: any }>();
const emit = defineEmits<{ close: [] }>();

const threadCount = ref(props.task.segments || 1);
const saving = ref(false);

async function submit() {
  saving.value = true;
  try {
    await invoke("set_task_segments", { taskId: props.task.id, segments: Math.max(1, Math.min(128, threadCount.value)) });
  } catch (e) {
    console.error("Failed to set segments:", e);
  }
  saving.value = false;
  emit("close");
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click="emit('close')"
  >
    <div :style="{ backgroundColor: 'var(--surface-bg)', border: '1px solid var(--surface-border)', borderRadius: '12px', width: '300px' }"
      class="shadow-2xl @click.stop"
    >
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--surface-border)' }">
        <span class="text-sm font-semibold spacing-2" :style="{ color: 'var(--text-primary)' }">编辑下载线程数</span>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: 'var(--text-muted)' }">
          <X class="h-4 w-4" />
        </button>
      </div>
      <div class="px-4 py-4">
        <div class="text-xs mb-2" :style="{ color: 'var(--text-secondary)' }">
          为 "{{ task.file_name || task.url.split('/').pop() }}" 设置线程数
        </div>
        <div class="flex items-center gap-3">
          <input v-model.number="threadCount" type="number" min="1" max="128"
            class="w-full rounded-md bg-transparent px-3 py-2 text-sm outline-none tabular-nums"
            :style="{ color: 'var(--text-primary)', border: '1px solid var(--surface-border)' }"
          />
          <span class="text-xs" :style="{ color: 'var(--text-muted)' }">线程</span>
        </div>
      </div>
      <div class="flex justify-end gap-2 px-4 py-3" :style="{ borderTop: '1px solid var(--surface-border)' }">
        <button @click="emit('close')" class="rounded-md px-3 py-1.5 text-sm transition-colors"
          :style="{ color: 'var(--text-secondary)' }">取消</button>
        <button @click="submit" class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
          :style="{ backgroundColor: 'var(--accent)', color: 'var(--text-primary)' }">应用</button>
      </div>
    </div>
  </div>
</template>
