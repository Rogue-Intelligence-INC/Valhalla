/*
 * ValhallaBase protocol header — valhalla-base-v1
 *
 * Rogue Intelligence Inc. · Valhalla Unified Core
 *
 * This is the JSON protocol contract for ValhallaBase inference.
 * There is NO binary C ABI — integrate via:
 *   1. Subprocess: `valhalla_base` binary (stdin JSON → stdout JSON)
 *   2. Rust crate:  hub_f64::valhalla_base_protocol::execute_request_json
 *
 * Schemas: manifestsys/hub-f64/schemas/valhalla-base-v1.*.schema.json
 * CLI doc: docs/valhalla/VALHALLA_BASE_CLI.md
 */

#ifndef VALHALLA_BASE_H
#define VALHALLA_BASE_H

#ifdef __cplusplus
extern "C" {
#endif

/* ── Protocol identity ─────────────────────────────────────────────── */

#define VALHALLA_BASE_PROTOCOL           "valhalla-base-v1"
#define VALHALLA_BASE_BINARY             "valhalla_base"
#define VALHALLA_BASE_DEFAULT_LM         "Qwen/Qwen2.5-0.5B-Instruct"
#define VALHALLA_BASE_DEFAULT_PATCH_BODY "triad"
#define VALHALLA_BASE_DEFAULT_STRENGTH   0.08
#define VALHALLA_BASE_DEFAULT_MAX_NEW    128
#define VALHALLA_BASE_DEFAULT_CYCLES     2

/* ── decode (config.decode) ──────────────────────────────────────── */

#define VALHALLA_DECODE_NATIVE           "native"
#define VALHALLA_DECODE_LM_PATCH         "lm_patch"
#define VALHALLA_DECODE_HYBRID           "hybrid"          /* production */

/* ── follow_up_decode ─────────────────────────────────────────────── */

#define VALHALLA_FOLLOW_UP_INHERIT                  "inherit"
#define VALHALLA_FOLLOW_UP_LM_PATCH_ON_FOLLOW_UP    "lm_patch_on_follow_up"
#define VALHALLA_FOLLOW_UP_NATIVE_FOLLOW_UP_AWARE   "native_follow_up_aware"

/* ── mcq_decode ───────────────────────────────────────────────────── */

#define VALHALLA_MCQ_LM_PATCH        "lm_patch"
#define VALHALLA_MCQ_STRUCTURE_FATE  "structure_fate"
#define VALHALLA_MCQ_MAX_PICK        "max_pick"

/* ── fate_ingress_routing ─────────────────────────────────────────── */

#define VALHALLA_INGRESS_QUAD_CYCLE          "quad_cycle"   /* production */
#define VALHALLA_INGRESS_ROUND_ROBIN         "round_robin"
#define VALHALLA_INGRESS_SEMANTIC            "semantic"
#define VALHALLA_INGRESS_HYBRID_QUAD_SEMANTIC "hybrid_quad_semantic"

/* ── score_type (questions[].score_type) ───────────────────────────── */

#define VALHALLA_SCORE_OPEN     "open"
#define VALHALLA_SCORE_MCQ      "mcq"
#define VALHALLA_SCORE_NUMERIC  "numeric"

/* ── Request top-level keys ─────────────────────────────────────────
 *
 * {
 *   "context":           ["text line", ...],          // legacy text corpus
 *   "context_items":     [{ "text"|"audio_features"|"image_features" }],
 *   "append_lines":      ["incremental line", ...],
 *   "append_items":      [{ ... }],
 *   "session":           { corpus, fingerprint, lines_ingested, snapshot,
 *                           question_history, fate_baseline_tick },
 *   "questions":         [{ id, text, score_type, reference_answer, keywords }],
 *   "proactive_idle":    false,
 *   "config":            { see config keys below }
 * }
 */

/* ── config keys (production stack) ────────────────────────────────
 *
 *   "decode":                    "hybrid"
 *   "follow_up_decode":          "native_follow_up_aware"
 *   "fate_ingress_routing":      "quad_cycle"
 *   "fate_qa_feedback":          true
 *   "fate_routing_free":         false
 *   "fate_weight_mode":          "fixed"
 *   "mcq_decode":                "lm_patch"
 *   "core_fusion":               false
 *   "proactive_enabled":         true
 *   "proactive_min_fate_signals": 64
 *   "lm_model":                  VALHALLA_BASE_DEFAULT_LM
 *   "patch_body":                "triad"
 *   "strength":                  0.08
 *   "max_new":                   128
 *   "incubation_cycles":         2
 *   "use_cpu":                   false
 */

/* ── Response top-level keys ───────────────────────────────────────
 *
 * {
 *   "protocol":            "valhalla-base-v1",
 *   "context_lines":       N,
 *   "context_fingerprint": "hex",
 *   "decode":              "hybrid",
 *   "session":             { ... round-trip state ... },
 *   "features":            ["hybrid_routing", "session_append", ...],
 *   "rows":                [{ id, answer, decode_path, correct, score, ... }],
 *   "fate_telemetry":      { hub_prefs, hub_routed, ... },
 *   "proactive":           { triggered, nudge, fog_decision, ... }  // optional
 * }
 */

/* ── Error envelope (--json-error on CLI failure) ────────────────────
 *
 * {
 *   "protocol":    "valhalla-base-v1",
 *   "ok":          false,
 *   "error":       "human-readable message",
 *   "error_kind":  "parse"|"validate"|"runtime"|"io"
 * }
 */

/* ── Subprocess integration (any language) ─────────────────────────
 *
 *   valhalla_base ask TEXT [--context LINE]...
 *   valhalla_base demo v1|needle|corpus
 *   valhalla_base serve [--port 8787]
 *   curl -X POST http://127.0.0.1:8787/v1/infer -d @request.json
 *
 * Exit code: 0 success · 1 error
 */

#ifdef __cplusplus
}
#endif

#endif /* VALHALLA_BASE_H */
