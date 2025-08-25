# 这个文件是 build vnt-dns 用

MODE ?= debug

BINARY_debug=target/debug/vnt-dns
IMAGE_NAME_debug=ghcr.io/middlescale/vnt-dns:debug

BINARY_release=target/release/vnt-dns
IMAGE_NAME_release=ghcr.io/middlescale/vnt-dns:latest

BINARY=$(BINARY_$(MODE))
IMAGE_NAME=$(IMAGE_NAME_$(MODE))

# 可选：从 .env 加载凭据（若存在）
ifneq (,$(wildcard .env))
include .env
export GHCR_TOKEN GHCR_USER
endif

GHCR_USER ?= middlescale

.PHONY: all debug release build docker push login help

all: build

debug:
	$(MAKE) MODE=debug build

release:
	$(MAKE) MODE=release build

build:
ifeq ($(MODE),release)
	cargo build -p vnt-dns --release
else
	cargo build -p vnt-dns
endif

docker: build
	docker build --build-arg BINARY_PATH=$(BINARY) -t $(IMAGE_NAME) .

login:
	@if [ -z "$$GHCR_TOKEN" ]; then echo "GHCR_TOKEN is not set"; exit 1; fi
	echo $$GHCR_TOKEN | docker login ghcr.io -u $(GHCR_USER) --password-stdin

push: build docker login
	docker push $(IMAGE_NAME)

help:
	@echo "环境变量:"
	@echo "  GHCR_USER   (默认: middlescale)"
	@echo "  GHCR_TOKEN  (必填: GitHub PAT，scopes: write:packages, read:packages)"
	@echo ""
	@echo "使用:"
	@echo "  make debug | make release"
	@echo "  可在 .env 中写入 GHCR_USER/GHCR_TOKEN 以免每次设置"
