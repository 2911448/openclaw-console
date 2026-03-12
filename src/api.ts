export type SessionItem = {
  sessionKey: string;
  sessionId: string;
  label?: string;
  kind?: string;
  model?: string;
  modelProvider?: string;
  updatedAt?: string;
};

export type LocalStatus = {
  openclawFound: boolean;
  openclawVersion?: string;
  openclawPath?: string;
  sessionsDir: string;
  sessionsFileExists: boolean;
  sessionsFileReadable: boolean;
  sessionsCount: number;
  latestSessionUpdatedAt?: string;
  errors: string[];
  gatewayState?: string;
  gatewayRpcOk: boolean;
  gatewayRpcError?: string;
  currentChannel?: string;
  configuredChannels: string[];
};

export type ActionResult = {
  ok: boolean;
  command: string;
  stdout: string;
  stderr: string;
  error?: string;
};

export type AgentItem = {
  agentId: string;
  sessionsCount: number;
  latestUpdatedAt?: string;
  sessionsPath: string;
};

export type AgentStatusItem = {
  agentId: string;
  running: boolean;
  status: string;
  sessionsCount: number;
  latestUpdatedAt?: string;
  bootstrapPending: boolean;
};

export type AgentPanelData = {
  defaultAgentId?: string;
  items: AgentStatusItem[];
  errors: string[];
};

export type GatewayLogs = {
  lines: string[];
  fetchedAt: number;
};

export type OpenclawConfigPayload = {
  path: string;
  content: string;
};

export type SkillItem = {
  name?: string;
  description?: string;
  eligible?: boolean;
  disabled?: boolean;
  source?: string;
  bundled?: boolean;
  missing?: {
    bins?: string[];
    anyBins?: string[];
    env?: string[];
    config?: string[];
    os?: string[];
  };
};

export type SkillsPayload = {
  workspaceDir?: string;
  managedSkillsDir?: string;
  skills: SkillItem[];
};

export type ModelOption = {
  provider: string;
  model: string;
  value: string;
  label: string;
  source: string;
};

export type GatewayPortItem = {
  port: number;
  pid?: number;
  command?: string;
};

type TauriCore = {
  invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
};

function getInvoke() {
  const w = window as typeof window & {
    __TAURI__?: { core?: TauriCore };
  };
  const invoke = w.__TAURI__?.core?.invoke;
  if (!invoke) {
    throw new Error('Tauri runtime not found. Please run as macOS app via `npm run dev`.');
  }
  return invoke as TauriCore['invoke'];
}

export async function health() {
  return getInvoke()<{ ok: boolean }>('health');
}

export async function getLocalStatus(gatewayPort?: number) {
  return getInvoke()<LocalStatus>('local_status', { gatewayPort });
}

export async function getLocalStatusQuick(gatewayPort?: number) {
  return getInvoke()<LocalStatus>('local_status_quick', { gatewayPort });
}

export async function listSessions() {
  return getInvoke()<SessionItem[]>('list_sessions');
}

export async function getSessionHistory(sessionKey: string, limit = 120) {
  return getInvoke()<any[]>('session_history', { sessionKey, limit });
}

export async function sendMessage(sessionKey: string, message: string, model?: string, gatewayPort?: number) {
  return getInvoke()<any>('send_message', { sessionKey, message, model, gatewayPort });
}

export async function restartGateway(gatewayPort?: number) {
  return getInvoke()<ActionResult>('restart_gateway', { gatewayPort });
}

export async function runDoctor(gatewayPort?: number) {
  return getInvoke()<ActionResult>('run_doctor', { gatewayPort });
}

export async function listAgents() {
  return getInvoke()<AgentItem[]>('list_agents');
}

export async function getAgentPanelStatus(gatewayPort?: number) {
  return getInvoke()<AgentPanelData>('agent_panel_status', { gatewayPort });
}

export async function getGatewayLogs(limit = 120, gatewayPort?: number) {
  return getInvoke()<GatewayLogs>('gateway_logs', { limit, gatewayPort });
}

export async function getOpenclawConfig() {
  return getInvoke()<OpenclawConfigPayload>('get_openclaw_config');
}

export async function saveOpenclawConfig(content: string) {
  return getInvoke()<OpenclawConfigPayload>('save_openclaw_config', { content });
}

export async function listSkills(gatewayPort?: number) {
  return getInvoke()<SkillsPayload>('list_skills', { gatewayPort });
}

export async function listModels(gatewayPort?: number) {
  return getInvoke()<ModelOption[]>('list_models', { gatewayPort });
}

export async function listGatewayPorts() {
  return getInvoke()<GatewayPortItem[]>('list_gateway_ports');
}
