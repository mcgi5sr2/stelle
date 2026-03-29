-- Add migration script here
Create TABLE pages(
    id                     INTEGER              PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    slug                VARCHAR(128)  NOT NULL UNIQUE,
    title                VARCHAR(512)  NOT NULL,
    body              TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX pages_slug_idx ON pages(slug);