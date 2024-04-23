dev:
	docker compose up database --detach --wait
	sleep 1
	sqlx database setup

down:
	docker compose down

prep:
	cargo sqlx prepare

prod:
	docker compose up

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@echo "  dev		- Starts postgres db and migration"
	@echo "  down		- Docker compose down"
	@echo "  prep		- Prepare files for offline sqlx compile verification"
	@echo "  prod		- Runs prod environment locally"
