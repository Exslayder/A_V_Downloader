import { Area, AreaChart, CartesianGrid, ResponsiveContainer, YAxis } from "recharts";

import { formatSpeed } from "../lib/format";
import { useI18n } from "../lib/i18n";

export function SpeedChart({ samples }: { samples: number[] }) {
  const { t } = useI18n();
  if (samples.length < 2) return null;

  const data = samples.map((speed, index) => ({ index, speed }));
  const current = formatSpeed(samples[samples.length - 1]) ?? "0 B/s";

  return (
    <div className="speed-chart">
      <div className="speed-chart-head">
        <span>{t("downloads.speed")}</span>
        <span className="mono">{current}</span>
      </div>
      <ResponsiveContainer width="100%" height={52}>
        <AreaChart data={data} margin={{ top: 6, right: 0, bottom: 0, left: 0 }}>
          <defs>
            <linearGradient id="speed-fill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#8b5cf6" stopOpacity={0.16} />
              <stop offset="100%" stopColor="#8b5cf6" stopOpacity={0.01} />
            </linearGradient>
          </defs>
          <CartesianGrid
            vertical={false}
            stroke="rgba(0, 0, 0, 0.06)"
            strokeDasharray="2 4"
          />
          <YAxis hide domain={[0, "dataMax"]} />
          <Area
            type="monotone"
            dataKey="speed"
            stroke="#8b5cf6"
            strokeWidth={1.4}
            fill="url(#speed-fill)"
            dot={false}
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
