BEGIN;
CREATE TABLE public.playback_settings (
    user_id uuid NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    collection_id uuid REFERENCES public.collections(id) ON DELETE CASCADE,
    settings jsonb NOT NULL CHECK (jsonb_typeof(settings) = 'object'),
    CONSTRAINT playback_settings_scope UNIQUE NULLS NOT DISTINCT (user_id, collection_id)
);
ALTER TABLE public.playback_settings OWNER TO glab;
COMMIT;
