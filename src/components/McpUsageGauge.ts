export interface McpGaugeProps {
  title: string;
  percentage: number;
  used: number;
  total: number;
}

export function createMcpUsageGauge(props: McpGaugeProps): HTMLElement {
  const gauge = document.createElement('div');
  gauge.className = 'gauge';

  const status = getStatusClass(props.percentage);

  // Title (own line)
  const title = document.createElement('div');
  title.className = 'gauge-title';
  title.textContent = props.title;

  // Progress bar
  const track = document.createElement('div');
  track.className = 'progress-track';

  const fill = document.createElement('div');
  fill.className = `progress-fill ${status}`;
  fill.style.width = `${props.percentage}%`;

  track.appendChild(fill);

  // Stats row: dot + "X% used" (left), "X / Y used" (right)
  const stats = document.createElement('div');
  stats.className = 'gauge-stats';

  const usedContainer = document.createElement('div');
  usedContainer.className = 'gauge-stats-used';

  const dot = document.createElement('span');
  dot.className = `gauge-dot ${status}`;

  const usedText = document.createElement('span');
  usedText.textContent = `${props.percentage.toFixed(0)}% used`;

  usedContainer.appendChild(dot);
  usedContainer.appendChild(usedText);

  const countText = document.createElement('span');
  countText.className = 'gauge-reset';
  countText.textContent = `${props.used} / ${props.total} used`;

  stats.appendChild(usedContainer);
  stats.appendChild(countText);

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
