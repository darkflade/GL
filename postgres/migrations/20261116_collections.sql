-- Run once against an existing database, with the old backend stopped.
BEGIN;
ALTER TABLE public.playlists RENAME TO collections;
ALTER TABLE public.playlist_items RENAME TO collection_items;
ALTER TABLE public.playlist_tags RENAME TO collection_tags;
ALTER TABLE public.collection_items RENAME COLUMN playlist_id TO collection_id;
ALTER TABLE public.collection_tags RENAME COLUMN playlist_id TO collection_id;

-- Retain the visible ordering; id resolves any pre-existing equal ranks.
WITH ordered AS (
    SELECT id, row_number() OVER (PARTITION BY collection_id ORDER BY rank COLLATE "C", id) AS n
    FROM public.collection_items
)
UPDATE public.collection_items AS item
SET rank = '80000000' || lpad(to_hex(ordered.n), 16, '0') || '00000000'
FROM ordered WHERE item.id = ordered.id;

DO $$
DECLARE obj record;
BEGIN
    FOR obj IN
        SELECT conrelid::regclass AS tbl, conname FROM pg_constraint
        WHERE connamespace = 'public'::regnamespace AND conname LIKE '%playlist%'
    LOOP
        EXECUTE format('ALTER TABLE %s RENAME CONSTRAINT %I TO %I', obj.tbl, obj.conname,
                       replace(obj.conname, 'playlist', 'collection'));
    END LOOP;
    FOR obj IN
        SELECT indexname FROM pg_indexes WHERE schemaname = 'public' AND indexname LIKE '%playlist%'
    LOOP
        EXECUTE format('ALTER INDEX public.%I RENAME TO %I', obj.indexname,
                       replace(obj.indexname, 'playlist', 'collection'));
    END LOOP;
END $$;
COMMIT;
