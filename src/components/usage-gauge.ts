export interface GaugeProps {
  resetsAt: string;
  title: string;
  utilization: number;
}

const WARNING_THRESHOLD = 0.7; // 70%
const DANGER_THRESHOLD = 0.9; // 90%

export function createUsageGauge(props: GaugeProps): HTMLElement {
  const gauge = document.createElement("div");
  gauge.className = "gauge";

  // Convert 0-1 ratio to 0-100 percentage
  const percentage = props.utilization * 100;
  const status = getStatusClass(props.utilization);

  // Title (own line)
  const title = document.createElement("div");
  title.className = "gauge-title";
  title.textContent = props.title;

  // Progress bar
  const track = document.createElement("div");
  track.className = "progress-track";

  const fill = document.createElement("div");
  fill.className = `progress-fill ${status}`;
  fill.style.width = `${percentage}%`;

  track.appendChild(fill);

  // Stats row: dot + "X% used" (left), "Resets in ..." (right)
  const stats = document.createElement("div");
  stats.className = "gauge-stats";

  const usedContainer = document.createElement("div");
  usedContainer.className = "gauge-stats-used";

  const dot = document.createElement("span");
  dot.className = `gauge-dot ${status}`;

  const usedText = document.createElement("span");
  usedText.textContent = `${percentage.toFixed(0)}% used`;

  usedContainer.appendChild(dot);
  usedContainer.appendChild(usedText);

  const resetText = document.createElement("span");
  resetText.className = "gauge-reset";
  resetText.textContent = getTimeUntilReset(props.resetsAt);

  stats.appendChild(usedContainer);
  stats.appendChild(resetText);

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

function getTimeUntilReset(resetsAt: string): string {
  if (!resetsAt) {
    return "";
  }

  const resetDate = new Date(resetsAt);

  if (Number.isNaN(resetDate.getTime())) {
    return "";
  }

  const now = new Date();
  const interval = resetDate.getTime() - now.getTime();

  if (interval <= 0) {
    return "";
  }

  const days = Math.floor(interval / (1000 * 60 * 60 * 24));
  const hours = Math.floor(
    (interval % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)
  );
  const minutes = Math.floor((interval % (1000 * 60 * 60)) / (1000 * 60));

  if (days > 0) {
    return `Resets in ${days}d ${hours}h`;
  }
  if (hours > 0) {
    return `Resets in ${hours}h ${minutes}m`;
  }
  return `Resets in ${minutes}m`;
}
