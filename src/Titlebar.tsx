import { useEffect, useMemo, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { t } from "./i18n/strings";

export default function Titlebar({ children }: { children: ReactNode }) {
  const appWindow = useMemo(() => getCurrentWindow(), []);
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    appWindow.isMaximized().then(setIsMaximized);
    const unlisten = appWindow.onResized(() => {
      void appWindow.isMaximized().then(setIsMaximized);
    });
    return () => {
      void unlisten.then((f) => f());
    };
  }, [appWindow]);

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-toolbar" data-tauri-drag-region>
        {children}
      </div>
      <div className="titlebar-controls">
        <button
          className="titlebar-button"
          onClick={() => void appWindow.minimize()}
          title={t.Minimize}
          aria-label={t.Minimize}
        >
          &#xE921;
        </button>
        <button
          className="titlebar-button"
          onClick={() => void appWindow.toggleMaximize()}
          title={isMaximized ? t.Restore : t.Maximize}
          aria-label={isMaximized ? t.Restore : t.Maximize}
        >
          {isMaximized ? <>&#xE923;</> : <>&#xE922;</>}
        </button>
        <button
          className="titlebar-button close"
          onClick={() => void appWindow.close()}
          title={t.Close}
          aria-label={t.Close}
        >
          &#xE8BB;
        </button>
      </div>
    </div>
  );
}
