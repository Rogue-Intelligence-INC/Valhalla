//! Multimodal fusion ingress (text + audio features → Triad).

use crate::{DlFrontConfig, ModalityInput, TriadSession};

pub fn fusion_enabled() -> bool {
    std::env::var("VALHALLA_MULTIMODAL_FUSION")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        })
        .unwrap_or(false)
}

pub fn feed_audio(
    session: &mut TriadSession,
    audio: Option<&[f64]>,
    front: &DlFrontConfig,
) {
    if !fusion_enabled() {
        return;
    }
    if let Some(vals) = audio.filter(|v| !v.is_empty()) {
        session.feed_modality(
            ModalityInput::AudioFeatures {
                values: vals.to_vec(),
            },
            front,
        );
    }
}

pub fn open_patch_bias_scale() -> f64 {
    std::env::var("VALHALLA_OPEN_PATCH_BIAS_SCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.15)
}
