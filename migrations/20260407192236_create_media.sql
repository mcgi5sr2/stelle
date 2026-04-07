-- Add migration script here
CREATE TYPE media_kind AS ENUM ('image', 'audio', 'video', 'pdf');

CREATE TABLE media (
    id           INTEGER     PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    kind         media_kind  NOT NULL,
    filename     VARCHAR(512) NOT NULL,
    storage_path TEXT        NOT NULL,
    mime_type    VARCHAR(128) NOT NULL,
    file_size    BIGINT      NOT NULL,
    metadata     JSONB       NOT NULL DEFAULT '{}',
    uploaded_by  INTEGER     REFERENCES users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE page_media (
    page_id    INTEGER  NOT NULL REFERENCES pages(id)  ON DELETE CASCADE,
    media_id   INTEGER  NOT NULL REFERENCES media(id)  ON DELETE RESTRICT,
    sort_order SMALLINT NOT NULL DEFAULT 0,
    caption    TEXT,
    PRIMARY KEY (page_id, media_id)
);

CREATE INDEX media_kind_idx ON media(kind);
CREATE INDEX page_media_page_id_idx ON page_media(page_id);
