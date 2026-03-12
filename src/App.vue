<template>
  <div class="h-full w-full app-shell">
    <div class="content-shell">
    <aside class="w-[260px] shrink-0 p-4 border-r border-white/10 bg-panel/35 backdrop-blur-md flex flex-col">
      <div class="flex items-center gap-3 px-2">
        <div class="h-9 w-9 rounded-xl bg-purple-600/90 flex items-center justify-center font-bold">OC</div>
        <div class="leading-tight">
          <div class="font-semibold">OpenClaw</div>
          <div class="text-xs text-subtext">macOS Console</div>
        </div>
      </div>

      <div class="mt-6 space-y-1">
        <NavItem label="本机状态" :active="activeView === 'local'" @select="switchView('local')" />
        <NavItem label="会话面板" :active="activeView === 'sessions'" @select="switchView('sessions')" />
        <NavItem label="Agent 面板" :active="activeView === 'agents'" @select="switchView('agents')" />
        <NavItem label="Skill 面板" :active="activeView === 'skills'" @select="switchView('skills')" />
        <NavItem label="日志面板" :active="activeView === 'gatewayLogs'" @select="switchView('gatewayLogs')" />
      </div>

      <div class="mt-auto pt-6">
        <div class="p-3 rounded-xl border border-white/10 bg-white/5 text-xs text-subtext">
          macOS App Only
        </div>
      </div>
    </aside>

    <section v-if="activeView === 'sessions'" class="w-[320px] shrink-0 p-4 border-r border-white/10 bg-white/[0.02]">
      <div class="flex items-center justify-between mb-3">
        <div class="font-semibold">Sessions</div>
        <div class="flex items-center gap-2">
          <button class="btn" @click="refreshSessions">刷新列表</button>
        </div>
      </div>

      <div class="flex items-center gap-2 mb-3">
        <input v-model="sessionQuery" class="input" placeholder="Session ID..." />
      </div>

      <div class="space-y-2 overflow-auto pr-1" style="max-height: calc(100vh - 170px)">
        <div class="text-[11px] text-subtext px-1">今天</div>
        <button
          v-for="s in groupedSessions.today"
          :key="s.sessionKey"
          class="w-full text-left p-3 rounded-xl border transition"
          :class="selected?.sessionKey === s.sessionKey ? 'border-purple-500/60 bg-purple-600/10' : 'border-white/10 bg-white/5 hover:bg-white/7'"
          @click="selectSession(s)"
        >
          <div class="font-medium truncate">{{ s.label || s.sessionKey }}</div>
          <div class="text-xs text-subtext truncate">{{ s.sessionKey }}</div>
          <div class="mt-1 flex items-center justify-between text-[11px] text-subtext">
            <span>{{ sessionKindLabel(s) }}</span>
            <span>{{ formatTimeLabel(s.updatedAt) }}</span>
          </div>
        </button>
        <div v-if="groupedSessions.today.length === 0" class="text-xs text-subtext px-1">暂无</div>

        <div class="text-[11px] text-subtext px-1 pt-2">过往</div>
        <button
          v-for="s in groupedSessions.past"
          :key="s.sessionKey"
          class="w-full text-left p-3 rounded-xl border transition"
          :class="selected?.sessionKey === s.sessionKey ? 'border-purple-500/60 bg-purple-600/10' : 'border-white/10 bg-white/5 hover:bg-white/7'"
          @click="selectSession(s)"
        >
          <div class="font-medium truncate">{{ s.label || s.sessionKey }}</div>
          <div class="text-xs text-subtext truncate">{{ s.sessionKey }}</div>
          <div class="mt-1 flex items-center justify-between text-[11px] text-subtext">
            <span>{{ sessionKindLabel(s) }}</span>
            <span>{{ formatTimeLabel(s.updatedAt) }}</span>
          </div>
        </button>
        <div v-if="groupedSessions.past.length === 0" class="text-xs text-subtext px-1">暂无</div>
      </div>
    </section>

    <main v-if="activeView === 'sessions'" class="flex-1 min-w-0 p-4 bg-black/10">
      <div class="flex items-center justify-between mb-3">
        <div class="min-w-0 flex-1 max-w-[42%] mr-3">
          <div class="font-semibold">Debug Console</div>
          <div class="text-xs text-subtext truncate">Session: {{ selected?.sessionKey || '-' }}</div>
          <div v-if="lastError" class="text-xs text-red-300 mt-1 truncate max-w-[780px]">{{ lastError }}</div>
        </div>
        <div class="shrink-0 flex flex-nowrap items-center gap-2">
          <button v-if="messageView!=='chat'" class="btn" @click="messageView='chat'">对话视图</button>
          <button class="btn" @click="messageView='debug'">调试视图</button>
          <button v-if="messageView==='chat'" class="btn" @click="chatOnlyText = !chatOnlyText">
            {{ chatOnlyText ? '显示工具块' : '仅文本' }}
          </button>
          <span class="text-xs text-subtext">Auto refresh</span>
          <button class="btn" @click="togglePolling">{{ polling ? '暂停' : '继续' }}</button>
        </div>
      </div>

      <div class="rounded-2xl border border-white/10 bg-white/5 shadow-soft overflow-hidden min-w-0">
        <div class="h-[calc(100vh-220px)] overflow-auto p-4 space-y-3 min-w-0" ref="scrollRef">
          <div v-if="!selected" class="text-subtext">选择一个 session 开始查看对话。</div>
          <template v-else>
            <div
              v-for="(m, idx) in renderedMessages"
              :key="idx"
              class="flex"
              :class="messageView === 'chat' && m.role === 'user' ? 'justify-end' : 'justify-start'"
            >
              <div class="min-w-0 rounded-[16px] px-4 py-2.5 border shadow-sm"
                   :class="messageView === 'debug' ? 'w-full bubble-debug' : (m.role==='user' ? 'max-w-[72%] bubble-user' : 'max-w-[72%] bubble-assistant')">
                <template v-if="messageView === 'chat' && (m.kind === 'toolCall' || m.kind === 'toolResult')">
                  <details class="text-xs">
                    <summary class="cursor-pointer select-none">
                      {{ m.kind === 'toolCall' ? 'Tool Call' : 'Tool Result' }}: {{ m.toolName || 'tool' }}
                    </summary>
                    <pre class="mt-2 whitespace-pre-wrap break-all text-[11px] text-subtext">{{ m.toolPayload || m.text }}</pre>
                  </details>
                </template>
                <div v-else class="whitespace-pre-wrap break-all"
                     :class="messageView === 'debug' ? 'text-xs font-mono leading-5' : 'text-sm'">{{ m.text }}</div>
                <div class="text-[10px] mt-1.5 opacity-70" v-if="m.meta">{{ formatClock(m.meta) }}</div>
              </div>
            </div>
          </template>
        </div>

        <div class="p-3 border-t border-white/10 flex items-center gap-2 bg-black/10">
          <input v-model="draft" class="input flex-1" placeholder="Message to selected session... (Cmd+Enter)" @keydown.enter="send" />
          <button class="btn-primary" :disabled="sending || !selected || !draft.trim()" @click="send">
            {{ sending ? '发送中...' : '发送' }}
          </button>
        </div>
      </div>
    </main>

    <aside v-if="activeView === 'sessions'" class="w-[340px] shrink-0 p-4 border-l border-white/10 bg-white/[0.02]">
      <div class="font-semibold mb-3">Local Runtime</div>
      <div class="p-3 rounded-xl border border-white/10 bg-white/5 space-y-2">
        <InfoRow label="OPENCLAW CLI" :value="status.openclawFound ? 'FOUND' : 'MISSING'" />
        <InfoRow label="VERSION" :value="status.openclawVersion || '-'" />
        <InfoRow label="SESSIONS FILE" :value="status.sessionsFileExists ? 'READY' : 'NOT FOUND'" />
        <InfoRow label="FILE READABLE" :value="status.sessionsFileReadable ? 'YES' : 'NO'" />
        <InfoRow label="SESSIONS COUNT" :value="String(status.sessionsCount)" />
        <InfoRow label="LATEST ACTIVE" :value="formatTimeLabel(status.latestSessionUpdatedAt)" />
      </div>

      <div class="mt-3 p-3 rounded-xl border border-white/10 bg-black/20">
        <div class="text-[11px] text-subtext mb-1">OPENCLAW PATH</div>
        <div class="text-xs break-all">{{ status.openclawPath || '-' }}</div>
      </div>

      <div class="mt-3 p-3 rounded-xl border border-white/10 bg-black/20">
        <div class="text-[11px] text-subtext mb-1">RUNTIME ISSUES</div>
        <div v-if="status.errors.length === 0" class="text-xs text-green-300">No blocking issues</div>
        <div v-for="(err, i) in status.errors" :key="i" class="text-xs text-amber-300 break-words">{{ err }}</div>
      </div>

      <div class="mt-3 p-3 rounded-xl border border-white/10 bg-black/20 space-y-2">
        <InfoRow label="SELECTED KEY" :value="selected?.sessionKey || '-'" />
        <InfoRow label="SELECTED ID" :value="selected?.sessionId || '-'" />
        <InfoRow label="MODEL" :value="selected?.model || '-'" />
        <InfoRow label="PROVIDER" :value="selected?.modelProvider || '-'" />
        <InfoRow label="UPDATED" :value="formatTimeLabel(selected?.updatedAt)" />
        <div class="pt-1">
          <div class="text-[11px] text-subtext mb-1">SWITCH MODEL</div>
          <div class="flex items-center gap-2">
            <select
              v-model="selectedModelDraft"
              class="input text-xs"
              :disabled="!selected || sending || modelOptionsLoading"
            >
              <option value="">跟随当前会话</option>
              <option v-for="m in modelOptions" :key="m.value" :value="m.value">
                {{ m.label }}
              </option>
            </select>
            <button class="btn" :disabled="!selected || sending" @click="applySelectedModel">
              应用
            </button>
          </div>
        </div>
      </div>

      <div class="mt-4 font-semibold mb-3">Session Log</div>
      <div class="mb-2 flex items-center gap-2">
        <button class="btn" @click="logFilter='all'">全部</button>
        <button class="btn" @click="logFilter='tool'">工具</button>
        <button class="btn" @click="logFilter='warn'">异常</button>
        <button class="btn" @click="logFilter='info'">信息</button>
      </div>
      <div class="p-3 rounded-xl border border-white/10 bg-white/5 overflow-auto" style="max-height: calc(100vh - 420px)">
        <div v-if="filteredLogs.length===0" class="text-subtext">暂无日志（工具调用/事件将显示在这里）。</div>
        <div v-for="(l, i) in filteredLogs" :key="i" class="text-xs">
          <div class="flex items-center gap-2 py-1">
            <span class="text-subtext">{{ l.ts }}</span>
            <span class="text-green-400" v-if="l.level==='info'">•</span>
            <span class="text-purple-400" v-else-if="l.level==='tool'">•</span>
            <span class="text-amber-300" v-else>•</span>
            <span class="truncate">{{ l.text }}</span>
          </div>
        </div>
      </div>
    </aside>

    <main v-else-if="activeView === 'agents'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">Agent 面板</div>
            <div class="text-xs text-subtext">与会话面板同级，查看 agent 运行状态与活跃度</div>
          </div>
          <button class="btn" @click="refreshAgentPanel">刷新 Agent 状态</button>
        </div>

        <div class="p-4 rounded-xl border border-white/10 bg-white/5 mb-3">
          <InfoRow label="DEFAULT AGENT" :value="agentPanel.defaultAgentId || '-'" />
        </div>

        <div class="p-4 rounded-xl border border-white/10 bg-black/20">
          <div v-if="agentPanel.items.length===0" class="text-sm text-subtext">暂无 Agent 数据</div>
          <div v-for="a in agentPanel.items" :key="a.agentId" class="py-3 border-b border-white/10 last:border-b-0">
            <div class="flex items-center justify-between">
              <div class="text-sm font-medium">{{ a.agentId }}</div>
              <div class="text-xs" :class="a.running ? 'text-green-300' : 'text-subtext'">{{ a.status }}</div>
            </div>
            <div class="text-xs text-subtext mt-1">
              sessions: {{ a.sessionsCount }} | latest: {{ formatTimeLabel(a.latestUpdatedAt) }} | bootstrap: {{ a.bootstrapPending ? 'pending' : 'ready' }}
            </div>
          </div>
        </div>

        <div class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20">
          <div class="text-[11px] text-subtext mb-2">AGENT ISSUES</div>
          <div v-if="agentPanel.errors.length === 0" class="text-sm text-green-300">No blocking issues</div>
          <div v-for="(err, i) in agentPanel.errors" :key="i" class="text-sm text-amber-300 break-words py-1">{{ err }}</div>
        </div>
      </div>
    </main>

    <main v-else-if="activeView === 'skills'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">Skills</div>
            <div class="text-xs text-subtext">已安装/可用技能清单</div>
          </div>
          <button class="btn" :disabled="skillsLoading" @click="refreshSkills">
            {{ skillsLoading ? '刷新中...' : '刷新 Skills' }}
          </button>
        </div>

        <div class="grid grid-cols-3 gap-3 mb-3">
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">TOTAL</div>
            <div class="text-lg font-semibold">{{ skillsData.skills.length }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">ELIGIBLE</div>
            <div class="text-lg font-semibold">{{ skillsData.skills.filter(s => s.eligible).length }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">MISSING</div>
            <div class="text-lg font-semibold">{{ skillsData.skills.filter(s => !s.eligible).length }}</div>
          </div>
        </div>

        <div class="p-3 rounded-xl border border-white/10 bg-black/20 mb-3 flex items-center gap-2">
          <input v-model="skillsSearch" class="input max-w-[300px]" placeholder="Search skills" />
          <button class="btn" @click="skillsFilter='all'">All</button>
          <button class="btn" @click="skillsFilter='eligible'">Eligible</button>
          <button class="btn" @click="skillsFilter='missing'">Missing</button>
        </div>

        <div class="p-3 rounded-xl border border-white/10 bg-black/20">
          <div v-if="visibleSkills.length===0" class="text-sm text-subtext">No skills</div>
          <div v-for="(s, i) in visibleSkills" :key="`${s.name}-${i}`" class="py-3 border-b border-white/10 last:border-b-0">
            <div class="flex items-center justify-between gap-3">
              <div class="font-medium">{{ s.name || '-' }}</div>
              <div class="text-xs" :class="s.eligible ? 'text-green-300' : 'text-amber-300'">
                {{ s.eligible ? 'eligible' : 'missing deps' }}
              </div>
            </div>
            <div class="text-xs text-subtext mt-1">{{ s.description || '-' }}</div>
            <div class="text-[11px] text-subtext mt-1">source: {{ s.source || '-' }}</div>
          </div>
        </div>
      </div>
    </main>

    <main v-else-if="activeView === 'gatewayLogs'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">Logs</div>
            <div class="text-xs text-subtext">Gateway file logs (JSONL)</div>
          </div>
          <div class="flex items-center gap-2">
            <button class="btn" :disabled="gatewayLogsLoading" @click="refreshGatewayLogs">
              {{ gatewayLogsLoading ? '刷新中...' : 'Refresh' }}
            </button>
            <button class="btn" @click="exportVisibleGatewayLogs">Export visible</button>
          </div>
        </div>

        <div class="mb-3 text-xs text-subtext">
          更新于 {{ formatTimeLabel(String(gatewayLogs.fetchedAt || '')) }}
        </div>

        <div class="p-4 rounded-xl border border-white/10 bg-black/20 mb-3">
          <div class="text-sm mb-2">Filter</div>
          <div class="flex items-center gap-2 mb-3">
            <input v-model="gatewayLogSearch" class="input max-w-[320px]" placeholder="Search logs" />
            <label class="text-xs flex items-center gap-1">
              <input type="checkbox" v-model="gatewayLogAutoFollow" />
              Auto-follow
            </label>
          </div>
          <div class="flex items-center gap-2 text-xs">
            <label v-for="lvl in ['trace','debug','info','warn','error','fatal']" :key="lvl" class="px-2 py-1 rounded-lg border border-white/10 flex items-center gap-1">
              <input type="checkbox" v-model="gatewayLogLevelEnabled[lvl]" />
              <span :class="lvl==='warn' ? 'text-amber-300' : (lvl==='error' || lvl==='fatal' ? 'text-red-300' : 'text-subtext')">{{ lvl }}</span>
            </label>
          </div>
        </div>

        <div id="gateway-log-scroll" class="p-0 rounded-xl border border-white/10 bg-black/20 overflow-auto" style="max-height: calc(100vh - 290px)">
          <div v-if="visibleGatewayLogs.length === 0" class="text-sm text-subtext p-4">暂无日志</div>
          <div v-for="line in visibleGatewayLogs" :key="line.id" class="grid grid-cols-[90px_80px_120px_1fr] gap-3 px-3 py-2 border-b border-white/5 text-xs">
            <div class="text-subtext">{{ line.ts }}</div>
            <div>
              <span class="px-2 py-0.5 rounded-md border border-white/10"
                    :class="line.level==='warn' ? 'text-amber-300' : (line.level==='error' || line.level==='fatal' ? 'text-red-300' : 'text-blue-300')">
                {{ line.level }}
              </span>
            </div>
            <div class="text-subtext truncate">{{ line.source }}</div>
            <div class="font-mono whitespace-pre-wrap break-all">{{ line.message }}</div>
          </div>
        </div>
      </div>
    </main>

    <main v-else class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">本机状态</div>
            <div class="text-xs text-subtext">检查 OpenClaw CLI、Session 存储和运行时异常</div>
          </div>
          <div class="flex items-center gap-2">
            <button class="btn" @click="refreshLocalStatus">刷新环境</button>
            <button class="btn" :disabled="actionLoading !== ''" @click="handleRestartGateway">
              {{ actionLoading === 'restart' ? '重启中...' : '重启' }}
            </button>
            <button class="btn" :disabled="actionLoading !== ''" @click="handleDoctor">
              {{ actionLoading === 'doctor' ? '执行中...' : '诊断' }}
            </button>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="p-4 rounded-xl border border-white/10 bg-white/5 space-y-2">
            <InfoRow label="OPENCLAW CLI" :value="status.openclawFound ? 'FOUND' : 'MISSING'" />
            <InfoRow label="VERSION" :value="status.openclawVersion || '-'" />
            <InfoRow label="PATH" :value="status.openclawPath || '-'" />
            <div class="pt-2 mt-2 border-t border-white/10">
              <div class="text-[11px] text-subtext mb-1">GATEWAY PORT</div>
              <div class="flex items-center gap-2">
                <select v-model="gatewayPortDraft" class="input text-xs" @change="applyGatewayPortSelection">
                  <option value="">自动（默认）</option>
                  <option v-for="p in gatewayPorts" :key="`${p.port}-${p.pid||0}`" :value="String(p.port)">
                    {{ p.port }}{{ p.pid ? ` (pid:${p.pid})` : '' }}
                  </option>
                </select>
                <button class="btn" :disabled="gatewayPortsLoading" @click="refreshGatewayPorts">
                  {{ gatewayPortsLoading ? '刷新中...' : '刷新端口' }}
                </button>
              </div>
            </div>
          </div>
          <div class="p-4 rounded-xl border border-white/10 bg-white/5 space-y-2">
            <InfoRow label="TARGET PORT" :value="selectedGatewayPort ? String(selectedGatewayPort) : 'DEFAULT'" />
            <InfoRow label="GATEWAY STATE" :value="status.gatewayState || '-'" />
            <InfoRow label="GATEWAY RPC" :value="status.gatewayRpcOk ? 'OK' : 'FAILED'" />
            <InfoRow label="CURRENT CHANNEL" :value="status.currentChannel || '-'" />
            <InfoRow label="SESSIONS FILE" :value="status.sessionsFileExists ? 'READY' : 'NOT FOUND'" />
            <InfoRow label="FILE READABLE" :value="status.sessionsFileReadable ? 'YES' : 'NO'" />
            <InfoRow label="SESSIONS COUNT" :value="String(status.sessionsCount)" />
            <InfoRow label="LATEST ACTIVE" :value="formatTimeLabel(status.latestSessionUpdatedAt)" />
          </div>
        </div>

        <div class="mt-3 grid grid-cols-2 gap-3">
          <div class="p-4 rounded-xl border border-white/10 bg-black/20">
            <div class="text-[11px] text-subtext mb-1">CONFIGURED CHANNELS</div>
            <div v-if="status.configuredChannels.length === 0" class="text-xs text-subtext">No channel summary</div>
            <div v-for="(c, i) in status.configuredChannels" :key="i" class="text-xs py-0.5 break-all">{{ c }}</div>
          </div>
          <div class="p-4 rounded-xl border border-white/10 bg-black/20">
            <div class="text-[11px] text-subtext mb-2">RUNTIME ISSUES</div>
            <div v-if="status.errors.length === 0" class="text-sm text-green-300">No blocking issues</div>
            <div v-for="(err, i) in status.errors" :key="i" class="text-sm text-amber-300 break-words py-1">{{ err }}</div>
          </div>
        </div>

        <div v-if="actionResult" class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20 text-xs space-y-1">
          <div :class="actionResult.ok ? 'text-green-300' : 'text-amber-300'">
            {{ actionResult.ok ? 'Command success' : 'Command failed' }}: {{ actionResult.command }}
          </div>
          <div v-if="actionResult.error" class="text-amber-300 break-words">{{ actionResult.error }}</div>
          <pre v-if="actionResult.stdout" class="whitespace-pre-wrap break-words text-subtext">{{ actionResult.stdout }}</pre>
        </div>

        <div class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20">
          <div class="flex items-center justify-between mb-2">
            <div>
              <div class="text-sm font-semibold">openclaw.json</div>
              <div class="text-[11px] text-subtext break-all">{{ openclawConfig.path || '-' }}</div>
            </div>
            <div class="flex items-center gap-2">
              <button class="btn" :disabled="openclawConfigLoading || openclawConfigSaving" @click="refreshOpenclawConfig">
                {{ openclawConfigLoading ? '读取中...' : '重新加载' }}
              </button>
              <button class="btn" :disabled="openclawConfigLoading || openclawConfigSaving" @click="persistOpenclawConfig">
                {{ openclawConfigSaving ? '保存中...' : '保存修改' }}
              </button>
            </div>
          </div>
          <textarea
            v-model="openclawConfigDraft"
            class="w-full h-[280px] rounded-lg bg-black/30 border border-white/10 p-3 text-xs font-mono outline-none focus:border-purple-500/50"
            spellcheck="false"
          />
          <div v-if="openclawConfigMsg" class="text-xs mt-2" :class="openclawConfigMsg === '保存成功' ? 'text-green-300' : 'text-amber-300'">
            {{ openclawConfigMsg }}
          </div>
        </div>
      </div>
    </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import NavItem from './components/NavItem.vue'
import InfoRow from './components/InfoRow.vue'
import { getAgentPanelStatus, getGatewayLogs, getLocalStatus, getLocalStatusQuick, getOpenclawConfig, getSessionHistory, listGatewayPorts, listModels, listSessions, listSkills, restartGateway, runDoctor, saveOpenclawConfig, sendMessage, type ActionResult, type AgentPanelData, type GatewayLogs, type GatewayPortItem, type LocalStatus, type ModelOption, type OpenclawConfigPayload, type SessionItem, type SkillItem, type SkillsPayload } from './api'

const sessions = ref<SessionItem[]>([])
const selected = ref<SessionItem | null>(null)
const sessionQuery = ref('')

type ChatMessage = {
  role: string;
  text: string;
  meta?: string;
  kind?: 'text' | 'toolCall' | 'toolResult';
  toolName?: string;
  toolPayload?: string;
}

const messages = ref<ChatMessage[]>([])
const debugMessages = ref<ChatMessage[]>([])
const logs = ref<{ ts: string; level: 'info' | 'tool' | 'warn'; text: string }[]>([])
const messageView = ref<'chat' | 'debug'>('chat')
const chatOnlyText = ref(false)
const logFilter = ref<'all' | 'info' | 'tool' | 'warn'>('all')

const draft = ref('')
const sending = ref(false)
const lastError = ref<string | null>(null)
const polling = ref(false)
const activeView = ref<'sessions' | 'agents' | 'skills' | 'gatewayLogs' | 'local'>('local')
let pollTimer: ReturnType<typeof setInterval> | null = null

const status = ref<LocalStatus>({
  openclawFound: false,
  openclawVersion: '',
  openclawPath: '',
  sessionsDir: '',
  sessionsFileExists: false,
  sessionsFileReadable: false,
  sessionsCount: 0,
  latestSessionUpdatedAt: '',
  errors: [],
  gatewayState: '',
  gatewayRpcOk: false,
  gatewayRpcError: '',
  currentChannel: '',
  configuredChannels: [],
})
const actionResult = ref<ActionResult | null>(null)
const actionLoading = ref<'restart' | 'doctor' | ''>('')
const agentPanel = ref<AgentPanelData>({
  defaultAgentId: '',
  items: [],
  errors: [],
})
const gatewayLogs = ref<GatewayLogs>({ lines: [], fetchedAt: 0 })
const gatewayLogsLoading = ref(false)
const skillsData = ref<SkillsPayload>({ workspaceDir: '', managedSkillsDir: '', skills: [] })
const skillsLoading = ref(false)
const skillsSearch = ref('')
const skillsFilter = ref<'all' | 'eligible' | 'missing'>('all')
const gatewayLogSearch = ref('')
const gatewayLogAutoFollow = ref(true)
const gatewayLogLevelEnabled = ref<Record<string, boolean>>({
  trace: true,
  debug: true,
  info: true,
  warn: true,
  error: true,
  fatal: true,
})
const openclawConfig = ref<OpenclawConfigPayload>({ path: '', content: '' })
const openclawConfigDraft = ref('')
const openclawConfigLoading = ref(false)
const openclawConfigSaving = ref(false)
const openclawConfigMsg = ref('')
const selectedModelDraft = ref('')
const modelOptions = ref<ModelOption[]>([])
const modelOptionsLoading = ref(false)
const gatewayPorts = ref<GatewayPortItem[]>([])
const gatewayPortsLoading = ref(false)
const gatewayPortDraft = ref('')
const viewBootstrapped = ref<Record<string, boolean>>({
  sessions: false,
  agents: false,
  skills: false,
  gatewayLogs: false,
  local: true,
})

const scrollRef = ref<HTMLElement | null>(null)
const selectedGatewayPort = computed(() => {
  const n = Number(gatewayPortDraft.value)
  return Number.isFinite(n) && n > 0 ? n : undefined
})

const filteredSessions = computed(() => {
  const q = sessionQuery.value.trim().toLowerCase()
  if (!q) return sessions.value
  return sessions.value.filter(s =>
    (s.sessionKey || '').toLowerCase().includes(q) ||
    (s.sessionId || '').toLowerCase().includes(q) ||
    (s.label || '').toLowerCase().includes(q),
  )
})

const dedupedSessions = computed(() => {
  const map = new Map<string, SessionItem>()
  for (const s of filteredSessions.value) {
    const id = (s.sessionId || '').trim()
    const key = id || s.sessionKey
    const prev = map.get(key)
    if (!prev) {
      map.set(key, s)
      continue
    }
    const prevTs = Number(prev.updatedAt || 0)
    const curTs = Number(s.updatedAt || 0)
    if (curTs >= prevTs) map.set(key, s)
  }
  return Array.from(map.values())
})

function isToday(raw?: string) {
  if (!raw) return false
  const n = Number(raw)
  const d = Number.isFinite(n) ? new Date(n) : new Date(raw)
  if (Number.isNaN(d.getTime())) return false
  const now = new Date()
  return d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate()
}

const groupedSessions = computed(() => {
  const today: SessionItem[] = []
  const past: SessionItem[] = []
  for (const s of dedupedSessions.value) {
    if (isToday(s.updatedAt)) today.push(s)
    else past.push(s)
  }
  return { today, past }
})

const filteredLogs = computed(() => {
  if (logFilter.value === 'all') return logs.value
  return logs.value.filter(l => l.level === logFilter.value)
})

const displayedMessages = computed(() => {
  if (messageView.value === 'debug') return debugMessages.value
  if (!chatOnlyText.value) return messages.value
  return messages.value.filter(m => m.kind !== 'toolCall' && m.kind !== 'toolResult')
})

const renderedMessages = computed(() => {
  // Keep DOM size bounded to avoid panel switch jank.
  const max = messageView.value === 'debug' ? 40 : 80
  const list = displayedMessages.value
  if (list.length <= max) return list
  return list.slice(list.length - max)
})

const parsedGatewayLogs = computed(() => {
  return gatewayLogs.value.lines.map((line, idx) => parseGatewayLine(line, idx))
})

const visibleGatewayLogs = computed(() => {
  const q = gatewayLogSearch.value.trim().toLowerCase()
  return parsedGatewayLogs.value.filter((it) => {
    if (!gatewayLogLevelEnabled.value[it.level]) return false
    if (!q) return true
    return (
      it.source.toLowerCase().includes(q) ||
      it.message.toLowerCase().includes(q) ||
      it.raw.toLowerCase().includes(q)
    )
  })
})

const visibleSkills = computed(() => {
  const q = skillsSearch.value.trim().toLowerCase()
  return (skillsData.value.skills || []).filter((s: SkillItem) => {
    if (skillsFilter.value === 'eligible' && !s.eligible) return false
    if (skillsFilter.value === 'missing') {
      const m = s.missing || {}
      const hasMissing = (m.bins?.length || 0) + (m.anyBins?.length || 0) + (m.env?.length || 0) + (m.config?.length || 0) + (m.os?.length || 0) > 0
      if (!hasMissing) return false
    }
    if (!q) return true
    return `${s.name || ''} ${s.description || ''} ${s.source || ''}`.toLowerCase().includes(q)
  })
})

function sessionKindLabel(s: SessionItem) {
  if (s.kind && s.kind.trim()) return s.kind
  if (s.sessionKey.includes(':cron:')) return 'cron'
  return 'direct'
}

function parseGatewayLine(line: string, idx: number) {
  const raw = line ?? ''
  try {
    const j = JSON.parse(raw)
    const levelRaw = String(j.logLevelName || j.level || j.severity || 'info').toLowerCase()
    const level = ['trace', 'debug', 'info', 'warn', 'error', 'fatal'].includes(levelRaw) ? levelRaw : 'info'
    const source = String(j.name || j.module || j.subsystem || j.fileName || '-')
    const message = String(
      j.message ??
      j.msg ??
      j.text ??
      (j.error ? JSON.stringify(j.error) : raw),
    )
    const t = String(j.time || j.timestamp || '')
    const ts = t ? formatClock(t) : '--:--:--'
    return { id: idx, ts, level, source, message, raw }
  } catch {
    const lower = raw.toLowerCase()
    const level = lower.includes('error') ? 'error' : (lower.includes('warn') ? 'warn' : 'info')
    return { id: idx, ts: '--:--:--', level, source: '-', message: raw, raw }
  }
}

function exportVisibleGatewayLogs() {
  const content = visibleGatewayLogs.value.map(x => x.raw).join('\n')
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `gateway-logs-${Date.now()}.log`
  a.click()
  URL.revokeObjectURL(url)
}

async function refreshLocalStatus() {
  status.value = await getLocalStatus(selectedGatewayPort.value)
}

async function refreshAgentPanel() {
  agentPanel.value = await getAgentPanelStatus(selectedGatewayPort.value)
}

async function refreshGatewayLogs() {
  gatewayLogsLoading.value = true
  try {
    gatewayLogs.value = await getGatewayLogs(120, selectedGatewayPort.value)
    if (gatewayLogAutoFollow.value) {
      nextTick(() => {
        const el = document.getElementById('gateway-log-scroll')
        if (el) el.scrollTop = el.scrollHeight
      })
    }
  } finally {
    gatewayLogsLoading.value = false
  }
}

async function refreshSkills() {
  skillsLoading.value = true
  try {
    skillsData.value = await listSkills(selectedGatewayPort.value)
  } finally {
    skillsLoading.value = false
  }
}

async function refreshModelOptions() {
  modelOptionsLoading.value = true
  try {
    modelOptions.value = await listModels(selectedGatewayPort.value)
  } finally {
    modelOptionsLoading.value = false
  }
}

async function refreshGatewayPorts() {
  gatewayPortsLoading.value = true
  try {
    gatewayPorts.value = await listGatewayPorts()
    if (gatewayPortDraft.value && !gatewayPorts.value.some((p) => String(p.port) === gatewayPortDraft.value)) {
      gatewayPortDraft.value = ''
    }
    const first = gatewayPorts.value[0]
    if (!gatewayPortDraft.value && first) {
      gatewayPortDraft.value = String(first.port)
      applyGatewayPortSelection()
    }
  } finally {
    gatewayPortsLoading.value = false
  }
}

function applyGatewayPortSelection() {
  refreshLocalStatus().catch(() => {})
  if (activeView.value === 'agents') refreshAgentPanel().catch(() => {})
  if (activeView.value === 'skills') refreshSkills().catch(() => {})
  if (activeView.value === 'gatewayLogs') refreshGatewayLogs().catch(() => {})
  if (activeView.value === 'sessions') {
    refreshModelOptions().catch(() => {})
    refreshSessions().catch(() => {})
  }
}

function sessionModelValue(s: SessionItem | null) {
  if (!s) return ''
  const provider = (s.modelProvider || '').trim()
  const model = (s.model || '').trim()
  if (provider && model) return `${provider}/${model}`
  return ''
}

async function refreshOpenclawConfig() {
  openclawConfigLoading.value = true
  openclawConfigMsg.value = ''
  try {
    const data = await getOpenclawConfig()
    openclawConfig.value = data
    openclawConfigDraft.value = data.content
  } catch (e: any) {
    openclawConfigMsg.value = e?.message ?? String(e)
  } finally {
    openclawConfigLoading.value = false
  }
}

async function persistOpenclawConfig() {
  openclawConfigSaving.value = true
  openclawConfigMsg.value = ''
  try {
    const data = await saveOpenclawConfig(openclawConfigDraft.value)
    openclawConfig.value = data
    openclawConfigDraft.value = data.content
    openclawConfigMsg.value = '保存成功'
    refreshLocalStatus().catch(() => {})
  } catch (e: any) {
    openclawConfigMsg.value = e?.message ?? String(e)
  } finally {
    openclawConfigSaving.value = false
  }
}

function switchView(v: 'sessions' | 'agents' | 'skills' | 'gatewayLogs' | 'local') {
  activeView.value = v
  if (v !== 'sessions' && polling.value) {
    polling.value = false
    stopPolling()
  }
  if (viewBootstrapped.value[v]) return
  viewBootstrapped.value[v] = true
  // defer heavy data loading so the panel switch paints first
  setTimeout(() => {
    if (v === 'sessions' && modelOptions.value.length === 0 && !modelOptionsLoading.value) {
      refreshModelOptions().catch(() => {})
    }
    if (v === 'agents' && agentPanel.value.items.length === 0) {
      refreshAgentPanel().catch(() => {})
    }
    if (v === 'skills' && skillsData.value.skills.length === 0) {
      refreshSkills().catch(() => {})
    }
    if (v === 'gatewayLogs' && gatewayLogs.value.lines.length === 0) {
      refreshGatewayLogs().catch(() => {})
    }
    if (v === 'local' && !openclawConfig.value.path && !openclawConfigLoading.value) {
      refreshOpenclawConfig().catch(() => {})
    }
  }, 0)
}

async function refreshSessions() {
  lastError.value = null
  sessions.value = await listSessions()
  if (selected.value) {
    const hit = sessions.value.find(s => s.sessionKey === selected.value?.sessionKey)
    if (hit) {
      selected.value = hit
      if (!selectedModelDraft.value.trim()) {
        selectedModelDraft.value = sessionModelValue(hit)
      }
    }
  }
}

function toText(content: any): string {
  if (content == null) return ''
  if (typeof content === 'string') return content
  if (Array.isArray(content)) return content.map(toText).join('\n')
  if (typeof content === 'object') {
    if (content.text) return String(content.text)
    if (content.type === 'text' && content.text) return String(content.text)
    if (content.content) return toText(content.content)
    return JSON.stringify(content, null, 2)
  }
  return String(content)
}

function toChatText(content: any): string {
  if (content == null) return ''
  if (typeof content === 'string') return content
  if (Array.isArray(content)) {
    return content.map(toChatText).filter(Boolean).join('\n')
  }
  if (typeof content === 'object') {
    if (typeof content.text === 'string') return content.text
    if (content.type === 'text' && typeof content.text === 'string') return content.text
    if (content.content != null) return toChatText(content.content)
    return ''
  }
  return ''
}

function formatTimeLabel(raw?: string) {
  if (!raw) return '-'
  const n = Number(raw)
  const d = Number.isFinite(n) ? new Date(n) : new Date(raw)
  if (Number.isNaN(d.getTime())) return String(raw)
  return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`
}

function formatClock(raw?: string) {
  if (!raw) return ''
  const d = new Date(raw)
  if (Number.isNaN(d.getTime())) return String(raw).slice(11, 19)
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

async function quickRefresh() {
  await Promise.all([
    refreshSessions(),
    refreshLocalStatus(),
    refreshAgentPanel(),
    refreshGatewayLogs(),
    refreshHistory(),
  ])
}

async function handleRestartGateway() {
  try {
    actionLoading.value = 'restart'
    actionResult.value = await restartGateway(selectedGatewayPort.value)
    await refreshLocalStatus()
  } finally {
    actionLoading.value = ''
  }
}

async function handleDoctor() {
  try {
    actionLoading.value = 'doctor'
    actionResult.value = await runDoctor(selectedGatewayPort.value)
    await refreshLocalStatus()
  } finally {
    actionLoading.value = ''
  }
}

function onWindowKeydown(e: KeyboardEvent) {
  if (e.metaKey && e.key.toLowerCase() === 'r') {
    e.preventDefault()
    quickRefresh().catch(() => {})
    return
  }
  if (e.metaKey && e.key === 'Enter') {
    e.preventDefault()
    send().catch(() => {})
    return
  }
}

async function refreshHistory() {
  if (!selected.value) return

  lastError.value = null
  const items = await getSessionHistory(selected.value.sessionKey, 120)

  const nextMsgs: ChatMessage[] = []
  const nextDebugMsgs: ChatMessage[] = []
  const nextLogs: { ts: string; level: 'info' | 'tool' | 'warn'; text: string }[] = []

  for (const it of items) {
    const typ = it.type || it.customType
    const tsRaw = it.timestamp || it.ts || it.createdAt || ''
    const ts = tsRaw ? String(tsRaw).slice(11, 19) : '--:--:--'
    const msgNode = it.message ?? null
    const innerType = msgNode?.type || msgNode?.customType
    const isToolCall = typ === 'toolCall' || typ === 'tool_call' || innerType === 'toolCall' || innerType === 'tool_call'
    const isToolResult = typ === 'toolResult' || typ === 'tool_result' || innerType === 'toolResult' || innerType === 'tool_result'
    nextDebugMsgs.push({
      role: 'assistant',
      text: typeof it === 'object' ? JSON.stringify(it, null, 2) : String(it),
      meta: it.timestamp || it.ts,
    })

    if (isToolCall) {
      const toolName = it.name || it.toolName || msgNode?.name || msgNode?.toolName || 'tool'
      nextMsgs.push({
        role: 'assistant',
        text: `${toolName} call`,
        meta: it.timestamp,
        kind: 'toolCall',
        toolName,
        toolPayload: JSON.stringify(it, null, 2),
      })
      nextLogs.push({ ts, level: 'tool', text: `${toolName} call` })
      continue
    }
    if (isToolResult) {
      const toolName = it.name || it.toolName || msgNode?.name || msgNode?.toolName || 'tool'
      nextMsgs.push({
        role: 'assistant',
        text: `${toolName} result`,
        meta: it.timestamp,
        kind: 'toolResult',
        toolName,
        toolPayload: JSON.stringify(it, null, 2),
      })
      nextLogs.push({ ts, level: 'tool', text: `${toolName} result` })
      continue
    }
    if (typ === 'error' || it.error || it.err || it.failed) {
      const errText = toText(it.error ?? it.err ?? it.message ?? it)
      nextLogs.push({ ts, level: 'warn', text: errText.slice(0, 240) })
      continue
    }
    if (typ === 'user') {
      nextMsgs.push({ role: 'user', text: toText(it.text ?? it.content ?? it), meta: it.timestamp, kind: 'text' })
      continue
    }
    if (typ === 'assistant') {
      nextMsgs.push({ role: 'assistant', text: toChatText(it.text ?? it.content ?? it), meta: it.timestamp, kind: 'text' })
      continue
    }
    if (typ === 'message' || it.message) {
      const msg = it.message ?? it
      const role = String(msg.role || it.role || 'assistant')
      const content = msg.content ?? msg.text ?? it.content ?? it.text ?? it
      nextMsgs.push({ role, text: toChatText(content), meta: it.timestamp || msg.timestamp, kind: 'text' })
      continue
    }

    // More tolerant event->log mapping so Session Log actually shows signal.
    if (typ && typ !== 'message') {
      const name = it.name || it.toolName || it.event || it.kind || typ
      nextLogs.push({ ts, level: 'info', text: String(name) })
      continue
    }
    if (it.toolName || it.name) {
      nextLogs.push({ ts, level: 'tool', text: `${it.toolName || it.name}` })
      continue
    }

    nextMsgs.push({ role: 'assistant', text: toText(it), meta: it.timestamp, kind: 'text' })
  }

  messages.value = nextMsgs
  debugMessages.value = nextDebugMsgs
  logs.value = nextLogs

  await nextTick()
  if (scrollRef.value) {
    scrollRef.value.scrollTop = scrollRef.value.scrollHeight
  }
}

async function selectSession(s: SessionItem) {
  selected.value = s
  selectedModelDraft.value = sessionModelValue(s)
  await refreshHistory()
}

function applySelectedModel() {
  if (!selected.value) return
  const next = selectedModelDraft.value.trim()
  const [provider, ...modelParts] = next.split('/')
  const model = modelParts.join('/').trim()
  const patched: SessionItem = {
    ...selected.value,
    model: model || selected.value.model,
    modelProvider: provider || selected.value.modelProvider,
  }
  selected.value = patched
  sessions.value = sessions.value.map((it) =>
    it.sessionKey === patched.sessionKey
      ? { ...it, model: patched.model, modelProvider: patched.modelProvider }
      : it,
  )
}

async function send() {
  if (!selected.value) return
  const msg = draft.value.trim()
  if (!msg) return

  try {
    sending.value = true
    lastError.value = null
    draft.value = ''
    const model = selectedModelDraft.value.trim() || undefined
    await sendMessage(selected.value.sessionKey, msg, model, selectedGatewayPort.value)
    if (model && selected.value) {
      const [provider, ...modelParts] = model.split('/')
      const modelName = modelParts.join('/').trim()
      selected.value = { ...selected.value, model: modelName || selected.value.model, modelProvider: provider || selected.value.modelProvider }
      sessions.value = sessions.value.map((it) =>
        it.sessionKey === selected.value?.sessionKey
          ? { ...it, model: selected.value?.model, modelProvider: selected.value?.modelProvider }
          : it,
      )
    }
    await refreshHistory()
  } catch (e: any) {
    lastError.value = e?.message ?? String(e)
  } finally {
    sending.value = false
  }
}

function startPolling() {
  stopPolling()
  pollTimer = setInterval(() => {
    if (!polling.value || activeView.value !== 'sessions') return

    refreshSessions().catch(() => {})
    refreshHistory().catch(() => {})
  }, 1500)
}

function stopPolling() {
  if (pollTimer) clearInterval(pollTimer)
  pollTimer = null
}

function togglePolling() {
  polling.value = !polling.value
  if (polling.value) startPolling()
  else stopPolling()
}

onMounted(() => {
  window.addEventListener('keydown', onWindowKeydown)
  refreshGatewayPorts().catch(() => {})

  // Fast startup: render local view quickly with lightweight status.
  getLocalStatusQuick(selectedGatewayPort.value)
    .then((v) => { status.value = v })
    .catch((e: any) => { lastError.value = e?.message ?? String(e) })

  // Defer heavier calls so first paint stays responsive.
  requestAnimationFrame(() => {
    setTimeout(() => {
      refreshLocalStatus().catch(() => {})
      refreshModelOptions().catch(() => {})
      refreshSessions().catch(() => {})
      if (activeView.value === 'local') refreshOpenclawConfig().catch(() => {})
    }, 1200)
  })
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onWindowKeydown)
  stopPolling()
})
</script>

<style scoped>
.app-shell {
  background: linear-gradient(180deg, rgba(21, 23, 38, 0.88) 0%, rgba(9, 10, 17, 0.94) 100%);
}

.content-shell {
  height: 100vh;
  display: flex;
}

.btn{ @apply text-xs px-3 py-1.5 rounded-lg border border-white/10 bg-white/5 hover:bg-white/10 transition-colors; }
.btn-primary{ @apply text-xs px-3 py-2 rounded-lg bg-purple-600 hover:bg-purple-500 disabled:opacity-50 disabled:cursor-not-allowed; }
.input{ @apply w-full text-sm px-3 py-2 rounded-lg bg-black/25 border border-white/10 outline-none focus:border-purple-500/50; }

.bubble-user {
  background: linear-gradient(180deg, rgba(124, 58, 237, 0.35) 0%, rgba(124, 58, 237, 0.22) 100%);
  border-color: rgba(167, 139, 250, 0.4);
}

.bubble-assistant {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.09) 0%, rgba(255, 255, 255, 0.04) 100%);
  border-color: rgba(255, 255, 255, 0.14);
}

.bubble-debug {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.07) 0%, rgba(255, 255, 255, 0.03) 100%);
  border-color: rgba(255, 255, 255, 0.12);
}
</style>
