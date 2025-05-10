-- Your SQL goes here
CREATE TABLE "users" (
	"id" UUID NOT NULL PRIMARY KEY,
	"email" VARCHAR NOT NULL,
	"password_hash" VARCHAR NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);

-- Add admin user with password "change_me"
INSERT INTO
	"users" (id, email, password_hash, created_at, updated_at)
VALUES
	(
		'58e4b30d-cb33-46a6-bf90-164f32b41998', 'admin@mensatt.de', '$argon2id$v=19$m=19456,t=2,p=1$TXHOfe2Zs7rwd/1IgBLKCw$83jzUygPd+8EF5Kp4r/NOjJ661DeXvy/MscVncsFPS4', '2023-07-31 19:08:50.508301+00', '2023-07-31 19:08:50.508302+00'
	);
