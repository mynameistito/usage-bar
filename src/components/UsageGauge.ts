export interface GaugeProps {
  title: string;
  utilization: number;
  resetsAt: string;
}

export function createUsageGauge(props: GaugeProps): HTMLElement {
  const gauge = document.createElement('div');
  gauge.className = 'gauge';

  const status = getStatusClass(props.utilization);

  // Title (own line)
  const title = document.createElement('div');
  title.className = 'gauge-title';
  title.textContent = props.title;

  // Progress bar
  const track = document.createElement('div');
  track.className = 'progress-track';

  const fill = document.createElement('div');
  fill.className = `progress-fill ${status}`;
  fill.style.width = `${props.utilization}%`;

  track.appendChild(fill);

  // Stats row: dot + "X% used" (left), "Resets in ..." (right)
  const stats = document.createElement('div');
  stats.className = 'gauge-stats';

  const usedContainer = document.createElement('div');
  usedContainer.className = 'gauge-stats-used';

  const dot = document.createElement('span');
  dot.className = `gauge-dot ${status}`;

  const usedText = document.createElement('span');
  usedText.textContent = `${props.utilization.toFixed(0)}% used`;

  usedContainer.appendChild(dot);
  usedContainer.appendChild(usedText);

  const resetText = document.createElement('span');
  resetText.className = 'gauge-reset';
  resetText.textContent = getTimeUntilReset(props.resetsAt);

  stats.appendChild(usedContainer);
  stats.appendChild(resetText);

  gauge.appendChild(title);
  gauge.appendChild(track);
  gauge.appendChild(stats);

  return gauge;
}

function getStatusClass(utilization: number): string {
  if (utilization < 50) return 'status-success';
  if (utilization < 75) return 'status-warning';
  return 'status-danger';
}

function getTimeUntilReset(resetsAt: string): string {
  if (!resetsAt) {
    return '';
  }

  const resetDate = new Date(resetsAt);

  if (isNaN(resetDate.getTime())) {
    return '';
  }

  const now = new Date();
  const interval = resetDate.getTime() - now.getTime();

  if (interval <= 0) {
    return '';
  }

  const days = Math.floor(interval / (1000 * 60 * 60 * 24));
  const hours = Math.floor((interval % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((interval % (1000 * 60 * 60)) / (1000 * 60));

  if (days > 0) {
    return `Resets in ${days}d ${hours}h`;
  } else if (hours > 0) {
    return `Resets in ${hours}h ${minutes}m`;
  } else {
    return `Resets in ${minutes}m`;
  }
}
