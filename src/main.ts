import { invoke } from "@tauri-apps/api/core";
import { createUsageGauge } from "./components/UsageGauge";
import { createMcpUsageGauge } from "./components/McpUsageGauge";
import { createSettingsView } from "./components/SettingsView";

const POLL_INTERVAL = 300000; // 5 minutes

let pollingTimer: number | null = null;
let activeTab: "claude" | "zai" = "claude";
let claudeLastRefresh: Date | null = null;
let zaiLastRefresh: Date | null = null;
let timestampTimer: number | null = null;

async function checkZaiApiKey(): Promise<boolean> {
  return await invoke<boolean>("zai_check_api_key");
}

function updateZaiHeaderState(hasApiKey: boolean): void {
  const zaiView = document.getElementById("zai-view");
  if (zaiView) {
    const header = zaiView.querySelector(".provider-header");
    header?.classList.toggle("empty", !hasApiKey);
  }
}

function updateZaiConnectionBadge(hasApiKey: boolean): void {
  const zaiConnectedStatus = document.getElementById("zai-connected-status");
  if (!zaiConnectedStatus) return;

  zaiConnectedStatus.innerHTML = "";

  const badge = document.createElement("span");
  badge.className = hasApiKey
    ? "zai-header-badge zai-header-badge-connected"
    : "zai-header-badge zai-header-badge-disconnected";

  const icon = document.createElement("span");
  icon.className = "zai-header-badge-icon";

  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("width", "12");
  svg.setAttribute("height", "12");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");

  if (hasApiKey) {
    svg.setAttribute("stroke-width", "3");
    const polyline = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
    polyline.setAttribute("points", "20 6 9 17 4 12");
    svg.appendChild(polyline);
  } else {
    svg.setAttribute("stroke-width", "2");
    const line1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line1.setAttribute("x1", "12");
    line1.setAttribute("y1", "5");
    line1.setAttribute("x2", "12");
    line1.setAttribute("y2", "19");
    const line2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line2.setAttribute("x1", "5");
    line2.setAttribute("y1", "12");
    line2.setAttribute("x2", "19");
    line2.setAttribute("y2", "12");
    svg.appendChild(line1);
    svg.appendChild(line2);
  }

  icon.appendChild(svg);

  const label = document.createElement("span");
  label.className = "zai-header-badge-label";
  label.textContent = hasApiKey ? "Connected" : "Not connected";

  badge.appendChild(icon);
  badge.appendChild(label);
  zaiConnectedStatus.appendChild(badge);
}

interface ClaudeUsageData {
  five_hour_utilization: number;
  five_hour_resets_at: string;
  seven_day_utilization: number;
  seven_day_resets_at: string;
  extra_usage_enabled: boolean;
  extra_usage_monthly_limit: number | null;
  extra_usage_used_credits: number | null;
  extra_usage_utilization: number | null;
}

interface ClaudeTierData {
  plan_name: string;
  rate_limit_tier: string;
}

interface ZaiUsageData {
  token_usage?: {
    percentage: number;
    resets_at?: string;
  };
  mcp_usage?: {
    percentage: number;
    used: number;
    total: number;
  };
  tier_name?: string;
}

interface ZaiTierData {
  plan_name: string;
}

let settingsOpening = false;

async function openSettings(): Promise<void> {
  const existing = document.getElementById("settings-view");
  if (existing || settingsOpening) return;
  settingsOpening = true;

  try {
  const hasZaiApiKey = await checkZaiApiKey();

  const settingsView = createSettingsView({
    checkZaiApiKey,
    validateZaiApiKey: async (apiKey: string) => {
      await invoke("zai_validate_api_key", { apiKey });
    },
    saveZaiApiKey: async (apiKey: string) => {
      await invoke("zai_save_api_key", { apiKey });
    },
    deleteZaiApiKey: async () => {
      await invoke("zai_delete_api_key");
    },
    onZaiKeyChanged: refreshZaiUI,
    onClose: closeSettings,
  }, hasZaiApiKey);

  const app = document.getElementById("app");
  app?.appendChild(settingsView);
  } finally {
    settingsOpening = false;
  }
}

function closeSettings(): void {
  const settingsView = document.getElementById("settings-view");
  if (!settingsView) return;

  settingsView.style.animation = "settings-slide-out 0.2s cubic-bezier(0.4, 0, 0.2, 1) forwards";
  settingsView.addEventListener("animationend", () => settingsView.remove(), { once: true });
}

