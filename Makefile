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
