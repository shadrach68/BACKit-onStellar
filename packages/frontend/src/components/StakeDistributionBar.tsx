"use client";

import { useEffect, useRef, useState } from "react";

interface StakeDistributionBarProps {
  /** Raw YES stake amount (any unit — USDC, XLM, etc.) */
  yes: number;
  /** Raw NO stake amount */
  no: number;
  /** Token label shown next to the total pool size (default "USDC") */
  currency?: string;
  /**
   * visual size variant:
   *  - "sm"  — compact bar used in CallCard
   *  - "lg"  — taller bar with bigger labels used in the market detail page
   */
  variant?: "sm" | "lg";
}

/**
 * StakeDistributionBar
 *
 * Shows the YES / NO stake split as an animated progress bar.
 *
 * Edge-cases handled:
 *  • Empty pool (yes === 0 && no === 0) → 50 / 50 grey bar
 *  • Single-sided (one side is 0) → 100 % one colour
 *  • Animates width from 0 on mount and whenever yes/no values change
 */
export default function StakeDistributionBar({
  yes,
  no,
  currency = "USDC",
  variant = "sm",
}: StakeDistributionBarProps) {
  const total = yes + no;
  const isEmpty = total === 0;

  // Target percentages
  const targetYesPct = isEmpty ? 50 : (yes / total) * 100;
  const targetNoPct = isEmpty ? 50 : (no / total) * 100;

  // Animated width state — starts at 50/50 and transitions to real values
  const [yesPct, setYesPct] = useState(50);
  const [noPct, setNoPct] = useState(50);
  const rafRef = useRef<number | null>(null);
  const prevYes = useRef(yes);
  const prevNo = useRef(no);

  useEffect(() => {
    // Cancel any in-flight animation
    if (rafRef.current) cancelAnimationFrame(rafRef.current);

    const start = performance.now();
    const duration = 700; // ms

    const fromYes = yesPct;
    const fromNo = noPct;
    const toYes = targetYesPct;
    const toNo = targetNoPct;

    function easeOutCubic(t: number) {
      return 1 - Math.pow(1 - t, 3);
    }

    function step(now: number) {
      const elapsed = now - start;
      const progress = Math.min(elapsed / duration, 1);
      const eased = easeOutCubic(progress);

      const currentYes = fromYes + (toYes - fromYes) * eased;
      const currentNo = fromNo + (toNo - fromNo) * eased;

      setYesPct(currentYes);
      setNoPct(currentNo);

      if (progress < 1) {
        rafRef.current = requestAnimationFrame(step);
      }
    }

    rafRef.current = requestAnimationFrame(step);
    prevYes.current = yes;
    prevNo.current = no;

    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [yes, no]);

  // ── Derived display values ──────────────────────────────────────────────
  const displayYesPct = Math.round(targetYesPct);
  const displayNoPct = Math.round(targetNoPct);
  const totalFormatted = total.toLocaleString();

  // ── Style variants ───────────────────────────────────────────────────────
  const barHeight = variant === "lg" ? "h-5" : "h-3";
  const labelSize = variant === "lg" ? "text-sm" : "text-[11px]";
  const poolSize = variant === "lg" ? "text-xs" : "text-[10px]";

  // ── Colour logic ─────────────────────────────────────────────────────────
  // Empty pool → grey; otherwise green / red
  const yesColour = isEmpty
    ? "bg-gray-300"
    : "bg-gradient-to-r from-emerald-400 to-green-500";
  const noColour = isEmpty
    ? "bg-gray-300"
    : "bg-gradient-to-r from-red-400 to-rose-500";

  // Glow pulse only when pool has real liquidity
  const yesGlow =
    !isEmpty && yes > 0
      ? "shadow-[inset_0_0_6px_rgba(52,211,153,0.4)]"
      : "";
  const noGlow =
    !isEmpty && no > 0
      ? "shadow-[inset_0_0_6px_rgba(251,113,133,0.4)]"
      : "";

  return (
    <div
      className="w-full select-none"
      aria-label={`Stake distribution: ${displayYesPct}% YES, ${displayNoPct}% NO`}
    >
      {/* ── Percentage labels ─────────────────────────────────────────── */}
      <div className={`flex justify-between font-bold mb-1.5 ${labelSize}`}>
        <span
          className={isEmpty ? "text-gray-400" : "text-emerald-600"}
          aria-label={`${displayYesPct}% UP`}
        >
          {isEmpty ? "—" : `${displayYesPct}%`}{" "}
          <span className={isEmpty ? "text-gray-400" : "text-emerald-500"}>
            UP
          </span>
        </span>
        <span
          className={isEmpty ? "text-gray-400" : "text-rose-500"}
          aria-label={`${displayNoPct}% DOWN`}
        >
          <span className={isEmpty ? "text-gray-400" : "text-rose-400"}>
            DOWN
          </span>{" "}
          {isEmpty ? "—" : `${displayNoPct}%`}
        </span>
      </div>

      {/* ── Animated bar ─────────────────────────────────────────────── */}
      <div
        className={`flex w-full ${barHeight} rounded-full overflow-hidden bg-gray-100`}
        role="progressbar"
        aria-valuenow={displayYesPct}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        {/* YES segment */}
        <div
          className={`${yesColour} ${yesGlow} rounded-l-full transition-none`}
          style={{ width: `${yesPct}%` }}
        />
        {/* NO segment */}
        <div
          className={`${noColour} ${noGlow} rounded-r-full flex-1`}
          style={{ width: `${noPct}%` }}
        />
      </div>

      {/* ── Pool total ───────────────────────────────────────────────── */}
      <div
        className={`mt-1.5 text-center ${poolSize} font-semibold ${
          isEmpty ? "text-gray-400" : "text-gray-500"
        }`}
      >
        {isEmpty ? (
          "No liquidity yet"
        ) : (
          <>
            Total pool:{" "}
            <span className="text-gray-700 font-bold">
              {totalFormatted} {currency}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
