export interface McpGaugeProps {
  percentage: number;
  title: string;
  total: number;
  used: number;
}

const WARNING_THRESHOLD = 0.7; // 70%
const DANGER_THRESHOLD = 0.9; // 90%

export function createMcpUsageGauge(props: McpGaugeProps): HTMLElement {
  const gauge = document.createElement("div");
  gauge.className = "gauge mcp-gauge";

  // percentage is already 0-100 from backend, but we need to convert to 0-1 for status check
  const utilizationRatio = props.percentage / 100;
  const status = getStatusClass(utilizationRatio);

  // Title (own line)
  const title = document.createElement("div");
  title.className = "gauge-title";
  title.textContent = props.title;

  // Progress bar
  const track = document.createElement("div");
  track.className = "progress-track";

  const fill = document.createElement("div");
  fill.className = `progress-fill ${status}`;
  fill.style.width = `${props.percentage}%`;

  track.appendChild(fill);

  // Stats row: dot + "X% used" (left), "X / Y used" (right)
  const stats = document.createElement("div");
  stats.className = "gauge-stats";

  const usedContainer = document.createElement("div");
  usedContainer.className = "gauge-stats-used";

  const dot = document.createElement("span");
  dot.className = `gauge-dot ${status}`;

  const usedText = document.createElement("span");
  usedText.textContent = `${props.percentage.toFixed(0)}% used`;

  usedContainer.appendChild(dot);
  usedContainer.appendChild(usedText);

  const countText = document.createElement("span");
  countText.className = "gauge-reset";
  countText.textContent = `${props.used} / ${props.total} used`;

  stats.appendChild(usedContainer);
  stats.appendChild(countText);

  gauge.appendChild(title);
  gauge.appendChild(track);
  gauge.appendChild(stats);

  return gauge;
}

function getStatusClass(utilization: number): string {
  if (utilization < WARNING_THRESHOLD) {
    return "status-success";
  }
  if (utilization < DANGER_THRESHOLD) {
    return "status-warning";
  }
  return "status-danger";
}
