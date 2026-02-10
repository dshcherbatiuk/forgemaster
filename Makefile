.PHONY: build test lint fmt clean check all cluster cluster-clean cluster-reset ui slides help

help:
	@echo "Available commands:"
	@echo "  make all           - fmt, lint, test, build"
	@echo "  make build         - cargo build --release"
	@echo "  make test          - cargo test"
	@echo "  make lint          - cargo clippy"
	@echo "  make fmt           - cargo fmt"
	@echo "  make fmt-check     - cargo fmt --check"
	@echo "  make check         - cargo check"
	@echo "  make clean         - cargo clean"
	@echo "  make cluster       - setup k8s cluster and deploy services"
	@echo "  make cluster-clean - cleanup cluster resources"
	@echo "  make cluster-reset - reset OrbStack k8s cluster"
	@echo "  make ui            - fmt, lint, build UI"
	@echo "  make slides        - generate pitch deck PDF"

all: fmt lint test build

ENV ?= default
ENV_DIR = ansible/environments/$(ENV)

cluster:
	orbctl start
	@echo "⏳ Waiting for Kubernetes to be ready..."
	@until kubectl cluster-info > /dev/null 2>&1; do sleep 2; done
	set -a && . .env.forgemaster && . $(ENV_DIR)/local.env && set +a && ANSIBLE_STDOUT_CALLBACK=debug ansible-playbook -v -i $(ENV_DIR)/inventory ansible/site.yml

cluster-clean:
	set -a && . .env.forgemaster && . $(ENV_DIR)/local.env && set +a && ansible-playbook -i $(ENV_DIR)/inventory ansible/cleanup-cluster.yml

cluster-reset:
	echo "y" | orbctl reset

slides:
	@test -d slides/.venv || python3 -m venv slides/.venv
	@slides/.venv/bin/pip install -q fpdf2 Pillow
	slides/.venv/bin/python slides/generate.py

ui:
	cd ui && npm run fmt && npm run lint && npm run build

build:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check

clean:
	cargo clean
