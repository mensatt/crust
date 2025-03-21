-- Your SQL goes here

 
CREATE TABLE "occurrences" (
	"id" UUID NOT NULL PRIMARY KEY,
	"date" TIMESTAMPTZ NOT NULL,
	"kj" BIGINT,
	"kcal" BIGINT,
	"fat" BIGINT,
	"saturated_fat" BIGINT,
	"carbohydrates" BIGINT,
	"sugar" BIGINT,
	"fiber" BIGINT,
	"protein" BIGINT,
	"salt" BIGINT,
	"price_student" BIGINT,
	"price_staff" BIGINT,
	"price_guest" BIGINT,
	"dish" UUID NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
	"location" UUID NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
	"not_available_after" TIMESTAMPTZ,
	"status" VARCHAR DEFAULT 'AWAITING_APPROVAL' NOT NULL
);

