type HaloMarkProps = {
  size?: number;
  className?: string;
};

/** The Halo mark: amber ring with a dot in the gap. */
export function HaloMark({ size = 64, className }: HaloMarkProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      role="img"
      aria-label="Halo"
      className={className}
    >
      <circle
        cx="32"
        cy="32"
        r="22"
        fill="none"
        stroke="var(--halo-focus)"
        strokeWidth="6"
        strokeLinecap="round"
        strokeDasharray="118 20.23"
        transform="rotate(-50 32 32)"
      />
      <circle cx="37.1" cy="10.6" r="3.6" fill="var(--halo-focus)" />
    </svg>
  );
}
