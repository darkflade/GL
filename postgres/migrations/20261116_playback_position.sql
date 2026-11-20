BEGIN;
CREATE TABLE public.collection_playback_positions (
    user_id uuid NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    collection_id uuid NOT NULL REFERENCES public.collections(id) ON DELETE CASCADE,
    item_id uuid REFERENCES public.collection_items(id) ON DELETE SET NULL,
    PRIMARY KEY (user_id, collection_id)
);
ALTER TABLE public.collection_playback_positions OWNER TO glab;
COMMIT;
