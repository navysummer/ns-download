<script setup lang="ts">
import { X, Command as CmdIcon } from "lucide-vue-next";

defineEmits<{ close: [] }>();

const shortcuts = [
  { keys: ["⌘", "N"], desc: "新建下载" },
  { keys: ["⌘", "F"], desc: "搜索任务" },
  { keys: ["⌘", "A"], desc: "全选（管理模式下）" },
  { keys: ["⌫"], desc: "删除选中任务" },
  { keys: ["Space"], desc: "暂停/恢复选中任务" },
  { keys: ["Esc"], desc: "关闭弹窗 / 退出搜索" },
  { keys: ["⌘", "K"], desc: "打开快捷键参考" },
];

function kbd(keys: string[]) {
  return keys.map(k => `<kbd style="background:var(--app-bg);border:1px solid var(--surface-border);border-radius:4px;padding:1px 5px;font-size:11px;color:var(--text-primary);font-family:inherit">${k}</kbd>`).join('<span style="color:var(--text-muted);margin:0 2px">+</span>');
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click.self="emit('close')"
  >
    <div :style="{ backgroundColor: 'var(--surface-bg)', border: '1px solid var(--surface-border)', borderRadius: '12px', width: '340px' }" class="shadow-2xl">
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--surface-border)' }">
        <div class="flex items-center gap-2">
          <CmdIcon class="h-4 w-4" :style="{ color: 'var(--text-muted)' }" />
          <span class="text-sm font-semibold" :style="{ color: 'var(--text-primary)' }">键盘快捷键</span>
        </div>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: 'var(--text-muted)' }">
          <X class="h-4 w-4" />
        </button>
      </div>
      <div class="px-4 py-3 space-y-2">
        <div v-for="s in shortcuts" :key="s.desc"
          class="flex items-center justify-between py-1.5"
          :style="{ borderBottom: '1px solid var(--surface-border)' }"
        >
          <span class="text-xs" :style="{ color: 'var(--text-secondary)' }">{{ s.desc }}</span>
          <span v-html="kbd(s.keys)" class="text-xs"></span>
        </div>
      </div>
      <div class="px-4 py-2 text-2xs" :style="{ color: 'var(--text-muted)', borderTop: '1px solid var(--surface-border)' }">
        Windows/Linux 用户请将 ⌘ 替换为 Ctrl
      </div>
    </div>
  </div>
</template>
