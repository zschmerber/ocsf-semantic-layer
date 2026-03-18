/**
 * ContextualTooltip component — inline popover that explains OCSF concepts.
 *
 * Renders a (?) help icon next to wrapped children. On hover or click,
 * shows a popover with a concise explanation of the OCSF term and an
 * optional "Learn more" link to OCSF documentation.
 *
 * Uses viewport boundary detection to reposition the popover if it would
 * overflow. Only renders the help icon when guideVisible is true.
 *
 * Requirements: 7.1, 7.2, 7.3, 7.4, 11.3
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import type { OCSFTerm } from '../../types/guide';
import { TOOLTIP_CONTENT } from '../../types/guide';
import { useGuideStore } from '../../store/guideStore';
import './ContextualTooltip.css';

export interface ContextualTooltipProps {
  term: OCSFTerm;
  children: React.ReactNode;
}

export function ContextualTooltip({ term, children }: ContextualTooltipProps) {
  const guideVisible = useGuideStore((s) => s.guideVisible);
  const [open, setOpen] = useState(false);
  const [position, setPosition] = useState<'above' | 'below'>('above');
  const iconRef = useRef<HTMLButtonElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);

  const content = TOOLTIP_CONTENT[term];

  /** Recompute popover position based on viewport boundaries. */
  const updatePosition = useCallback(() => {
    if (!iconRef.current) return;
    const rect = iconRef.current.getBoundingClientRect();
    // If the icon is near the top of the viewport, flip popover below
    const spaceAbove = rect.top;
    setPosition(spaceAbove < 120 ? 'below' : 'above');
  }, []);

  /** Close popover on click outside or Escape key. */
  useEffect(() => {
    if (!open) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        popoverRef.current &&
        !popoverRef.current.contains(e.target as Node) &&
        iconRef.current &&
        !iconRef.current.contains(e.target as Node)
      ) {
        setOpen(false);
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [open]);

  /** Recalculate position when popover opens. */
  useEffect(() => {
    if (open) updatePosition();
  }, [open, updatePosition]);

  const handleToggle = () => {
    setOpen((prev) => !prev);
  };

  const handleMouseEnter = () => {
    updatePosition();
    setOpen(true);
  };

  const handleMouseLeave = () => {
    setOpen(false);
  };

  // Only render the help icon when guide is visible
  if (!guideVisible) {
    return <>{children}</>;
  }

  return (
    <span className="contextual-tooltip" onMouseLeave={handleMouseLeave}>
      {children}
      <button
        ref={iconRef}
        className="contextual-tooltip__icon"
        onClick={handleToggle}
        onMouseEnter={handleMouseEnter}
        aria-label={`Help: ${term.replace(/_/g, ' ')}`}
        aria-expanded={open}
        type="button"
      >
        ?
      </button>

      {open && (
        <div
          ref={popoverRef}
          className={`contextual-tooltip__popover contextual-tooltip__popover--${position}`}
          role="tooltip"
        >
          <div className="contextual-tooltip__arrow" />
          <p className="contextual-tooltip__explanation">{content.explanation}</p>
          {content.learnMoreUrl && (
            <a
              className="contextual-tooltip__learn-more"
              href={content.learnMoreUrl}
              target="_blank"
              rel="noopener noreferrer"
            >
              Learn more →
            </a>
          )}
        </div>
      )}
    </span>
  );
}

export default ContextualTooltip;