async function loadContent() {
  const loading = document.getElementById("loading");
  const content = document.getElementById("content");
  const quitButton = document.getElementById("quit-button");

  if (!loading || !content || !quitButton) return;

  content.style.display = "none";

  try {
    await fetchClaudeData();
    await fetchZaiData();

    const hasZaiApiKey = await checkZaiApiKey();
    updateZaiHeaderState(hasZaiApiKey);
    updateZaiConnectionBadge(hasZaiApiKey);

    loading.style.display = "none";
    content.style.display = "flex";

    setupTabSwitching();

    const savedTab = localStorage.getItem("activeTab");
    if (savedTab === "zai") {
      switchTab("zai");
    }

    startPolling();
    startTimestampUpdater();
  } catch (error) {
    console.error("Failed to load content:", error);
    loading.innerHTML = "<span>Failed to load</span>";
  }

  quitButton.addEventListener("click", async () => {
    await invoke("quit_app");
  });

  const settingsButton = document.getElementById("settings-button");
  settingsButton?.addEventListener("click", () => openSettings());
}

function setupTabSwitching() {
  const tabClaude = document.getElementById("tab-claude");
  const tabZai = document.getElementById("tab-zai");

  tabClaude?.addEventListener("click", () => switchTab("claude"));
  tabZai?.addEventListener("click", () => switchTab("zai"));
}

function switchTab(tab: "claude" | "zai") {
  activeTab = tab;
  localStorage.setItem("activeTab", tab);

  const claudeView = document.getElementById("claude-view");
  const zaiView = document.getElementById("zai-view");
  const tabClaude = document.getElementById("tab-claude");
  const tabZai = document.getElementById("tab-zai");

  if (claudeView && zaiView && tabClaude && tabZai) {
    claudeView.style.display = tab === "claude" ? "block" : "none";
    zaiView.style.display = tab === "zai" ? "block" : "none";

    tabClaude.classList.toggle("active", tab === "claude");
    tabZai.classList.toggle("active", tab === "zai");
  }
}

async function fetchClaudeData() {
  try {
    const [usageData, tierData] = await invoke<[ClaudeUsageData, ClaudeTierData]>("claude_get_all");

    const errorContainer = document.getElementById("claude-error");
    const dataContainer = document.getElementById("claude-data");
    const errorMessage = document.getElementById("claude-error-message");

    if (errorContainer) errorContainer.style.display = "none";
    if (dataContainer) {
      dataContainer.style.display = "block";
      dataContainer.innerHTML = "";
    }

    const sessionGauge = createUsageGauge({
      title: "Session",
      utilization: usageData.five_hour_utilization / 100,
      resetsAt: usageData.five_hour_resets_at,
    });

    const weeklyGauge = createUsageGauge({
      title: "Weekly",
      utilization: usageData.seven_day_utilization / 100,
      resetsAt: usageData.seven_day_resets_at,
    });

    if (dataContainer) {
      dataContainer.appendChild(sessionGauge);
      dataContainer.appendChild(weeklyGauge);
    }

    const extraUsageLabel = document.getElementById("extra-usage-label");
    const extraUsageValue = document.getElementById("extra-usage-value");
    const extraUsageSection = document.getElementById("extra-usage-section");

    if (extraUsageLabel && extraUsageValue && extraUsageSection) {
      if (usageData.extra_usage_enabled) {
        extraUsageSection.style.display = "block";
        const monthlyLimit = usageData.extra_usage_monthly_limit ?? 0;
        const usedCredits = usageData.extra_usage_used_credits ?? 0;
        const utilization = usageData.extra_usage_utilization ?? 0;

        extraUsageLabel.textContent = `This month: $${usedCredits.toFixed(2)} / $${monthlyLimit.toFixed(2)}`;
        extraUsageValue.textContent = `${(utilization * 100).toFixed(0)}% used`;
      } else {
        extraUsageSection.style.display = "none";
      }
    }

    const tierEl = document.getElementById("claude-tier");
    if (tierEl) {
      tierEl.textContent = tierData.plan_name;
      tierEl.title = "";
    }

    claudeLastRefresh = new Date();
    updateTimestamp("claude");
  } catch (error) {
    const errorMsg = String(error);
    const errorContainer = document.getElementById("claude-error");
    const dataContainer = document.getElementById("claude-data");
    const errorMessage = document.getElementById("claude-error-message");

    if (errorContainer && errorMessage) {
      if (
        errorMsg.includes("Credential not found") ||
        errorMsg.includes("not found")
      ) {
        errorMessage.textContent =
          "Claude credentials not found. Please sign in to Claude Code first.";
      } else if (errorMsg.includes("Access denied")) {
        errorMessage.textContent = "Access denied -- check your permissions";
      } else if (errorMsg.includes("Rate limited")) {
        errorMessage.textContent = "Rate limited -- please wait and try again";
      } else {
        errorMessage.textContent = errorMsg;
      }
      errorContainer.style.display = "flex";
    }
    if (dataContainer) dataContainer.style.display = "none";

    const tierEl = document.getElementById("claude-tier");
    if (tierEl) {
      tierEl.textContent = "Error";
      tierEl.title = String(error);
    }
  }
}

