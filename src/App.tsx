import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import ContextMenu, { type MenuItem } from "./ContextMenu";
import { detectLanguage, getStrings, type Strings } from "./i18n/strings";
import type { AppSettings, FailureReason, FilterMode, IdleFailure, SortMode, SteamGame } from "./types";
import Titlebar from "./Titlebar";
import "./styles/app.css";

const MAX_SELECTION = 32;
const NOTICE_MS = 6000;
const ICON_CLEAR = "\uE711";
const ICON_STAR = "\uE734";
const ICON_STAR_FILLED = "\uE735";
const lang = detectLanguage();
const t = getStrings(lang);

const FILTERS: { value: FilterMode; label: string }[] = [
  { value: "all", label: t.FilterAll },
  { value: "favorites", label: t.FilterFavorites },
  { value: "idling", label: t.FilterIdling },
];

const FAILURE_TEXT: Record<FailureReason, string> = {
  steamNotRunning: t.ErrSteamNotRunning,
  launchFailed: t.ErrLaunchFailed,
  steamClosed: t.ErrSteamClosed,
  exited: t.ErrExited,
};

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const area = document.createElement("textarea");
    area.value = text;
    document.body.appendChild(area);
    area.select();
    document.execCommand("copy");
    area.remove();
  }
}

function sortIcon(active: boolean, ascending: boolean) {
  if (!active) return "";
  return ascending ? " ↑" : " ↓";
}

