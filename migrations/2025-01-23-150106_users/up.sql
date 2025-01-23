-- Your SQL goes here
CREATE TABLE "users" (
	"id" UUID NOT NULL PRIMARY KEY,
	"email" VARCHAR NOT NULL,
	"password_hash" VARCHAR NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);

INSERT INTO
	"users" (id, email, password_hash, created_at, updated_at)
VALUES
	(
		'58e4b30d-cb33-46a6-bf90-164f32b41998', 'admin@mensatt.de', '$2a$10$pdvY6v8k2McSYbFk3HRDl.h8QfMjOxfpm2CywkDDzfOzlYDZV8NUm', '2023-07-31 19:08:50.508301+00', '2023-07-31 19:08:50.508302+00'
	);
