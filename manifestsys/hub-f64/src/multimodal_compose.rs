//! Valhalla multimodal compose — TTS + LM via decompose/recompose, not embed concat.
//!
//! Protocol `valhalla-compose-v1-text-tts-shards`:
//!   TEXT_LM_QUERY (Stem) + AUDIO_TTS_BAND_* (Tile) → CROSS_MODAL_FATE (Hub) → LM_RECOMPOSE (Triad)

pub const COMPOSE_PROTOCOL: &str = "valhalla-compose-v1-text-tts-shards";

pub fn compose_enabled() -> bool {
    crate::multimodal::fusion_enabled()
}
