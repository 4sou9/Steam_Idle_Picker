import { useEffect, useLayoutEffect, useRef, useState } from "react";

export interface MenuItem {
  label: string;
  onSelect: () => void;
}

/**
 * Minimal Fluent-style context menu; closes on outside click, Escape, Tab, blur or
 * scroll. The first item is focused on open and the arrow, Home and End keys move
 * between items.
 */
export default function ContextMenu({
  x,
  y,
  items,
  onClose,
}: {
  x: number;
  y: number;
  items: MenuItem[];
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ left: x, top: y });
  // Where focus was before the menu opened, restored when the menu closes by keyboard
  // or by choosing an item (not on an outside click, which moves focus itself).
  const returnFocusRef = useRef<HTMLElement | null>(null);

  // Keep the menu inside the window.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const { width, height } = el.getBoundingClientRect();
    setPosition({
      left: Math.max(4, Math.min(x, window.innerWidth - width - 4)),
      top: Math.max(4, Math.min(y, window.innerHeight - height - 4)),
    });
  }, [x, y]);

  useEffect(() => {
    returnFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    ref.current?.querySelector<HTMLButtonElement>("[role=menuitem]")?.focus();
  }, []);

  useEffect(() => {
    const closeWithFocus = () => {
      onClose();
      returnFocusRef.current?.focus();
    };
    const onPointerDown = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) onClose();
    };
    const onKeyDown = (e: KeyboardEvent) => {
      const buttons = Array.from(ref.current?.querySelectorAll<HTMLButtonElement>("[role=menuitem]") ?? []);
      const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
      const focusAt = (i: number) => buttons[(i + buttons.length) % buttons.length]?.focus();
      switch (e.key) {
        case "Escape":
        case "Tab":
          e.preventDefault();
          closeWithFocus();
          break;
        case "ArrowDown":
          e.preventDefault();
          focusAt(index + 1);
          break;
        case "ArrowUp":
          e.preventDefault();
          focusAt(index < 0 ? -1 : index - 1);
          break;
        case "Home":
          e.preventDefault();
          focusAt(0);
          break;
        case "End":
          e.preventDefault();
          focusAt(-1);
          break;
      }
    };
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("blur", onClose);
    window.addEventListener("wheel", onClose, { passive: true });
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("blur", onClose);
      window.removeEventListener("wheel", onClose);
    };
  }, [onClose]);

  return (
    <div ref={ref} className="context-menu" style={position} role="menu">
      {items.map((item) => (
        <button
          key={item.label}
          className="context-menu-item"
          role="menuitem"
          onClick={() => {
            onClose();
            returnFocusRef.current?.focus();
            item.onSelect();
          }}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
