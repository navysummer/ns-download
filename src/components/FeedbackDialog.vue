<script setup lang="ts">
import { ref } from "vue";
import { X, Send } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{ close: [] }>();

const feedbackType = ref("suggestion");
const message = ref("");
const contact = ref("");
const sent = ref(false);

async function submit() {
  if (!message.value.trim()) return;
  const feedback = {
    feedback_type: feedbackType.value,
    message: message.value,
    contact: contact.value,
    time: new Date().toISOString(),
  };
  try {
    await invoke("submit_feedback", { feedback });
  } catch (e) {
    console.error("Failed to submit feedback:", e);
    // Fallback: persist to localStorage
    try {
      const existing = JSON.parse(localStorage.getItem('ns_feedback') || '[]');
      existing.push(feedback);
      localStorage.setItem('ns_feedback', JSON.stringify(existing));
    } catch (e2) {
      console.error("Failed to save feedback fallback:", e2);
    }
  }
  sent.value = true;
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click.self="emit('close')"
  >
    <div :style="{ backgroundColor: '#2C2C2E', border: '1px solid #48484A', borderRadius: '12px', width: '360px' }"
      class="shadow-2xl"
    >
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid #48484A' }">
        <span class="text-sm font-semibold" :style="{ color: '#F5F5F7' }">反馈</span>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: '#8E8E93' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <template v-if="sent">
        <div class="flex flex-col items-center justify-center py-8 gap-2">
          <span class="text-lg" :style="{ color: '#22C55E' }">✓</span>
          <span class="text-sm font-medium" :style="{ color: '#F5F5F7' }">感谢您的反馈！</span>
          <span class="text-xs" :style="{ color: '#8E8E93' }">我们会在后续版本中考虑您的建议</span>
        </div>
      </template>
      <template v-else>
        <div class="px-4 py-4 space-y-3">
          <div>
            <span class="text-xs font-medium" :style="{ color: '#A1A1A6' }">类型</span>
            <div class="flex gap-2 mt-1">
              <button v-for="t in [{id:'bug',label:'Bug'},{id:'suggestion',label:'建议'},{id:'other',label:'其他'}]" :key="t.id"
                @click="feedbackType = t.id"
                class="rounded-md px-3 py-1.5 text-xs transition-colors"
                :style="{
                  backgroundColor: feedbackType === t.id ? 'var(--accent)' : '#3A3A3C',
                  color: feedbackType === t.id ? '#fff' : '#A1A1A6',
                }"
              >{{ t.label }}</button>
            </div>
          </div>
          <div>
            <span class="text-xs font-medium" :style="{ color: '#A1A1A6' }">内容</span>
            <textarea v-model="message" rows="4" placeholder="请描述您的反馈…"
              class="mt-1 w-full rounded-md px-3 py-2 text-sm outline-none resize-none"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }"
            ></textarea>
          </div>
          <div>
            <span class="text-xs font-medium" :style="{ color: '#A1A1A6' }">联系方式 (可选)</span>
            <input v-model="contact" placeholder="邮箱 / 社交账号"
              class="mt-1 w-full rounded-md px-3 py-2 text-sm outline-none"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }"
            />
          </div>
        </div>
        <div class="flex justify-end px-4 py-3" :style="{ borderTop: '1px solid #48484A' }">
          <button @click="submit" class="flex items-center gap-1.5 rounded-md px-4 py-1.5 text-sm font-medium transition-colors"
            :style="{ backgroundColor: 'var(--accent)', color: '#fff' }">
            <Send class="h-3.5 w-3.5" /> 发送
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
