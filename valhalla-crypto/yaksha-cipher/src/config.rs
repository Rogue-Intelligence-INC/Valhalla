//! Runtime throughput profile — `YAKSHA_PROFILE=fast` (default) or `audit`.

/// MAC implementation selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacMode {
    /// Keyed FNV + block cascade (~GB/s class on large payloads).
    Fast,
    /// Fate consistency MAC (Industry Gate / full audit path).
    Audit,
}

/// Global throughput vs audit trade-off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThroughputProfile {
    Fast,
    Audit,
}

/// `YAKSHA_PROFILE`: `fast` (default) | `audit`
pub fn profile() -> ThroughputProfile {
    match std::env::var("YAKSHA_PROFILE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "audit" => ThroughputProfile::Audit,
        _ => ThroughputProfile::Fast,
    }
}

/// `YAKSHA_MAC_MODE` overrides profile when set: `fast` | `audit`
pub fn mac_mode() -> MacMode {
    if let Ok(v) = std::env::var("YAKSHA_MAC_MODE") {
        return match v.to_ascii_lowercase().as_str() {
            "audit" => MacMode::Audit,
            _ => MacMode::Fast,
        };
    }
    match profile() {
        ThroughputProfile::Fast => MacMode::Fast,
        ThroughputProfile::Audit => MacMode::Audit,
    }
}

/// Ocean mixing rounds per stream byte.
/// Fast default: 8. Audit: 48. Override with `YAKSHA_OCEAN_MIX_ROUNDS`.
pub fn ocean_mix_rounds() -> usize {
    if let Ok(v) = std::env::var("YAKSHA_OCEAN_MIX_ROUNDS") {
        if let Ok(n) = v.parse::<usize>() {
            return n.clamp(1, 96);
        }
    }
    match profile() {
        ThroughputProfile::Fast => 8,
        ThroughputProfile::Audit => 48,
    }
}

/// Whether AEAD uses shell (ocean + body). Always on in audit; fast default on.
pub fn shell_enabled() -> bool {
    if let Ok(v) = std::env::var("YAKSHA_SHELL") {
        return !matches!(v.to_ascii_lowercase().as_str(), "0" | "false" | "off");
    }
    true
}
