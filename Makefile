# Valhalla Base — third-party reproduction targets
# Usage: make help

CARGO         ?= cargo
PKG           := hub-f64
FEATURES      := patch-lm
GIT_SHA       := $(shell git rev-parse --short HEAD 2>/dev/null || echo dev)
export VALHALLA_GIT_SHA := $(GIT_SHA)
RUSTFLAGS     ?= -L /opt/cuda/lib64
export RUSTFLAGS

BIN           := target/release/valhalla_base
DASH          := target/release/valhalla_dashboard
PY            ?= python3
REPO          := $(CURDIR)

.PHONY: help build build-all smoke demo serve serve-dash dashboard suite suite-all test-triad highlight clean repro

help:
	@echo "Valhalla Base engineering targets"
	@echo "  make highlight    — one-click pillar test (see scripts/run_highlight_test.sh)"
	@echo "  make build        — valhalla_base release (patch-lm)"
	@echo "  make build-all    — valhalla_base + valhalla_dashboard"
	@echo "  make smoke        — triad smoke via demo v1"
	@echo "  make demo         — alias smoke"
	@echo "  make serve        — HTTP infer API :8787"
	@echo "  make serve-dash   — infer API + Tile/Stem dashboard"
	@echo "  make dashboard    — standalone triad dashboard :8765"
	@echo "  make suite        — fate-core benchmark suite"
	@echo "  make suite-all    — all sequential suites"
	@echo "  make test-triad   — Rust unit tests (fate modules)"
	@echo "  make repro        — full third-party smoke path"
	@echo "  GIT_SHA=$(GIT_SHA)"

build:
	$(CARGO) build -p $(PKG) --release --bin valhalla_base --features $(FEATURES)

build-all: build
	$(CARGO) build -p $(PKG) --release --bin valhalla_dashboard --features $(FEATURES)

smoke demo: build
	$(BIN) demo v1

serve: build
	$(BIN) serve --port $${VALHALLA_BASE_PORT:-8787}

serve-dash: build
	VALHALLA_SERVE_DASHBOARD=1 $(BIN) serve --with-dashboard --port $${VALHALLA_BASE_PORT:-8787}

dashboard: build-all
	$(DASH)

suite: build
	$(PY) tools/valhalla_inference/run_suite_sequential.py --suite fate-core

suite-all: build
	$(PY) tools/valhalla_inference/run_suite_sequential.py --all

test-triad:
	$(CARGO) test -p $(PKG) --lib --features $(FEATURES) -- fate_pretrain_holdout fate_longchain_unified

repro: build build-all smoke suite test-triad
	@echo "→ Reproduction complete. Reports: reports/valhalla_inference/"
	@echo "→ Dashboard: make dashboard  OR  make serve-dash"
	@echo "→ Manifest: curl -s http://127.0.0.1:8787/v1/repro (when serve running)"

make highlight:
	./scripts/run_highlight_test.sh

make highlight-full:
	./scripts/run_highlight_test.sh --full

make full-test:
	./scripts/run_full_valhalla_test.sh --skip-build

incubation-bench:
	.venv-llm/bin/python3 tools/valhalla_inference/run_incubation_llm_benchmarks.py --fast

pareto-matrix:
	.venv-llm/bin/python3 tools/valhalla_inference/run_external_pareto_matrix.py --fast

pareto-matrix-full:
	.venv-llm/bin/python3 tools/valhalla_inference/run_external_pareto_matrix.py --locomo-limit 10 --longbench-limit 10

highlight-train-proof:
	./scripts/run_highlight_test.sh --train-proof

chat-web:
	./scripts/run_valhalla_chat_web.sh

chat-test:
	./scripts/run_valhalla_chat_web.sh --test

clean:
	$(CARGO) clean -p $(PKG)
