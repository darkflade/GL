CREATE INDEX IF NOT EXISTS idx_files_path ON public.files (path);
CREATE INDEX IF NOT EXISTS idx_files_hash ON public.files (hash);