async function fetchZaiData(forceRefresh = false) {
  const zaiView = document.getElementById("zai-view");
  const errorContainer = document.getElementById("zai-error");
  const dataContainer = document.getElementById("zai-data");
  const errorMessage = document.getElementById("zai-error-message");

  if (!zaiView || !errorContainer || !dataContainer || !errorMessage) return;

  try {
    const command = forceRefresh ? "zai_refresh_all" : "zai_get_all";
    const [usageData, tierData] = await invoke<[ZaiUsageData, ZaiTierData]>(command);

    if (!usageData) return;

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    if (usageData.token_usage) {
      const tokenGauge = createUsageGauge({
        title: "Session",
        utilization: usageData.token_usage.percentage / 100,
        resetsAt: usageData.token_usage.resets_at
          ? new Date(usageData.token_usage.resets_at).toISOString()
          : "",
      });
      dataContainer.appendChild(tokenGauge);
    }

    if (usageData.mcp_usage) {
      const mcpGauge = createMcpUsageGauge({
        title: "MCP Usage (Monthly)",
        percentage: usageData.mcp_usage.percentage,
        used: usageData.mcp_usage.used,
        total: usageData.mcp_usage.total,
      });
      dataContainer.appendChild(mcpGauge);
    }

    const tierEl = document.getElementById("zai-tier");
    if (tierEl) {
      tierEl.textContent = tierData.plan_name;
      tierEl.title = "";
    }

    zaiLastRefresh = new Date();
    updateTimestamp("zai");
  } catch (error) {
    const errorMsg = String(error);
    if (errorMsg.includes("not configured")) {
      dataContainer.style.display = "none";
      errorContainer.style.display = "none";
    } else {
      errorMessage.textContent = errorMsg;
      errorContainer.style.display = "flex";
      dataContainer.style.display = "none";
    }

    const tierEl = document.getElementById("zai-tier");
    if (tierEl) {
      tierEl.textContent = "Error";
      tierEl.title = String(error);
    }
  }
}

async function refreshZaiUI(): Promise<void> {
  const hasZaiApiKey = await checkZaiApiKey();
  updateZaiHeaderState(hasZaiApiKey);
  updateZaiConnectionBadge(hasZaiApiKey);
  await fetchZaiData(true);
}

async function fetchClaudeUsage() {
  const errorContainer = document.getElementById("claude-error");
  const dataContainer = document.getElementById("claude-data");
  const errorMessage = document.getElementById("claude-error-message");

  if (!errorContainer || !dataContainer || !errorMessage) return;

  try {
    const data = await invoke<ClaudeUsageData>("claude_get_usage");

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    const sessionGauge = createUsageGauge({
      title: "Session",
      utilization: data.five_hour_utilization / 100, // Convert percentage to 0-1 ratio
      resetsAt: data.five_hour_resets_at,
    });

    const weeklyGauge = createUsageGauge({
      title: "Weekly",
      utilization: data.seven_day_utilization / 100, // Convert percentage to 0-1 ratio
      resetsAt: data.seven_day_resets_at,
    });

    dataContainer.appendChild(sessionGauge);
    dataContainer.appendChild(weeklyGauge);

    // Update extra usage section
    const extraUsageLabel = document.getElementById("extra-usage-label");
    const extraUsageValue = document.getElementById("extra-usage-value");
    const extraUsageSection = document.getElementById("extra-usage-section");

    if (extraUsageLabel && extraUsageValue && extraUsageSection) {
      if (data.extra_usage_enabled) {
        extraUsageSection.style.display = "block";
        const monthlyLimit = data.extra_usage_monthly_limit ?? 0;
        const usedCredits = data.extra_usage_used_credits ?? 0;
        const utilization = data.extra_usage_utilization ?? 0;

        extraUsageLabel.textContent = `This month: $${usedCredits.toFixed(2)} / $${monthlyLimit.toFixed(2)}`;
        extraUsageValue.textContent = `${(utilization * 100).toFixed(0)}% used`;
      } else {
        extraUsageSection.style.display = "none";
      }
    }

    claudeLastRefresh = new Date();
    updateTimestamp("claude");
  } catch (error) {
    const errorMsg = String(error);

    if (
      errorMsg.includes("Credential not found") ||
      errorMsg.includes("not found")
    ) {
      errorMessage.textContent =
        "Claude credentials not found. Please sign in to Claude Code first.";
    } else if (errorMsg.includes("Access denied")) {
      errorMessage.textContent = "Access denied -- check your permissions";
    } else if (errorMsg.includes("Rate limited")) {
      errorMessage.textContent = "Rate limited -- please wait and try again";
    } else {
      errorMessage.textContent = errorMsg;
    }

    errorContainer.style.display = "flex";
    dataContainer.style.display = "none";
  }
}

