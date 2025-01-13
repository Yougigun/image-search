export TOOL_HASH := $(shell sha256sum Dockerfile.tool | cut -d' ' -f1 | cut -c1-8)
-include .env

# start database and run migration
.PHONY: start-db
start-db:
	@make build-tool
	docker compose up -d postgres --remove-orphans
	docker compose run migration

# cargo sqlx prepare
.PHONY: sqlx-prepare
sqlx-prepare:
	# @make down
	# @make start-db
	@export DATABASE_URL=postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@localhost:${DATABASE_PORT}/${WEB_SERVER_SERVICE_NAME} && \
	cargo sqlx prepare --workspace -- --bin ${WEB_SERVER_SERVICE_NAME}

# Build the services
.PHONY: build
build:
	@printf '\033[0;34m> Building service images...\033[0m'
	docker compose build
	@make build-tool

# Build tool image and clean up old ones
.PHONY: build-tool
build-tool:
	@printf '\033[0;32m> Checking tool image...\033[0m\n'
	@if ! docker images image-search-tools:$(TOOL_HASH) | grep -q $(TOOL_HASH); then \
		printf '\033[0;32m> Building tool image...\033[0m\n'; \
		docker build --file Dockerfile.tool --tag image-search-tools:$(TOOL_HASH) . ; \
		printf '\033[0;32m> Tool image built: image-search-tools:$(TOOL_HASH)\033[0m\n'; \
	else \
		printf '\033[0;32m> Tool image already exists\033[0m\n'; \
	fi
	@printf '\033[0;32m> Cleaning up old tool images...\033[0m\n'
	@docker images image-search-tools --format "{{.Tag}}" | grep -v $(TOOL_HASH) | xargs -r -I {} docker rmi image-search-tools:{}


# Running all service (no rebuild)
.PHONY: run
run:
	@printf '\033[0;34m> Starting services...\033[0m\n'
	docker compose up --detach --remove-orphans
	# docker compose up

# Shut down all services and clean the volumes and network
.PHONY: down
down:
	@printf '\033[0;34m> Down services...\033[0m\n'
	docker compose down --volumes

# Run tests
.PHONY: test
test:
	@make build-tool
	@printf '\033[0;34m> Run tests...\033[0m\n'
	docker run --rm -i \
		--network image-search-network \
		-v ./tests:/tests \
		image-search-tools:$(TOOL_HASH) /tests/run-test.sh; \

