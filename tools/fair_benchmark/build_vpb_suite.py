#!/usr/bin/env python3
"""Build VPB Phase B memory suite + Phase C corpus tiers (S/M/L)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
FAIR_SUITE = FAIR / "fair_benchmark_suite.json"
OUT_SUITE = FAIR / "vpb_memory_suite.json"
OUT_TIERS = FAIR / "vpb_corpus_tiers.json"


def load_fair() -> dict:
    if not FAIR_SUITE.exists():
        subprocess_run_build()
    return json.loads(FAIR_SUITE.read_text(encoding="utf-8"))


def subprocess_run_build() -> None:
    import subprocess

    subprocess.run([sys.executable, str(FAIR / "build_fair_suite.py")], check=True)


def build_corpus_tiers(fair: dict) -> dict:
    from contamination import filter_clean_corpus, filter_padding_lines

    m = fair["corpus"]["ingest_clean"]
    s = m[:22]

    legacy = REPO / "tools/valhalla_model_bridge/incubation_benchmark_suite.json"
    large = json.loads(legacy.read_text(encoding="utf-8"))["corpus"]["large"]
    prompts = fair["prompts"]
    clean_large, _ = filter_clean_corpus(large, prompts)
    seen = set(m)
    extra = []
    for line in filter_padding_lines(clean_large):
        if line not in seen:
            seen.add(line)
            extra.append(line)
    # Additional clean domain lines from large tail (no numeric eval leaks)
    for line in large:
        if line.startswith("Incubation knowledge line") and line not in seen:
            seen.add(line)
            extra.append(line)
    l = m + extra[:71]  # target ~120

    # Knowledge-update lines (L-only extensions)
    updates = [
        "Revision note 2026: Mercury remains the smallest planet after Pluto reclassification.",
        "Updated timeline: World War Two in Europe ended May 1945; Pacific war ended August 1945.",
        "Policy update: Canberra remains the capital of Australia; Sydney is the largest city.",
    ]
    for u in updates:
        if u not in seen:
            l.append(u)
            seen.add(u)

    return {
        "meta": {
            "version": "vpb-corpus-v1",
            "tiers": {"S": len(s), "M": len(m), "L": len(l)},
            "description": "LONGMEMEVAL-style scale axis; all tiers decontaminated",
        },
        "tiers": {
            "S": s,
            "M": m,
            "L": l,
        },
        "line_index_M": {f"c{i:03d}": line for i, line in enumerate(m)},
    }


def memory_prompts() -> list[dict]:
    """80 VPB memory-taxonomy prompts (LoCoMo / LONGMEMEVAL aligned)."""
    P: list[dict] = []

    def add(
        pid: str,
        task: str,
        text: str,
        ref: str,
        kws: list[str],
        evidence: list[str],
        *,
        score_type: str = "open",
        abstention: bool = False,
        split: str = "test",
    ) -> None:
        P.append(
            {
                "id": pid,
                "memory_task": task,
                "domain": "vpb_memory",
                "source": "vpb_synthetic",
                "text": text,
                "reference_answer": ref,
                "keywords": kws,
                "score_type": score_type,
                "evidence_line_ids": evidence,
                "abstention_expected": abstention,
                "split": split,
            }
        )

    # --- single_hop (20) ---
    singles = [
        ("VPB_SH_01", "What is the capital of Australia?", "Canberra", ["canberra"], ["c002"]),
        ("VPB_SH_02", "When did World War Two end?", "1945", ["1945"], ["c001"]),
        ("VPB_SH_03", "What organelle performs photosynthesis?", "chloroplast", ["chloroplast"], ["c000"]),
        ("VPB_SH_04", "Who was the first US President?", "Washington", ["washington"], ["c018"]),
        ("VPB_SH_05", "What vitamin deficiency causes scurvy?", "vitamin C", ["vitamin c", "ascorbic"], ["c007"]),
        ("VPB_SH_06", "What is the boiling point of water at standard pressure in Celsius?", "100", ["100"], ["c010"]),
        ("VPB_SH_07", "Who wrote Romeo and Juliet?", "Shakespeare", ["shakespeare"], ["c009"]),
        ("VPB_SH_08", "What is the largest ocean?", "Pacific", ["pacific"], ["c011"]),
        ("VPB_SH_09", "What is the SI unit of force?", "Newton", ["newton"], ["c024"]),
        ("VPB_SH_10", "Who wrote Nineteen Eighty-Four?", "Orwell", ["orwell"], ["c031"]),
        ("VPB_SH_11", "What is the capital of France?", "Paris", ["paris"], ["c036"]),
        ("VPB_SH_12", "Which planet is closest to the Sun?", "Mercury", ["mercury"], ["c017"]),
        ("VPB_SH_13", "DNA uses how many nucleotide bases?", "four", ["four", "4"], ["c004"]),
        ("VPB_SH_14", "When did the French Revolution begin?", "1789", ["1789"], ["c013"]),
        ("VPB_SH_15", "Who is credited with Dream of the Red Chamber?", "Cao Xueqin", ["cao", "曹雪芹"], ["c033"]),
        ("VPB_SH_16", "Are whales fish or mammals?", "mammals", ["mammal"], ["c035"]),
        ("VPB_SH_17", "What does GDP measure?", "goods and services", ["gdp", "goods"], ["c019"]),
        ("VPB_SH_18", "What is binary search time complexity?", "O log n", ["log n", "olog"], ["c026"]),
        ("VPB_SH_19", "Where does the Yangtze River originate?", "Tibetan Plateau", ["tibet", "yangtze"], ["c008"]),
        ("VPB_SH_20", "How many solar terms in the Chinese calendar?", "twenty four", ["24", "twenty"], ["c032"]),
    ]
    for pid, q, ref, kws, ev in singles:
        add(pid, "single_hop", q, ref, kws, ev, split="test" if int(pid[-2:]) <= 16 else "smoke")

    # --- multi_hop (15) ---
    hops = [
        (
            "VPB_MH_01",
            "Photosynthesis converts CO2 and water into glucose using sunlight — in which organelle?",
            "chloroplast",
            ["chloroplast", "chloroplasts"],
            ["c000", "c025"],
        ),
        (
            "VPB_MH_02",
            "Newton's second law relates force to mass and acceleration — what is the SI unit of force?",
            "Newton",
            ["newton"],
            ["c003", "c024"],
        ),
        (
            "VPB_MH_03",
            "Scurvy involves bleeding gums — which vitamin prevents it?",
            "vitamin C",
            ["vitamin c"],
            ["c007", "c042"],
        ),
        (
            "VPB_MH_04",
            "Aristotle studied under Plato — who tutored Alexander the Great?",
            "Aristotle",
            ["aristotle"],
            ["c027", "c037"],
        ),
        (
            "VPB_MH_05",
            "GDP sums consumption and investment — what does GDP stand for measuring?",
            "goods and services",
            ["gdp", "goods"],
            ["c019", "c039"],
        ),
        (
            "VPB_MH_06",
            "Binary search needs sorted arrays — what is its time complexity?",
            "O log n",
            ["log n"],
            ["c006", "c026"],
        ),
        (
            "VPB_MH_07",
            "Canberra was chosen as compromise capital — what country is it capital of?",
            "Australia",
            ["australia"],
            ["c002", "c044"],
        ),
        (
            "VPB_MH_08",
            "Mitochondria produce ATP — in what type of cells?",
            "eukaryotes",
            ["eukaryote"],
            ["c014", "c025"],
        ),
        (
            "VPB_MH_09",
            "Electrons orbit the nucleus — what charge do electrons carry?",
            "negative",
            ["negative"],
            ["c004", "c024"],
        ),
        (
            "VPB_MH_10",
            "World War II ran 1939-1945 — in what year did WWII end?",
            "1945",
            ["1945"],
            ["c041", "c001"],
        ),
        (
            "VPB_MH_11",
            "Supply and demand intersect at equilibrium — supply increases when what rises?",
            "price",
            ["price"],
            ["c005", "c040"],
        ),
        (
            "VPB_MH_12",
            "CPU executes instructions — name one other core hardware component.",
            "memory",
            ["memory", "storage"],
            ["c038", "c041"],
        ),
        (
            "VPB_MH_13",
            "Plato wrote The Republic — who was his famous student later tutoring Alexander?",
            "Aristotle",
            ["aristotle"],
            ["c027", "c037"],
        ),
        (
            "VPB_MH_14",
            "Inflation is price level increase — GDP measures total value of what?",
            "goods and services",
            ["goods"],
            ["c016", "c019"],
        ),
        (
            "VPB_MH_15",
            "Mercury is smallest planet — how long is its year in Earth days (about)?",
            "eighty eight",
            ["88", "eighty"],
            ["c017", "c045"],
        ),
    ]
    for i, (pid, q, ref, kws, ev) in enumerate(hops):
        add(pid, "multi_hop", q, ref, kws, ev, split="smoke" if i < 3 else "test")

    # --- temporal (10) ---
    temps = [
        ("VPB_TP_01", "Did WWII end before or after 1940?", "after", ["after", "1945"], ["c001"]),
        ("VPB_TP_02", "Was the French Revolution before the US Civil War era (1860s)?", "before", ["before", "1789"], ["c013"]),
        ("VPB_TP_03", "Did George Washington serve before Napoleon rose?", "before", ["before"], ["c018"]),
        ("VPB_TP_04", "Was Nineteen Eighty-Four published after WWII?", "after", ["after", "1949"], ["c031"]),
        ("VPB_TP_05", "Does Mercury orbit faster than Earth (shorter year)?", "yes", ["yes", "88"], ["c045"]),
        ("VPB_TP_06", "Did WWII involve years after 1939?", "yes", ["yes", "1945"], ["c041"]),
        ("VPB_TP_07", "Was Thomas Jefferson active before Orwell wrote 1984?", "before", ["before"], ["c030", "c031"]),
        ("VPB_TP_08", "Did the French Revolution occur in the 18th century?", "yes", ["yes", "1789"], ["c013"]),
        ("VPB_TP_09", "Is WWII end year 1945?", "yes", ["1945"], ["c001", "c041"]),
        ("VPB_TP_10", "Was Washington president in the 18th century?", "yes", ["yes", "1700"], ["c018"]),
    ]
    for i, row in enumerate(temps):
        add(row[0], "temporal", row[1], row[2], row[3], row[4], split="smoke" if i < 2 else "test")

    # --- knowledge_update (10) — L-tier adds revision lines; M-tier uses dual evidence ---
    ku_items = [
        ("VPB_KU_01", "After Pluto reclassification, what is still the smallest planet?", "Mercury", ["mercury"], ["c017"]),
        ("VPB_KU_02", "When did WWII end according to corpus (year only)?", "1945", ["1945"], ["c001", "c041"]),
        ("VPB_KU_03", "What remains the capital of Australia?", "Canberra", ["canberra"], ["c002", "c044"]),
        ("VPB_KU_04", "Does Mercury remain the smallest planet in the solar system?", "yes", ["yes", "mercury"], ["c017"]),
        ("VPB_KU_05", "Did WWII end in 1945?", "yes", ["1945"], ["c001"]),
        ("VPB_KU_06", "Is Canberra still the Australian capital?", "yes", ["canberra"], ["c002"]),
        ("VPB_KU_07", "Is Sydney the capital of Australia?", "no", ["no", "canberra"], ["c002"]),
        ("VPB_KU_08", "Was WWII over by 1946?", "yes", ["yes", "1945"], ["c001"]),
        ("VPB_KU_09", "Is Mercury closer to the Sun than Earth?", "yes", ["yes", "mercury"], ["c017", "c045"]),
        ("VPB_KU_10", "Did Japan surrender marking WWII end in 1945?", "yes", ["1945"], ["c021", "c001"]),
    ]
    for i, (pid, q, ref, kws, ev) in enumerate(ku_items):
        add(pid, "knowledge_update", q, ref, kws, ev, split="smoke" if i < 2 else "test")

    # --- abstention (15) ---
    abst = [
        ("VPB_AB_01", "What is the capital of Brazil?", "", ["brasilia", "brazil"]),
        ("VPB_AB_02", "Who won the FIFA World Cup in 2022?", "", ["argentina", "2022"]),
        ("VPB_AB_03", "What is the speed of light in m/s?", "", ["299", "speed of light"]),
        ("VPB_AB_04", "Who discovered penicillin?", "", ["fleming", "penicillin"]),
        ("VPB_AB_05", "What is the population of Tokyo?", "", ["tokyo", "million"]),
        ("VPB_AB_06", "When was the iPhone first released?", "", ["2007", "iphone"]),
        ("VPB_AB_07", "What is the chemical symbol for gold?", "", ["au", "gold"]),
        ("VPB_AB_08", "Who painted the Mona Lisa?", "", ["vinci", "mona lisa"]),
        ("VPB_AB_09", "What is the tallest mountain in the world?", "", ["everest"]),
        ("VPB_AB_10", "What programming language was Rust inspired by?", "", ["rust", "c++"]),
        ("VPB_AB_11", "What year did the Berlin Wall fall?", "", ["1989", "berlin"]),
        ("VPB_AB_12", "What is the largest mammal?", "", ["blue whale"]),
        ("VPB_AB_13", "Who developed the theory of relativity?", "", ["einstein"]),
        ("VPB_AB_14", "What is the capital of Japan?", "", ["tokyo"]),
        ("VPB_AB_15", "What is Planck's constant?", "", ["planck", "constant"]),
    ]
    for i, (pid, q, ref, kws) in enumerate(abst):
        add(pid, "abstention", q, ref, kws, [], abstention=True, split="smoke" if i < 3 else "test")

    # --- adversarial (10) ---
    adv = [
        (
            "VPB_AD_01",
            "What is the capital of Australia — Sydney or Canberra?",
            "Canberra",
            ["canberra"],
            ["c002", "c044"],
        ),
        (
            "VPB_AD_02",
            "Is Sydney the capital of Australia?",
            "no",
            ["no", "canberra"],
            ["c002", "c044"],
        ),
        (
            "VPB_AD_03",
            "Is Pluto the smallest planet?",
            "no",
            ["no", "mercury"],
            ["c017", "c045"],
        ),
        (
            "VPB_AD_04",
            "Does scurvy come from excess vitamin C?",
            "no",
            ["no", "deficiency"],
            ["c007", "c042"],
        ),
        (
            "VPB_AD_05",
            "Is the Nile in Asia?",
            "no",
            ["no", "africa"],
            ["c029"],
        ),
        (
            "VPB_AD_06",
            "Is Paris in Germany?",
            "no",
            ["no", "france"],
            ["c036"],
        ),
        (
            "VPB_AD_07",
            "Does photosynthesis produce carbon dioxide as main product?",
            "no",
            ["no", "glucose"],
            ["c000"],
        ),
        (
            "VPB_AD_08",
            "Is binary search O(n) on sorted arrays?",
            "no",
            ["no", "log"],
            ["c026"],
        ),
        (
            "VPB_AD_09",
            "Was WWII only in Europe?",
            "no",
            ["no", "pacific", "japan"],
            ["c001", "c021"],
        ),
        (
            "VPB_AD_10",
            "Is Mercury the largest planet?",
            "no",
            ["no", "smallest"],
            ["c017"],
        ),
    ]
    for i, (pid, q, ref, kws, ev) in enumerate(adv):
        add(pid, "adversarial", q, ref, kws, ev, split="smoke" if i < 2 else "test")

    assert len(P) == 80, f"expected 80 prompts, got {len(P)}"
    return P


def build_suite() -> dict:
    fair = load_fair()
    tiers_doc = build_corpus_tiers(fair)
    prompts = memory_prompts()

    splits: dict[str, list[str]] = {"smoke": [], "test": []}
    for p in prompts:
        splits[p.pop("split")].append(p["id"])

    by_task: dict[str, int] = {}
    for p in prompts:
        by_task[p["memory_task"]] = by_task.get(p["memory_task"], 0) + 1

    return {
        "meta": {
            "version": "vpb-memory-v1",
            "name": "Valhalla Potential Benchmark — Memory Taxonomy",
            "prompt_count": len(prompts),
            "memory_task_counts": by_task,
            "methodology": [
                "LoCoMo QA taxonomy (single/multi-hop/temporal/adversarial)",
                "LONGMEMEVAL abstention + evidence_line_ids",
                "Decontaminated — abstention items have no corpus evidence",
            ],
        },
        "prompts": prompts,
        "splits": splits,
        "corpus_tiers_ref": "vpb_corpus_tiers.json",
        "phases": {
            "smoke": {"split": "smoke", "corpus_tier": "M"},
            "test": {"split": "test", "corpus_tier": "M"},
        },
    }, tiers_doc


def main() -> None:
    suite, tiers = build_suite()
    OUT_SUITE.write_text(json.dumps(suite, indent=2, ensure_ascii=False), encoding="utf-8")
    OUT_TIERS.write_text(json.dumps(tiers, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Wrote {OUT_SUITE} ({suite['meta']['prompt_count']} prompts)", file=sys.stderr)
    print(f"Wrote {OUT_TIERS} tiers S/M/L = {tiers['meta']['tiers']}", file=sys.stderr)
    print(f"Tasks: {suite['meta']['memory_task_counts']}", file=sys.stderr)


if __name__ == "__main__":
    main()
