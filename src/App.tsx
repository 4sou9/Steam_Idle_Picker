import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import { detectLanguage, getStrings } from "./i18n/strings";
import type { AppSettings, SortMode, SteamGame } from "./types";
import "./styles/app.css";

const MAX_SELECTION = 32;
const lang = detectLanguage();
const t = getStrings(lang);

function sortIcon(active: boolean, ascending: boolean) {
  if (!active) return "";
  return ascending ? " ↑" : " ↓";
}

export default function App() {
  const [games, setGames] = useState<SteamGame[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [idlingIds, setIdlingIds] = useState<Set<number>>(new Set());
  const [searchText, setSearchText] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("none");
  const [sortAscending, setSortAscending] = useState(true);
  const [statusMessage, setStatusMessage] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);

  const settingsRef = useRef<AppSettings>({ Language: lang, SelectedGames: [] });
  const pollRef = useRef<number | null>(null);

  const isIdling = idlingIds.size > 0;

  // ── Initial load ──────────────────────────────────────────────────────
  useEffect(() => {
    (async () => {
      const [cache, settings] = await Promise.all([api.loadCache(), api.loadSettings()]);
      settingsRef.current = settings;
      setSelectedIds(new Set(settings.SelectedGames));
      if (cache) {
        setGames(cache.Games);
      } else {
        setStatusMessage(t.NoCache);
      }
      const ids = await api.getIdlingIds();
      if (ids.length > 0) setIdlingIds(new Set(ids));
    })();
  }, []);

  // ── Polling while idling (5.1.1: replace idlingIds wholesale) ──────────
  useEffect(() => {
    if (!isIdling) {
      if (pollRef.current !== null) {
        window.clearInterval(pollRef.current);
        pollRef.current = null;
      }
      return;
    }
    pollRef.current = window.setInterval(async () => {
      const ids = await api.getIdlingIds();
      setIdlingIds(new Set(ids));
    }, 1000);
    return () => {
      if (pollRef.current !== null) window.clearInterval(pollRef.current);
      pollRef.current = null;
    };
  }, [isIdling]);

  const persistSelection = useCallback((ids: Set<number>) => {
    const settings: AppSettings = { ...settingsRef.current, SelectedGames: Array.from(ids) };
    settingsRef.current = settings;
    void api.saveSettings(settings);
  }, []);

  // ── Selection ────────────────────────────────────────────────────────
  const toggleSelected = useCallback(
    (appId: number, checked: boolean) => {
      setSelectedIds((prev) => {
        if (checked && prev.size >= MAX_SELECTION) return prev; // hard cap, ignore
        const next = new Set(prev);
        if (checked) next.add(appId);
        else next.delete(appId);
        persistSelection(next);
        return next;
      });

      if (isIdling) {
        if (checked) {
          void api.startIdle(appId).then((ok) => {
            if (ok) setIdlingIds((prev) => new Set(prev).add(appId));
          });
        } else {
          void api.stopIdle(appId).then(() => {
            setIdlingIds((prev) => {
              const next = new Set(prev);
              next.delete(appId);
              return next;
            });
          });
        }
      }
    },
    [isIdling, persistSelection]
  );

  const clearSelection = useCallback(() => {
    setSelectedIds(new Set());
    persistSelection(new Set());
  }, [persistSelection]);

  // ── Idle start/stop ──────────────────────────────────────────────────
  const startIdling = useCallback(async () => {
    const started = new Set<number>();
    for (const appId of selectedIds) {
      const ok = await api.startIdle(appId);
      if (ok) started.add(appId);
    }
    setIdlingIds(started);
  }, [selectedIds]);

  const stopAllIdling = useCallback(async () => {
    await api.stopAll();
    setIdlingIds(new Set());
  }, []);

  const toggleIdle = useCallback(() => {
    if (isIdling) void stopAllIdling();
    else void startIdling();
  }, [isIdling, startIdling, stopAllIdling]);

  const stopSingle = useCallback(
    async (appId: number) => {
      await api.stopIdle(appId);
      setIdlingIds((prev) => {
        const next = new Set(prev);
        next.delete(appId);
        return next;
      });
      setSelectedIds((prev) => {
        if (!prev.has(appId)) return prev;
        const next = new Set(prev);
        next.delete(appId);
        persistSelection(next);
        return next;
      });
    },
    [persistSelection]
  );

  // ── Refresh ──────────────────────────────────────────────────────────
  const refreshLibrary = useCallback(async () => {
    setIsRefreshing(true);
    setStatusMessage(t.LoadingLibrary);
    try {
      const result = await api.refreshLibrary();
      setGames(result.cache.Games);
      setStatusMessage("");
    } catch (e) {
      setStatusMessage(t.LoadError + String(e));
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  // ── Sort ─────────────────────────────────────────────────────────────
  const toggleSort = useCallback(
    (mode: SortMode) => {
      if (sortMode === mode) {
        setSortAscending((a) => !a);
      } else {
        setSortMode(mode);
        setSortAscending(true);
      }
    },
    [sortMode]
  );

  // ── Derived display list — pure, never touches selection/idle state ───
  const filteredGames = useMemo(() => {
    const search = searchText.trim().toLowerCase();
    let result = games;
    if (search) {
      result = result.filter((g) => g.Name.toLowerCase().includes(search));
    }

    const sorted = [...result];
    switch (sortMode) {
      case "name":
        sorted.sort((a, b) => a.Name.localeCompare(b.Name, undefined, { sensitivity: "base" }));
        break;
      case "id":
        sorted.sort((a, b) => a.AppId - b.AppId);
        break;
      case "status":
        sorted.sort((a, b) => Number(idlingIds.has(b.AppId)) - Number(idlingIds.has(a.AppId)));
        break;
      default:
        break;
    }
    if (!sortAscending && sortMode !== "none") sorted.reverse();
    return sorted;
  }, [games, searchText, sortMode, sortAscending, idlingIds]);

  const selectedCount = selectedIds.size;
  const idlingCount = idlingIds.size;

  return (
    <div className="app">
      <div className="toolbar">
        <div className="search-box">
          <span className="icon">&#xE721;</span>
          <input
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder={t.SearchPlaceholder}
          />
        </div>
        <button
          className={"icon-button" + (isRefreshing ? " spinning" : "")}
          onClick={() => void refreshLibrary()}
          disabled={isRefreshing}
          title={t.RefreshLibrary}
        >
          &#xE72C;
        </button>
      </div>

      {isIdling && (
        <div className="idle-status">
          <span className="dot" />
          <span>
            {t.IdlingPrefix}
            <strong>
              {idlingCount}/{MAX_SELECTION}
            </strong>
            {t.IdlingGames}
          </span>
        </div>
      )}

      <div className="list-panel">
        <div className="sort-header">
          <button className="sort-status" onClick={() => toggleSort("status")} title={t.SortStatus}>
            &#xE73A;
          </button>
          <button className="sort-name" onClick={() => toggleSort("name")}>
            {t.SortName}
            {sortIcon(sortMode === "name", sortAscending)}
          </button>
          <button className="sort-id" onClick={() => toggleSort("id")}>
            {t.SortId}
            {sortIcon(sortMode === "id", sortAscending)}
          </button>
        </div>

        <div className="game-list">
          {filteredGames.map((game) => (
            <GameRow
              key={game.AppId}
              game={game}
              selected={selectedIds.has(game.AppId)}
              idling={idlingIds.has(game.AppId)}
              onToggle={toggleSelected}
              onStop={stopSingle}
            />
          ))}
          {statusMessage && <div className="status-overlay">{statusMessage}</div>}
        </div>
      </div>

      <div className="bottom-bar">
        <button className="icon-button" onClick={clearSelection} title={t.ClearAll}>
          &#xE894;
        </button>
        <div className="spacer" />
        <button
          className={"primary-button" + (isIdling ? " stop" : "")}
          onClick={toggleIdle}
          disabled={!isIdling && selectedCount === 0}
          title={isIdling ? t.IdleStop : t.IdleStart}
        >
          {isIdling ? "" : ""}
        </button>
      </div>
    </div>
  );
}

function GameRow({
  game,
  selected,
  idling,
  onToggle,
  onStop,
}: {
  game: SteamGame;
  selected: boolean;
  idling: boolean;
  onToggle: (appId: number, checked: boolean) => void;
  onStop: (appId: number) => void;
}) {
  return (
    <div className={"game-row" + (selected ? " selected" : "")}>
      <button
        className={"checkbox" + (selected ? " checked" : "")}
        onClick={() => onToggle(game.AppId, !selected)}
        aria-pressed={selected}
      >
        {selected ? "" : ""}
      </button>
      <div className="game-name">
        {idling && <span className="idling-icon">&#xEA3B;</span>}
        <span className="name-text">{game.Name}</span>
      </div>
      <span className="app-id">{game.AppId}</span>
      {idling && (
        <button className="small-button" onClick={() => onStop(game.AppId)}>
          &#xE71A;
        </button>
      )}
    </div>
  );
}
