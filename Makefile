dev:
	docker compose up database --detach --wait
	@sleep 2
	sqlx database setup

down:
	docker compose down

prep:
	@echo "Preparing files for offline sqlx compilation."
	@echo ""
	cargo sqlx prepare

prod:
	docker compose up database --detach --wait
	@sleep 2
	docker compose up telebot --build

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@echo "  dev		- Starts postgres db and migration"
	@echo "  down		- Docker compose down"
	@echo "  prep		- Prepare files for offline sqlx compile verification"
	@echo "  prod		- Runs prod environment locally"