export default function App() {
  const [games, setGames] = useState<SteamGame[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [favoriteIds, setFavoriteIds] = useState<Set<number>>(new Set());
  const [filter, setFilter] = useState<FilterMode>("all");
  const [idlingIds, setIdlingIds] = useState<Set<number>>(new Set());
  const [searchText, setSearchText] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("none");
  const [sortAscending, setSortAscending] = useState(true);
  const [statusMessage, setStatusMessage] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [notice, setNotice] = useState("");
  const [menu, setMenu] = useState<{ game: SteamGame; x: number; y: number } | null>(null);

  const settingsRef = useRef<AppSettings>({ Language: lang, SelectedGames: [], Favorites: [], Filter: "all" });
  const pollRef = useRef<number | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const isIdling = idlingIds.size > 0;
  const gameIds = useMemo(() => new Set(games.map((g) => g.AppId)), [games]);
  const gameNames = useMemo(() => new Map(games.map((g) => [g.AppId, g.Name])), [games]);

  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(""), NOTICE_MS);
    return () => window.clearTimeout(timer);
  }, [notice]);

  // Helpers often end a few seconds apart (e.g. when Steam exits), so failures that
  // arrive while the notice is still showing are combined into one message.
  const recentFailuresRef = useRef<{ failure: IdleFailure; at: number }[]>([]);
  const showFailures = useCallback(
    (failures: IdleFailure[]) => {
      if (failures.length === 0) return;
      const now = Date.now();
      const recent = recentFailuresRef.current.filter((f) => now - f.at < NOTICE_MS);
      recent.push(...failures.map((failure) => ({ failure, at: now })));
      recentFailuresRef.current = recent;

      const latest = failures[failures.length - 1];
      const same = recent.filter((f) => f.failure.reason === latest.reason);
      const name = gameNames.get(same[0].failure.appId) ?? String(same[0].failure.appId);
      const others = same.length > 1 ? t.AndOthers.replace("{n}", String(same.length - 1)) : "";
      setNotice(`${name}${others}: ${FAILURE_TEXT[latest.reason]}`);
    },
    [gameNames]
  );

  // The webview's own menu (Reload, Inspect, ...) is not useful here; inputs keep theirs.
  useEffect(() => {
    const onContextMenu = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (target.closest("input, textarea")) return;
      e.preventDefault();
    };
    window.addEventListener("contextmenu", onContextMenu);
    return () => window.removeEventListener("contextmenu", onContextMenu);
  }, []);

  // ── Initial load ──────────────────────────────────────────────────────
  useEffect(() => {
    (async () => {
      const [cache, settings] = await Promise.all([api.loadCache(), api.loadSettings()]);
      settingsRef.current = settings;
      setSelectedIds(new Set(settings.SelectedGames));
      setFavoriteIds(new Set(settings.Favorites));
      if (FILTERS.some((f) => f.value === settings.Filter)) setFilter(settings.Filter);
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
      showFailures(await api.takeIdleFailures());
    }, 1000);
    return () => {
      if (pollRef.current !== null) window.clearInterval(pollRef.current);
      pollRef.current = null;
    };
  }, [isIdling, showFailures]);

  const updateSettings = useCallback((patch: Partial<AppSettings>) => {
    const settings: AppSettings = { ...settingsRef.current, ...patch };
    settingsRef.current = settings;
    void api.saveSettings(settings);
  }, []);

  const changeFilter = useCallback(
    (value: FilterMode) => {
      setFilter(value);
      updateSettings({ Filter: value });
    },
    [updateSettings]
  );

  // Favorites are independent of selection. IDs that vanish from the list are kept,
  // so this only ever adds or removes the clicked ID.
  const toggleFavorite = useCallback(
    (appId: number) => {
      setFavoriteIds((prev) => {
        const next = new Set(prev);
        if (next.has(appId)) next.delete(appId);
        else next.add(appId);
        updateSettings({ Favorites: Array.from(next) });
        return next;
      });
    },
    [updateSettings]
  );

  // ── Selection ────────────────────────────────────────────────────────
  const toggleSelected = useCallback(
    (appId: number, checked: boolean) => {
      if (checked && !selectedIds.has(appId) && selectedIds.size >= MAX_SELECTION) {
        setNotice(t.MaxSelection);
        return;
      }
      setSelectedIds((prev) => {
        if (checked && prev.size >= MAX_SELECTION) return prev; // hard cap, ignore
        const next = new Set(prev);
        if (checked) next.add(appId);
        else next.delete(appId);
        updateSettings({ SelectedGames: Array.from(next) });
        return next;
      });

      if (isIdling) {
        if (checked) {
          void api.startIdle(appId).then((ok) => {
            if (ok) setIdlingIds((prev) => new Set(prev).add(appId));
            else setNotice(`${gameNames.get(appId) ?? appId}: ${t.ErrStartFailed}`);
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
    [isIdling, updateSettings, selectedIds, gameNames]
  );

  // ── Idle start/stop ──────────────────────────────────────────────────
  const startIdling = useCallback(async () => {
    const started = new Set<number>();
    for (const appId of selectedIds) {
      // Saved selections can outlive the game list (e.g. after a library reload).
      if (!gameIds.has(appId)) continue;
      const ok = await api.startIdle(appId);
      if (ok) started.add(appId);
      else setNotice(`${gameNames.get(appId) ?? appId}: ${t.ErrStartFailed}`);
    }
    setIdlingIds(started);
  }, [selectedIds, gameIds, gameNames]);

  const stopAllIdling = useCallback(async () => {
    await api.stopAll();
    setIdlingIds(new Set());
  }, []);

  const toggleIdle = useCallback(() => {
    if (isIdling) void stopAllIdling();
    else void startIdling();
  }, [isIdling, startIdling, stopAllIdling]);

  // ── Refresh ──────────────────────────────────────────────────────────
  const refreshLibrary = useCallback(async () => {
    setIsRefreshing(true);
    // With a list on screen the spinning button is enough; the overlay would cover rows.
    if (games.length === 0) setStatusMessage(t.LoadingLibrary);
    try {
      const result = await api.refreshLibrary();
      setGames(result.cache.Games);
      setStatusMessage("");
      if (!result.connected) setNotice(t.RefreshOffline);
    } catch (e) {
      setStatusMessage(t.LoadError + String(e));
    } finally {
      setIsRefreshing(false);
    }
  }, [games.length]);

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
    if (filter === "favorites") result = result.filter((g) => favoriteIds.has(g.AppId));
    else if (filter === "idling") result = result.filter((g) => idlingIds.has(g.AppId));

    const sorted = [...result];
    switch (sortMode) {
      case "name":
        sorted.sort((a, b) => a.Name.localeCompare(b.Name, undefined, { sensitivity: "base" }));
        break;
      case "id":
        sorted.sort((a, b) => a.AppId - b.AppId);
        break;
      default:
        break;
    }
    if (!sortAscending && sortMode !== "none") sorted.reverse();

    // Checked → favorites → others, each keeping the chosen sort.
    const checked: SteamGame[] = [];
    const favorites: SteamGame[] = [];
    const others: SteamGame[] = [];
    for (const g of sorted) {
      if (selectedIds.has(g.AppId)) checked.push(g);
      else if (favoriteIds.has(g.AppId)) favorites.push(g);
      else others.push(g);
    }
    return checked.concat(favorites, others);
  }, [games, searchText, filter, sortMode, sortAscending, selectedIds, favoriteIds, idlingIds]);

  const emptyMessage =
    filter === "favorites" && !games.some((g) => favoriteIds.has(g.AppId)) ? t.FavoritesEmpty : t.NoResults;
  const overlay = statusMessage || (games.length > 0 && filteredGames.length === 0 ? emptyMessage : "");

  const openMenu = useCallback((game: SteamGame, x: number, y: number) => setMenu({ game, x, y }), []);
  const closeMenu = useCallback(() => setMenu(null), []);

  const menuItems = (game: SteamGame): MenuItem[] => [
    {
      label: favoriteIds.has(game.AppId) ? t.RemoveFavorite : t.AddFavorite,
      onSelect: () => toggleFavorite(game.AppId),
    },
    { label: t.MenuCopyId, onSelect: () => void copyText(String(game.AppId)) },
    { label: t.MenuOpenStore, onSelect: () => void api.openSteamStore(game.AppId) },
    { label: t.MenuOpenLibrary, onSelect: () => void api.openSteamLibrary(game.AppId) },
  ];

  const startableSelected = Array.from(selectedIds).some((id) => gameIds.has(id));
  const idlingCount = idlingIds.size;

  return (
    <div className="app">
      <Titlebar>
        <button
          className={"icon-button primary" + (isIdling ? " stop" : "")}
          onClick={toggleIdle}
          disabled={!isIdling && !startableSelected}
          title={isIdling ? t.IdleStop : t.IdleStart}
        >
          {isIdling ? "" : ""}
        </button>
        <button
          className={"icon-button small" + (isRefreshing ? " spinning" : "")}
          onClick={() => void refreshLibrary()}
          disabled={isRefreshing}
          title={t.RefreshLibrary}
        >
          &#xE72C;
        </button>
      </Titlebar>

      <div className="toolbar">
        <div className="search-box">
          <span className="icon">&#xE721;</span>
          <input
            ref={searchInputRef}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder={t.SearchPlaceholder}
          />
          {searchText && (
            <button
              className="search-clear"
              onClick={() => {
                setSearchText("");
                searchInputRef.current?.focus();
              }}
              title={t.ClearSearch}
              aria-label={t.ClearSearch}
            >
              {ICON_CLEAR}
            </button>
          )}
        </div>
        <select
          className="filter-select"
          value={filter}
          onChange={(e) => changeFilter(e.target.value as FilterMode)}
        >
          {FILTERS.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
      </div>

      <div className="list-panel">
        <div className="sort-header">
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
              favorite={favoriteIds.has(game.AppId)}
              strings={t}
              onToggle={toggleSelected}
              onToggleFavorite={toggleFavorite}
              onContextMenu={openMenu}
            />
          ))}
          {overlay && <div className="status-overlay">{overlay}</div>}
        </div>
      </div>

      <div className="footer">
        <span className="footer-status">
          {t.StatusLabel}
          <strong>
            {idlingCount}/{MAX_SELECTION}
          </strong>
          {t.StatusRunning}
        </span>
        <span className="footer-info" title={notice}>
          {notice}
        </span>
      </div>

      {menu && <ContextMenu x={menu.x} y={menu.y} items={menuItems(menu.game)} onClose={closeMenu} />}
    </div>
  );
}

