dev:
	docker compose up --build

fetch-spec:
	curl -sf http://localhost:3000/openapi.json -o openapi.json

generate: fetch-spec
	cd front-end && pnpm run generate
