"""Legacy shim — use valhalla_backbone.experiment() instead."""

from valhalla_backbone import experiment, probe, weights_ok  # noqa: F401

from backbone_registry import Backbone as BackboneSpec  # noqa: F401

# Map old arm names
def run_spec(bb, corpus, prompts, *, skip_missing=True):
    row = experiment(bb, corpus, prompts, skip_missing=skip_missing)
    if "arms" in row:
        arms = row["arms"]
        if "native" in arms:
            arms["valhalla_native"] = arms.pop("native")
        if "hybrid" in arms:
            arms["valhalla_hybrid"] = arms.pop("hybrid")
        if "trad" in arms:
            arms["trad_lm"] = arms.pop("trad")
        if "encoder" in arms:
            arms["valhalla_encoder"] = arms.pop("encoder")
    return row


def probe_weights(bb):
    ok, path = weights_ok(bb)
    return {"weights_ok": ok, "model_path": path if ok else None, "error": path if not ok else None}


def probe_causal(bb):
    return probe(bb)
