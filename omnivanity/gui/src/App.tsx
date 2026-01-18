import { useState, useCallback, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

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

// Chain categories organized by ADDRESS FORMAT (not chain name)
const CHAIN_CATEGORIES: Record<string, string[]> = {
  "🔷 EVM (0x...)": [
    "ETH", "BNB", "MATIC", "ARB", "OP", "AVAX", "FTM", "GNO", "CELO", "ETC",
    "VET", "FLR", "CRO", "MNT", "IMX", "HYPE", "MEMECORE", "MONAD", "IP",
    "LINK", "UNI", "AAVE", "CRV", "LDO", "ETHFI", "AERO", "MORPHO", "ZRO", "ONDO",
    "CAKE", "VIRTUAL", "MYX", "LIT", "USDT", "USDC", "USDe", "DAI", "XAUt", "PAXG",
    "PYUSD", "FDUSD", "TUSD", "USDG", "USD1", "RLUSD", "LEO", "BGB", "OKB", "KCS",
    "GT", "NEXO", "CHZ", "SHIB", "PEPE", "FLOKI", "WLD", "FET", "QNT", "ENA",
    "SKY", "ASTER", "WLFI", "SPX", "CMC20", "XDC", "HBAR"
  ],
  "₿ UTXO (Bitcoin-Like)": [
    "BTC", "LTC", "DOGE", "DASH", "ZEC", "RVN", "DGB", "BCH"
  ],
  "🌙 Solana (Base58)": [
    "SOL", "TRUMP", "BONK", "PENGU", "PUMP", "JUP", "RENDER", "USDT-SPL", "USDC-SPL"
  ],
  "⚛️ Cosmos (Bech32)": [
    "ATOM", "OSMO", "INJ", "SEI", "TIA", "JUNO", "KAVA", "SCRT", "RUNE"
  ],
  "🔴 Polkadot (SS58)": [
    "DOT", "KSM", "ACA", "CFG", "HDX", "TAO"
  ],
  "🅃 TRON (T...)": [
    "TRX", "USDT-TRC20", "USDC-TRC20", "USDD"
  ],
  "⚡ Other L1s": [
    "XRP", "XLM", "APT", "SUI", "NEAR", "IOTA", "ALGO", "FIL", "ZIL", "XNO",
    "TON", "STX", "KAS", "XTZ", "ADA", "XMR", "ICP", "NIGHT"
  ],
};

function App() {
  const [chains, setChains] = useState<ChainInfo[]>([]);
  const [selectedChain, setSelectedChain] = useState<ChainInfo | null>(null);
  const [addressType, setAddressType] = useState<string | null>(null);
  const [pattern, setPattern] = useState("");
  const [patternType, setPatternType] = useState<"prefix" | "suffix" | "contains">("prefix");
  const [caseInsensitive, setCaseInsensitive] = useState(false);
  const [useGpu, setUseGpu] = useState(true); // GPU enabled by default (auto-detect)
  const [isSearching, setIsSearching] = useState(false);
  const [result, setResult] = useState<SearchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<string>("All");

  // Load chains from backend
  useEffect(() => {
    async function loadChains() {
      try {
        const chainList = await invoke<ChainInfo[]>("list_chains");
        setChains(chainList);
        if (chainList.length > 0) {
          setSelectedChain(chainList[0]);
        }
      } catch (e) {
        console.error("Failed to load chains:", e);
      }
    }
    loadChains();
  }, []);

  // Filter chains by search and category
  const filteredChains = useMemo(() => {
    let result = chains;

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      result = result.filter(c =>
        c.ticker.toLowerCase().includes(query) ||
        c.name.toLowerCase().includes(query)
      );
    }

    if (selectedCategory !== "All") {
      const categoryTickers = CHAIN_CATEGORIES[selectedCategory] || [];
      result = result.filter(c => categoryTickers.includes(c.ticker));
    }

    return result;
  }, [chains, searchQuery, selectedCategory]);

  // Get current prefix based on selected address type
  const currentPrefix = useMemo(() => {
    if (!selectedChain) return "";
    if (addressType && selectedChain.addressTypes) {
      const at = selectedChain.addressTypes.find(a => a.id === addressType);
      if (at) return at.prefix;
    }
    return selectedChain.prefix;
  }, [selectedChain, addressType]);

  const startSearch = useCallback(async () => {
    if (!selectedChain || !pattern.trim()) {
      setError("Please select a chain and enter a pattern");
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
        addressType: addressType,
        useGpu: useGpu,
      });

      setResult(searchResult);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsSearching(false);
    }
  }, [selectedChain, pattern, patternType, caseInsensitive, addressType]);

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

  const categories = ["All", ...Object.keys(CHAIN_CATEGORIES)];

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">
          <span className="logo-omni">Omni</span>
          <span className="logo-vanity">Vanity</span>
        </h1>
        <p className="tagline">{chains.length} chains supported</p>
      </header>

      <main className="main">
        {/* Chain Selector with Search */}
        <section className="card chain-selector">
          <div className="chain-header">
            <h2>Select Chain</h2>
            <div className="chain-search">
              <input
                type="text"
                placeholder="🔍 Search chains..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="search-input"
              />
            </div>
          </div>

          {/* Category Tabs */}
          <div className="category-tabs">
            {categories.map(cat => (
              <button
                key={cat}
                className={`category-tab ${selectedCategory === cat ? "active" : ""}`}
                onClick={() => setSelectedCategory(cat)}
              >
                {cat}
              </button>
            ))}
          </div>

          {/* Chain Grid */}
          <div className="chain-grid">
            {filteredChains.map((chain) => (
              <button
                key={chain.ticker}
                className={`chain-btn ${selectedChain?.ticker === chain.ticker ? "active" : ""}`}
                onClick={() => {
                  setSelectedChain(chain);
                  setAddressType(null);
                }}
                disabled={isSearching}
              >
                <span className="chain-ticker">{chain.ticker}</span>
                <span className="chain-name">{chain.name}</span>
                <span className="chain-prefix">{chain.prefix}</span>
              </button>
            ))}
          </div>

          {filteredChains.length === 0 && (
            <p className="no-chains">No chains match your search</p>
          )}
        </section>

        {/* Address Type Selector (for chains with multiple types) */}
        {selectedChain?.addressTypes && selectedChain.addressTypes.length > 1 && (
          <section className="card address-type-selector">
            <h2>Address Type</h2>
            <div className="radio-group">
              {selectedChain.addressTypes.map((at) => (
                <button
                  key={at.id}
                  className={`option-btn ${(addressType === at.id || (!addressType && at.isDefault)) ? "active" : ""}`}
                  onClick={() => setAddressType(at.id)}
                  disabled={isSearching}
                >
                  {at.name}
                </button>
              ))}
            </div>
          </section>
        )}

        {/* Pattern Input */}
        <section className="card pattern-section">
          <h2>Vanity Pattern</h2>
          <div className="pattern-input-group">
            <span className="prefix-hint">{currentPrefix}</span>
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
            </div>
          </div>

          <button
            className={`search-btn ${isSearching ? "searching" : ""}`}
            onClick={isSearching ? stopSearch : startSearch}
            disabled={!selectedChain}
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
        <p>OmniVanity v0.2.0 • {chains.length} Chains • CPU Mode • All keys generated locally</p>
      </footer>
    </div>
  );
}

export default App;
