.PHONY: build test lint fmt clean check all cluster cluster-clean cluster-reset help

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
	@echo "  make cluster       - setup OrbStack k8s cluster"
	@echo "  make cluster-clean - cleanup cluster resources"
	@echo "  make cluster-reset - reset OrbStack k8s cluster"

all: fmt lint test build

cluster:
	orbctl start
	@echo "Waiting for Kubernetes to be ready..."
	@until kubectl cluster-info > /dev/null 2>&1; do sleep 2; done
	ansible-playbook ansible/setup-cluster.yml

cluster-clean:
	ansible-playbook ansible/cleanup-cluster.yml

cluster-reset:
	echo "y" | orbctl reset

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
