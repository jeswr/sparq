// [OPUS-4.8] sq-hsyg — a tiny dependency-free SVG line chart, used for both the per-metric
// TREND (metric over commits) and SCALING (metric vs dataset size/depth) views ported from
// the standalone bench/dashboard (dashboard.js Chart.js cards). Why hand-rolled SVG and not
// recharts/Chart.js: the site is a STATIC EXPORT (`output: export`) on React 19; an SVG
// component is SSR-safe, adds zero bundle weight + zero peer-dep risk, and these are simple
// single-series line/area plots. The chart is computed entirely from passed-in points (no
// fabrication) and degrades gracefully — a single point renders as one marker, empty renders
// nothing. Colours use the site's --chart-* theme tokens so it matches the AppShell.
import * as React from "react";

export interface ChartPoint {
  x: number; // numeric x (epoch-ms for trend, size/depth for scaling)
  y: number; // metric value (smaller is better)
  label: string; // x-axis tick / tooltip label (date or size)
}

const W = 520;
const H = 200;
const PAD = { top: 12, right: 16, bottom: 28, left: 52 };

function niceTicks(min: number, max: number, count: number): number[] {
  if (min === max) return [min];
  const step = (max - min) / count;
  const ticks: number[] = [];
  for (let i = 0; i <= count; i++) ticks.push(min + step * i);
  return ticks;
}

// Compact numeric formatter for axis ticks (mirrors dashboard.js fmtNum altitude).
function fmtTick(v: number): string {
  const abs = Math.abs(v);
  if (abs >= 1_000_000) return (v / 1_000_000).toLocaleString("en-US", { maximumFractionDigits: 1 }) + "M";
  if (abs >= 1000) return (v / 1000).toLocaleString("en-US", { maximumFractionDigits: 1 }) + "k";
  if (abs >= 1) return (Math.round(v * 100) / 100).toLocaleString("en-US");
  return (Math.round(v * 10000) / 10000).toString();
}

export function LineChart({
  points,
  unit,
  colorVar = "--chart-1",
  xTickFormat,
  ariaLabel,
}: {
  points: ChartPoint[];
  unit: string;
  colorVar?: string;
  xTickFormat?: (x: number) => string;
  ariaLabel: string;
}) {
  if (!points.length) return null;

  const plotW = W - PAD.left - PAD.right;
  const plotH = H - PAD.top - PAD.bottom;

  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const xMin = Math.min(...xs);
  const xMax = Math.max(...xs);
  const yMin = Math.min(...ys, 0); // include 0 baseline so magnitude reads honestly
  const yMaxRaw = Math.max(...ys);
  const yMax = yMaxRaw === yMin ? yMin + 1 : yMaxRaw;

  const sx = (x: number) =>
    PAD.left + (xMax === xMin ? plotW / 2 : ((x - xMin) / (xMax - xMin)) * plotW);
  const sy = (y: number) => PAD.top + plotH - ((y - yMin) / (yMax - yMin)) * plotH;

  const linePath = points
    .map((p, i) => `${i === 0 ? "M" : "L"}${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`)
    .join(" ");
  const areaPath =
    `M${sx(points[0].x).toFixed(1)},${sy(yMin).toFixed(1)} ` +
    points.map((p) => `L${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`).join(" ") +
    ` L${sx(points[points.length - 1].x).toFixed(1)},${sy(yMin).toFixed(1)} Z`;

  const yTicks = niceTicks(yMin, yMax, 4);
  // X ticks: first / mid / last so a 20-point window stays legible.
  const xTickIdx =
    points.length <= 3
      ? points.map((_, i) => i)
      : [0, Math.floor((points.length - 1) / 2), points.length - 1];

  const stroke = `var(${colorVar})`;

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="h-44 w-full"
      role="img"
      aria-label={ariaLabel}
      preserveAspectRatio="none"
    >
      {/* y grid + ticks */}
      {yTicks.map((t, i) => (
        <g key={`y${i}`}>
          <line
            x1={PAD.left}
            x2={W - PAD.right}
            y1={sy(t)}
            y2={sy(t)}
            stroke="var(--border)"
            strokeWidth={1}
          />
          <text
            x={PAD.left - 6}
            y={sy(t)}
            textAnchor="end"
            dominantBaseline="middle"
            className="fill-muted-foreground text-[10px]"
          >
            {fmtTick(t)}
          </text>
        </g>
      ))}
      {/* x ticks */}
      {xTickIdx.map((idx) => {
        const p = points[idx];
        return (
          <text
            key={`x${idx}`}
            x={sx(p.x)}
            y={H - 8}
            textAnchor={idx === 0 ? "start" : idx === points.length - 1 ? "end" : "middle"}
            className="fill-muted-foreground text-[10px]"
          >
            {xTickFormat ? xTickFormat(p.x) : p.label}
          </text>
        );
      })}
      {/* area + line */}
      <path d={areaPath} fill={stroke} fillOpacity={0.12} stroke="none" />
      <path d={linePath} fill="none" stroke={stroke} strokeWidth={2} />
      {/* point markers (only when sparse, so a 20-pt trend stays clean) */}
      {points.length <= 8 &&
        points.map((p, i) => (
          <circle key={`pt${i}`} cx={sx(p.x)} cy={sy(p.y)} r={3} fill={stroke}>
            <title>
              {p.label}: {fmtTick(p.y)} {unit}
            </title>
          </circle>
        ))}
    </svg>
  );
}
