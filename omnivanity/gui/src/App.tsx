
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { StatsDashboard, SearchStats } from "./components/StatsDashboard";
import { ActivityLog, LogEntry } from "./components/ActivityLog";

interface AddressTypeInfo {
  id: string;
  name: string;
  prefix: string;
  isDefault: boolean;
}

interface ChainInfo {
  ticker: string;
  name: string;
  prefix: string;
  addressTypes: AddressTypeInfo[];
}

interface SearchResult {
  address: string;
  privateKeyHex: string;
  privateKeyNative: string;
  publicKeyHex: string;
  keysTestedFormatted: string;
  timeSecs: number;
  keysPerSecond: number;
}

function App() {
  const [chains, setChains] = useState<ChainInfo[]>([]);
  const [selectedChain, setSelectedChain] = useState<ChainInfo | null>(null);
  const [pattern, setPattern] = useState("");
  const [patternType, setPatternType] = useState("prefix");
  const [caseInsensitive, setCaseInsensitive] = useState(false);
  const [addressType, setAddressType] = useState<string | null>(null);
  const [isSearching, setIsSearching] = useState(false);
  const [result, setResult] = useState<SearchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState("all");
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [useGpu, setUseGpu] = useState(true);
  const [gpuBatchPower, setGpuBatchPower] = useState(19); // 2^19 = 524,288 (512K)

  // Stats and Logs
  const [stats, setStats] = useState<SearchStats | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  useEffect(() => {
    // Load chains
    invoke<ChainInfo[]>("list_chains").then((data) => {
      setChains(data);
      // Default to ETH
      const eth = data.find((c) => c.ticker === "ETH");
      if (eth) setSelectedChain(eth);
    });

    // Setup event listeners
    const unlistenStats = listen<SearchStats>("search-stats", (event) => {
      setStats(event.payload);
    });

    const unlistenLog = listen<LogEntry>("search-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });

    return () => {
      unlistenStats.then((f) => f());
      unlistenLog.then((f) => f());
    };
  }, []);

  // Reset inputs when chain changes
  useEffect(() => {
    if (selectedChain) {
      setAddressType(null); // Will default to chain default in backend
    }
  }, [selectedChain]);

  const filteredChains = chains.filter((chain) => {
    const matchesSearch =
      chain.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      chain.ticker.toLowerCase().includes(searchQuery.toLowerCase());

    if (!matchesSearch) return false;

    if (activeCategory === "all") return true;

    // Simple category mapping based on ticker/name
    const isEvm = chain.addressTypes.some(t => t.id === "evm");
    const isBtc = ["BTC", "LTC", "DOGE", "BCH"].includes(chain.ticker);
    const isCosmos = chain.addressTypes.some(t => t.id === "cosmos");

    if (activeCategory === "evm") return isEvm;
    if (activeCategory === "bitcoin") return isBtc;
    if (activeCategory === "cosmos") return isCosmos;
    if (activeCategory === "other") return !isEvm && !isBtc && !isCosmos;

    return true;
  });

  async function startSearch() {
    if (!selectedChain) return;

    setIsSearching(true);
    setResult(null);
    setError(null);
    setStats(null);
    setLogs([]); // Clear logs on new search
    setShowPrivateKey(false);

    try {
      // Pass useGpu to backend
      const searchResult = await invoke<SearchResult>("search_vanity", {
        chain: selectedChain.ticker,
        pattern,
        patternType,
        caseInsensitive,
        addressType,
        useGpu: useGpu,
        batchSize: Math.pow(2, gpuBatchPower),
      });
      setResult(searchResult);
    } catch (e) {
      setError(e as string);
    } finally {
      setIsSearching(false);
    }
  }

  async function stopSearch() {
    await invoke("stop_search");
    setIsSearching(false);
  }

  // Get current address prefix for hint
  const currentPrefix = selectedChain
    ? (addressType
      ? selectedChain.addressTypes.find(t => t.id === addressType)?.prefix
      : selectedChain.prefix)
    : "";

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">
          <span className="logo-omni">Omni</span>
          <span className="logo-vanity">Vanity</span>
        </h1>
        <p className="tagline">GPU-Accelerated Multi-Chain Vanity Address Generator</p>
      </header>

      <div className="card">
        <h2>Select Chain</h2>

        <div className="chain-selector">
          <div className="chain-header">
            <input
              type="text"
              placeholder="Search chains..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="search-input chain-search"
            />
          </div>

          <div className="category-tabs">
            {["all", "evm", "bitcoin", "cosmos", "other"].map((cat) => (
              <button
                key={cat}
                className={`category-tab ${activeCategory === cat ? "active" : ""}`}
                onClick={() => setActiveCategory(cat)}
              >
                {cat.charAt(0).toUpperCase() + cat.slice(1)}
              </button>
            ))}
          </div>

          <div className="chain-grid">
            {filteredChains.map((chain) => (
              <button
                key={chain.ticker}
                className={`chain-btn ${selectedChain?.ticker === chain.ticker ? "active" : ""}`}
                onClick={() => setSelectedChain(chain)}
              >
                <div className="chain-ticker">{chain.ticker}</div>
                <div className="chain-name">{chain.name}</div>
                <div className="chain-prefix">{chain.prefix}</div>
              </button>
            ))}
            {filteredChains.length === 0 && (
              <div className="no-chains">No chains found</div>
            )}
          </div>
        </div>
      </div>

      {/* Stats Dashboard (Under Chain Selector) */}
      <StatsDashboard stats={stats} useGpu={useGpu} />

      <div className="card">
        <h2>Pattern Configuration</h2>

        <div className="pattern-input-group">
          <div className="prefix-hint">
            {currentPrefix}
          </div>
          <input
            type="text"
            placeholder="dead..."
            value={pattern}
            onChange={(e) => setPattern(e.target.value)}
            className="pattern-input"
            disabled={isSearching}
          />
        </div>

        <div className="pattern-options">
          <div className="option-group">
            <label>Position</label>
            <div className="radio-group">
              {["prefix", "suffix", "contains"].map((type) => (
                <button
                  key={type}
                  className={`option-btn ${patternType === type ? "active" : ""}`}
                  onClick={() => setPatternType(type)}
                  disabled={isSearching}
                >
                  {type.charAt(0).toUpperCase() + type.slice(1)}
                </button>
              ))}
            </div>
          </div>

          <div className="option-group">
            <label>
              <input
                type="checkbox"
                checked={caseInsensitive}
                onChange={(e) => setCaseInsensitive(e.target.checked)}
                disabled={isSearching}
              />
              Case Insensitive
            </label>
          </div>

          <div className="option-group">
            <label>
              <input
                type="checkbox"
                checked={useGpu}
                onChange={(e) => setUseGpu(e.target.checked)}
                disabled={isSearching}
              />
              🚀 GPU Acceleration
            </label>
            {useGpu && (
              <div className="gpu-slider-container">
                <label className="slider-label">
                  Batch Size: {(() => {
                    const size = Math.pow(2, gpuBatchPower);
                    if (size >= 1048576) return `${(size / 1048576).toFixed(1)}M`;
                    if (size >= 1024) return `${Math.round(size / 1024)}K`;
                    return size.toString();
                  })()} keys
                </label>
                <input
                  type="range"
                  min="12"
                  max="22"
                  step="1"
                  value={gpuBatchPower}
                  onChange={(e) => setGpuBatchPower(parseInt(e.target.value))}
                  disabled={isSearching}
                  className="gpu-slider"
                />
              </div>
            )}
          </div>
        </div>

        <button
          className={`search-btn ${isSearching ? "searching" : ""}`}
          onClick={isSearching ? stopSearch : startSearch}
        >
          {isSearching ? (
            <>
              <div className="spinner"></div>
              Stop Search
            </>
          ) : (
            "Start Search"
          )}
        </button>
      </div>

      {/* Activity Log (Below Search) */}
      <ActivityLog logs={logs} />

      {error && (
        <div className="card error-section">
          <h2>Error</h2>
          <p className="error-text">{error}</p>
        </div>
      )}

      {result && (
        <div className="card result-section">
          <h2>🎉 Address Found!</h2>

          <div className="result-field">
            <label>Address</label>
            <div className="result-value">
              <code>{result.address}</code>
              <button
                className="copy-btn"
                onClick={() => navigator.clipboard.writeText(result.address)}
              >
                Copy
              </button>
            </div>
          </div>

          <div className="result-field">
            <label>Private Key</label>
            <div className="result-value">
              <code className={!showPrivateKey ? "hidden" : ""}>
                {showPrivateKey ? result.privateKeyHex : "••••••••••••••••••••••••••••••••"}
              </code>
              <button
                className="reveal-btn"
                onClick={() => setShowPrivateKey(!showPrivateKey)}
              >
                {showPrivateKey ? "Hide" : "Reveal"}
              </button>
              {showPrivateKey && (
                <button
                  className="copy-btn"
                  onClick={() => navigator.clipboard.writeText(result.privateKeyHex)}
                >
                  Copy
                </button>
              )}
            </div>
          </div>

          <div className="result-stats">
            <span>⏱️ {result.timeSecs.toFixed(2)}s</span>
            <span>🔑 {result.keysTestedFormatted} keys</span>
            <span>⚡ {(result.keysPerSecond / 1000000).toFixed(2)} M/s</span>
          </div>

          <div className="warning-box">
            ⚠️ Never share your private key with anyone!
          </div>
        </div>
      )}

      <footer className="footer">
        <p>OmniVanity v0.1.0 • Built with Rust + Tauri + React</p>
      </footer>
    </div>
  );
}

export default App;