function GameRow({
  game,
  selected,
  idling,
  favorite,
  strings,
  onToggle,
  onToggleFavorite,
  onContextMenu,
}: {
  game: SteamGame;
  selected: boolean;
  idling: boolean;
  favorite: boolean;
  strings: Strings;
  onToggle: (appId: number, checked: boolean) => void;
  onToggleFavorite: (appId: number) => void;
  onContextMenu: (game: SteamGame, x: number, y: number) => void;
}) {
  return (
    <div
      className={"game-row" + (selected ? " selected" : "")}
      onContextMenu={(e) => {
        e.preventDefault();
        onContextMenu(game, e.clientX, e.clientY);
      }}
    >
      <button
        className={"checkbox" + (selected ? " checked" : "")}
        onClick={() => onToggle(game.AppId, !selected)}
        aria-pressed={selected}
      >
        {selected ? "" : ""}
      </button>
      <button
        className={"fav-button" + (favorite ? " on" : "")}
        onClick={() => onToggleFavorite(game.AppId)}
        title={favorite ? strings.RemoveFavorite : strings.AddFavorite}
        aria-pressed={favorite}
      >
        {favorite ? ICON_STAR_FILLED : ICON_STAR}
      </button>
      <div className="game-name">
        {idling && <span className="idling-icon">&#xEA3B;</span>}
        <span className="name-text">{game.Name}</span>
      </div>
      <span className="app-id">{game.AppId}</span>
    </div>
  );
}
