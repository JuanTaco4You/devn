// OmniVanity Generic Pattern Matcher (WGSL)
// Cross-platform GPU pattern matching for vanity address search
// Works with ANY chain - CPU generates addresses, GPU matches patterns

struct MatchParams {
    pattern_len: u32,      // Length of pattern in bytes
    match_type: u32,       // 0=prefix, 1=suffix, 2=contains
    case_insensitive: u32, // 1=ignore case, 0=exact
    num_addresses: u32,    // Number of addresses to check
}

struct MatchResult {
    found: atomic<u32>,    // Set to 1 if any match found
    first_match_idx: atomic<u32>, // Index of first matching address
}

// Input: pattern bytes (up to 32 bytes)
@group(0) @binding(0) var<storage, read> pattern: array<u32>;

// Input: flattened array of address bytes
// Each address is 64 bytes (padded), so address[i] starts at i*16 (in u32s)
@group(0) @binding(1) var<storage, read> addresses: array<u32>;

// Uniform params
@group(0) @binding(2) var<uniform> params: MatchParams;

// Output: match results
@group(0) @binding(3) var<storage, read_write> result: MatchResult;

// Output: array of match flags (1 per address)
@group(0) @binding(4) var<storage, read_write> match_flags: array<u32>;

// Convert hex char to nibble value (works for 0-9, a-f, A-F)
fn hex_to_nibble(c: u32) -> u32 {
    if (c >= 48u && c <= 57u) {  // '0'-'9'
        return c - 48u;
    } else if (c >= 97u && c <= 102u) {  // 'a'-'f'
        return c - 97u + 10u;
    } else if (c >= 65u && c <= 70u) {  // 'A'-'F'
        return c - 65u + 10u;
    }
    return 255u; // Invalid
}

// Get byte at index from u32 array
fn get_byte(arr: ptr<storage, array<u32>, read>, byte_idx: u32) -> u32 {
    let word_idx = byte_idx / 4u;
    let byte_offset = byte_idx % 4u;
    return ((*arr)[word_idx] >> (byte_offset * 8u)) & 0xFFu;
}

// Compare bytes with optional case insensitivity
fn bytes_equal(a: u32, b: u32, case_insensitive: bool) -> bool {
    if (!case_insensitive) {
        return a == b;
    }
    // Make both lowercase for comparison
    var aa = a;
    var bb = b;
    if (aa >= 65u && aa <= 90u) { aa = aa + 32u; }
    if (bb >= 65u && bb <= 90u) { bb = bb + 32u; }
    return aa == bb;
}

@compute @workgroup_size(256)
fn pattern_match(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let addr_idx = global_id.x;
    
    // Bounds check
    if (addr_idx >= params.num_addresses) {
        return;
    }
    
    // Each address is 64 bytes = 16 u32s
    let addr_base = addr_idx * 16u;
    let pattern_len = params.pattern_len;
    let match_type = params.match_type;
    let case_insensitive = params.case_insensitive == 1u;
    
    // Get address length (stored in first byte, or assume 42 for EVM-style)
    // For simplicity, we'll search the full 64 bytes
    let addr_len = 64u;
    
    var matched = false;
    
    if (match_type == 0u) {
        // PREFIX match: compare first pattern_len bytes after any prefix (e.g., "0x")
        // Skip common prefixes: "0x" (2 bytes), "bc1" (3), "addr1" (5), etc.
        // For now, start at byte 0 and let CPU handle prefix stripping
        matched = true;
        for (var i = 0u; i < pattern_len; i = i + 1u) {
            let addr_byte = get_byte(&addresses, addr_base * 4u + i);
            let pat_byte = get_byte(&pattern, i);
            if (!bytes_equal(addr_byte, pat_byte, case_insensitive)) {
                matched = false;
                break;
            }
        }
    } else if (match_type == 1u) {
        // SUFFIX match: compare last pattern_len bytes
        matched = true;
        for (var i = 0u; i < pattern_len; i = i + 1u) {
            let addr_byte = get_byte(&addresses, addr_base * 4u + addr_len - pattern_len + i);
            let pat_byte = get_byte(&pattern, i);
            if (!bytes_equal(addr_byte, pat_byte, case_insensitive)) {
                matched = false;
                break;
            }
        }
    } else {
        // CONTAINS match: sliding window search
        let max_start = addr_len - pattern_len;
        for (var start = 0u; start <= max_start; start = start + 1u) {
            var found = true;
            for (var i = 0u; i < pattern_len; i = i + 1u) {
                let addr_byte = get_byte(&addresses, addr_base * 4u + start + i);
                let pat_byte = get_byte(&pattern, i);
                if (!bytes_equal(addr_byte, pat_byte, case_insensitive)) {
                    found = false;
                    break;
                }
            }
            if (found) {
                matched = true;
                break;
            }
        }
    }
    
    if (matched) {
        match_flags[addr_idx] = 1u;
        atomicStore(&result.found, 1u);
        atomicMin(&result.first_match_idx, addr_idx);
    }
}
