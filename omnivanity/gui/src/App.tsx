import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Chain {
  ticker: string;
  name: string;
  prefix: string;
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

interface SearchStats {
  keysPerSecond: number;
  totalKeys: number;
  probability: number;
  eta: string;
  running: boolean;
}

const CHAINS: Chain[] = [
  { ticker: "ETH", name: "Ethereum", prefix: "0x" },
  { ticker: "BTC", name: "Bitcoin", prefix: "1/bc1" },
  { ticker: "SOL", name: "Solana", prefix: "" },
  { ticker: "LTC", name: "Litecoin", prefix: "L/ltc1" },
  { ticker: "DOGE", name: "Dogecoin", prefix: "D" },
  { ticker: "ZEC", name: "Zcash", prefix: "t1" },
];

function App() {
  const [selectedChain, setSelectedChain] = useState<Chain>(CHAINS[0]);
  const [pattern, setPattern] = useState("");
  const [patternType, setPatternType] = useState<"prefix" | "suffix" | "contains">("prefix");
  const [caseInsensitive, setCaseInsensitive] = useState(false);
  const [isSearching, setIsSearching] = useState(false);
  const [result, setResult] = useState<SearchResult | null>(null);
  const [stats, setStats] = useState<SearchStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showPrivateKey, setShowPrivateKey] = useState(false);

  const startSearch = useCallback(async () => {
    if (!pattern.trim()) {
      setError("Please enter a pattern");
      return;
    }
    
    setError(null);
    setResult(null);
    setIsSearching(true);
    setShowPrivateKey(false);

    try {
      const searchResult = await invoke<SearchResult>("search_vanity", {
        chain: selectedChain.ticker,
        pattern: pattern,
        patternType: patternType,
        caseInsensitive: caseInsensitive,
      });
      
      setResult(searchResult);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsSearching(false);
    }
  }, [selectedChain, pattern, patternType, caseInsensitive]);

  const stopSearch = useCallback(async () => {
    try {
      await invoke("stop_search");
    } catch (e) {
      console.error(e);
    }
    setIsSearching(false);
  }, []);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">
          <span className="logo-omni">Omni</span>
          <span className="logo-vanity">Vanity</span>
        </h1>
        <p className="tagline">Multi-chain vanity wallet generator</p>
      </header>

      <main className="main">
        {/* Chain Selector */}
        <section className="card chain-selector">
          <h2>Select Chain</h2>
          <div className="chain-grid">
            {CHAINS.map((chain) => (
              <button
                key={chain.ticker}
                className={`chain-btn ${selectedChain.ticker === chain.ticker ? "active" : ""}`}
                onClick={() => setSelectedChain(chain)}
                disabled={isSearching}
              >
                <span className="chain-ticker">{chain.ticker}</span>
                <span className="chain-name">{chain.name}</span>
              </button>
            ))}
          </div>
        </section>

        {/* Pattern Input */}
        <section className="card pattern-section">
          <h2>Vanity Pattern</h2>
          <div className="pattern-input-group">
            <span className="prefix-hint">{selectedChain.prefix}</span>
            <input
              type="text"
              className="pattern-input"
              value={pattern}
              onChange={(e) => setPattern(e.target.value)}
              placeholder="Enter pattern..."
              disabled={isSearching}
              maxLength={12}
            />
          </div>

          <div className="pattern-options">
            <div className="option-group">
              <label>Match Type</label>
              <div className="radio-group">
                {["prefix", "suffix", "contains"].map((type) => (
                  <button
                    key={type}
                    className={`option-btn ${patternType === type ? "active" : ""}`}
                    onClick={() => setPatternType(type as any)}
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
          </div>

          <button
            className={`search-btn ${isSearching ? "searching" : ""}`}
            onClick={isSearching ? stopSearch : startSearch}
          >
            {isSearching ? (
              <>
                <span className="spinner"></span>
                Stop Search
              </>
            ) : (
              "🔍 Start Search"
            )}
          </button>
        </section>

        {/* Search Stats */}
        {isSearching && stats && (
          <section className="card stats-section">
            <div className="stat">
              <span className="stat-value">{(stats.keysPerSecond / 1000).toFixed(1)}K</span>
              <span className="stat-label">keys/sec</span>
            </div>
            <div className="stat">
              <span className="stat-value">{stats.totalKeys.toLocaleString()}</span>
              <span className="stat-label">tested</span>
            </div>
            <div className="stat">
              <span className="stat-value">{(stats.probability * 100).toFixed(1)}%</span>
              <span className="stat-label">probability</span>
            </div>
            <div className="stat">
              <span className="stat-value">{stats.eta}</span>
              <span className="stat-label">ETA 50%</span>
            </div>
          </section>
        )}

        {/* Error */}
        {error && (
          <section className="card error-section">
            <p className="error-text">⚠️ {error}</p>
          </section>
        )}

        {/* Result */}
        {result && (
          <section className="card result-section">
            <h2>🎉 Match Found!</h2>
            
            <div className="result-field">
              <label>Address</label>
              <div className="result-value address">
                <code>{result.address}</code>
                <button className="copy-btn" onClick={() => copyToClipboard(result.address)}>
                  📋
                </button>
              </div>
            </div>

            <div className="result-field">
              <label>Private Key (WIF/Native)</label>
              <div className="result-value private-key">
                {showPrivateKey ? (
                  <>
                    <code>{result.privateKeyNative}</code>
                    <button className="copy-btn" onClick={() => copyToClipboard(result.privateKeyNative)}>
                      📋
                    </button>
                  </>
                ) : (
                  <>
                    <code className="hidden">{"•".repeat(32)}</code>
                    <button className="reveal-btn" onClick={() => setShowPrivateKey(true)}>
                      👁️ Reveal
                    </button>
                  </>
                )}
              </div>
            </div>

            <div className="result-stats">
              <span>Tested: {result.keysTestedFormatted}</span>
              <span>Time: {result.timeSecs.toFixed(2)}s</span>
              <span>Speed: {(result.keysPerSecond / 1000).toFixed(1)}K/s</span>
            </div>

            <div className="warning-box">
              ⚠️ <strong>Important:</strong> Copy your private key immediately and store it securely. 
              It will not be saved after closing this window.
            </div>
          </section>
        )}
      </main>

      <footer className="footer">
        <p>OmniVanity v0.1.0 • CPU Mode • All keys generated locally</p>
      </footer>
    </div>
  );
}

export default App;
