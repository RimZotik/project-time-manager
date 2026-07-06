// Фоновый эко-акцент: мягкий бело-зелёный градиент, срезанный волнами
// по диагонали. Тонкий, живёт под контентом (fixed, pointer-events: none),
// с медленным «дыханием» через framer-motion.
import { motion } from "framer-motion";

export default function WaveBackground() {
  return (
    <div
      aria-hidden
      className="pointer-events-none fixed inset-0 -z-10 overflow-hidden bg-[linear-gradient(180deg,#f8fbf8_0%,#eef6ef_100%)]"
    >
      <svg
        className="absolute inset-0 h-full w-full"
        viewBox="0 0 1440 900"
        preserveAspectRatio="xMidYMid slice"
      >
        <defs>
          <linearGradient id="wave-a" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="#d1fae5" stopOpacity="0.9" />
            <stop offset="100%" stopColor="#a7f3d0" stopOpacity="0.35" />
          </linearGradient>
          <linearGradient id="wave-b" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="#ecfdf5" stopOpacity="0.9" />
            <stop offset="100%" stopColor="#bbf7d0" stopOpacity="0.5" />
          </linearGradient>
        </defs>

        {/* Верхняя диагональная волна */}
        <motion.path
          fill="url(#wave-a)"
          initial={{ y: -12 }}
          animate={{ y: [-12, 10, -12] }}
          transition={{ duration: 18, repeat: Infinity, ease: "easeInOut" }}
          d="M0,150 C320,60 620,240 900,150 C1130,80 1320,190 1440,120 L1440,0 L0,0 Z"
        />

        {/* Нижняя диагональная волна */}
        <motion.path
          fill="url(#wave-b)"
          initial={{ y: 12 }}
          animate={{ y: [12, -10, 12] }}
          transition={{ duration: 22, repeat: Infinity, ease: "easeInOut" }}
          d="M0,760 C300,700 560,860 880,780 C1150,712 1300,840 1440,770 L1440,900 L0,900 Z"
        />
      </svg>
    </div>
  );
}