async function fetchZaiUsage(forceRefresh = false): Promise<void> {
  const zaiView = document.getElementById("zai-view");
  const errorContainer = document.getElementById("zai-error");
  const dataContainer = document.getElementById("zai-data");
  const errorMessage = document.getElementById("zai-error-message");

  if (!zaiView || !errorContainer || !dataContainer || !errorMessage) return;

  try {
    const command = forceRefresh ? "zai_refresh_usage" : "zai_get_usage";
    const data = await invoke<ZaiUsageData>(command);

    if (!data) return;

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    if (data.token_usage) {
      const tokenGauge = createUsageGauge({
        title: "Session",
        utilization: data.token_usage.percentage / 100, // Convert percentage to 0-1 ratio
        resetsAt: data.token_usage.resets_at
          ? new Date(data.token_usage.resets_at).toISOString()
          : "",
      });
      dataContainer.appendChild(tokenGauge);
    }

    if (data.mcp_usage) {
      const mcpGauge = createMcpUsageGauge({
        title: "MCP Usage (Monthly)",
        percentage: data.mcp_usage.percentage,
        used: data.mcp_usage.used,
        total: data.mcp_usage.total,
      });
      dataContainer.appendChild(mcpGauge);
    }

    zaiLastRefresh = new Date();
    updateTimestamp("zai");
  } catch (error) {
    const errorMsg = String(error);
    if (errorMsg.includes("not configured")) {
      // Hide data, show nothing — settings will handle it
      dataContainer.style.display = "none";
      errorContainer.style.display = "none";
    } else {
      errorMessage.textContent = errorMsg;
      errorContainer.style.display = "flex";
      dataContainer.style.display = "none";
    }
  }
}

function updateTimestamp(provider: "claude" | "zai") {
  const el = document.getElementById(`${provider}-updated`);
  if (!el) return;

  const lastRefresh =
    provider === "claude" ? claudeLastRefresh : zaiLastRefresh;
  if (!lastRefresh) {
    el.textContent = "Updated just now";
    return;
  }

  const now = new Date();
  const diffMs = now.getTime() - lastRefresh.getTime();
  const diffMin = Math.floor(diffMs / 60000);

  if (diffMin < 1) {
    el.textContent = "Updated just now";
  } else if (diffMin < 60) {
    el.textContent = `Updated ${diffMin}m ago`;
  } else {
    const diffHours = Math.floor(diffMin / 60);
    el.textContent = `Updated ${diffHours}h ago`;
  }
}

function startTimestampUpdater() {
  if (timestampTimer !== null) return;
  timestampTimer = window.setInterval(() => {
    updateTimestamp("claude");
    updateTimestamp("zai");
  }, 30000); // update every 30s
}

async function handleRefresh() {
  try {
    await fetchClaudeData();
    await fetchZaiData();
  } catch (error) {
    console.error("Failed to refresh:", error);
  }
}

function startPolling() {
  if (pollingTimer !== null) return;

  pollingTimer = window.setInterval(async () => {
    try {
      await fetchClaudeData();
      await fetchZaiData();
    } catch (error) {
      console.error("Polling error:", error);
    }
  }, POLL_INTERVAL);
}

function stopPolling() {
  if (pollingTimer !== null) {
    clearInterval(pollingTimer);
    pollingTimer = null;
  }
  if (timestampTimer !== null) {
    clearInterval(timestampTimer);
    timestampTimer = null;
  }
}

document.addEventListener("DOMContentLoaded", loadContent);

window.addEventListener("beforeunload", stopPolling);
