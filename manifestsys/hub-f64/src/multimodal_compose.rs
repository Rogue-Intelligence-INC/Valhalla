//! Valhalla multimodal compose — TTS + LM via decompose/recompose, not embed concat.
//!
//! v2 `valhalla-compose-v2-structure-recompose`: shards → Stem/Tile/Hub/Triad → LM weight patch (clean prompt).
//! v1 legacy: prompt shard lines (deprecated for LM ingress).

pub const COMPOSE_PROTOCOL: &str = "valhalla-compose-v2-structure-recompose";
pub const COMPOSE_MCQ_PROTOCOL: &str = "valhalla-compose-v2-mcq-structure-recompose";
pub const COMPOSE_PROTOCOL_V1: &str = "valhalla-compose-v1-text-tts-shards";
pub const COMPOSE_MCQ_PROTOCOL_V1: &str = "valhalla-compose-v1-mcq-text-tts-shards";

pub fn compose_enabled() -> bool {
    crate::multimodal::fusion_enabled()
}

pub fn is_structure_recompose_protocol(protocol: &str) -> bool {
    protocol.starts_with("valhalla-compose-v2")
}
