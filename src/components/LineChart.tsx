interface LineChartProps {
  data: number[];
  color?: string;
  max?: number;
  height?: number;
  width?: number;
  fill?: boolean;
}

export default function LineChart({
  data,
  color = "#0f6cbd",
  max,
  height = 60,
  width = 600,
  fill = true,
}: LineChartProps) {
  if (data.length < 2) {
    return <svg width="100%" height={height} style={{ display: "block" }} />;
  }
  const peak = Math.max(max ?? 0, ...data, 1);
  const stepX = width / (data.length - 1);
  const points = data.map((v, i) => {
    const x = i * stepX;
    const y = height - (Math.min(v, peak) / peak) * (height - 2) - 1;
    return [x, y] as const;
  });

  const line = points
    .map((p, i) => `${i === 0 ? "M" : "L"}${p[0].toFixed(1)},${p[1].toFixed(1)}`)
    .join(" ");
  const area = `${line} L${width},${height} L0,${height} Z`;
  const gid = `tmg-${color.replace(/[^a-z0-9]/gi, "")}`;

  return (
    <svg
      width="100%"
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      style={{ display: "block" }}
    >
      <defs>
        <linearGradient id={gid} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.35" />
          <stop offset="100%" stopColor={color} stopOpacity="0.02" />
        </linearGradient>
      </defs>
      {fill && <path d={area} fill={`url(#${gid})`} stroke="none" />}
      <path d={line} fill="none" stroke={color} strokeWidth={1.5} vectorEffect="non-scaling-stroke" />
    </svg>
  );
}
