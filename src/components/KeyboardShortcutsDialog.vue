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
  return keys.map(k => `<kbd style="background:#1A120E;border:1px solid #5A4330;border-radius:4px;padding:1px 5px;font-size:11px;color:#EDE0C8;font-family:inherit">${k}</kbd>`).join('<span style="color:#9C8260;margin:0 2px">+</span>');
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click.self="emit('close')"
  >
    <div :style="{ backgroundColor: '#261C14', border: '1px solid #5A4330', borderRadius: '12px', width: '340px' }" class="shadow-2xl">
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid #5A4330' }">
        <div class="flex items-center gap-2">
          <CmdIcon class="h-4 w-4" :style="{ color: '#9C8260' }" />
          <span class="text-sm font-semibold" :style="{ color: '#EDE0C8' }">键盘快捷键</span>
        </div>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: '#9C8260' }">
          <X class="h-4 w-4" />
        </button>
      </div>
      <div class="px-4 py-3 space-y-2">
        <div v-for="s in shortcuts" :key="s.desc"
          class="flex items-center justify-between py-1.5"
          :style="{ borderBottom: '1px solid #5A4330' }"
        >
          <span class="text-xs" :style="{ color: '#C9B393' }">{{ s.desc }}</span>
          <span v-html="kbd(s.keys)" class="text-xs"></span>
        </div>
      </div>
      <div class="px-4 py-2 text-2xs" :style="{ color: '#9C8260', borderTop: '1px solid #5A4330' }">
        Windows/Linux 用户请将 ⌘ 替换为 Ctrl
      </div>
    </div>
  </div>
</template>
