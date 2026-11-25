import { motion } from "motion/react";

export function Meter({ value, processing = false }: { value: number; processing?: boolean }) {
  const ratio = Math.max(0, Math.min(1, value / 100));
  const hidden = 100 - ratio * 100;
  return (
    <div className="meter">
      <motion.div
        className={`meter-fill ${processing ? "processing" : ""}`}
        initial={false}
        animate={{ clipPath: `inset(0 ${hidden}% 0 0)` }}
        transition={{ duration: 0.35, ease: [0.2, 0, 0, 1] }}
      />
    </div>
  );
}
