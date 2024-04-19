dev:
	docker compose up database --detach --wait

down:
	docker compose down

prep:
	cargo sqlx prepare

prod:
	docker compose up
