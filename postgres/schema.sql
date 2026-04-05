--
-- PostgreSQL database dump
--

\restrict SNOsey0ZcWhXXfVTayRYBhLen7nHgcOxQK9ekjHg0paeZOfgsiQzaxn9c6ZgHjF

-- Dumped from database version 18.2 (Debian 18.2-1.pgdg13+1)
-- Dumped by pg_dump version 18.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: update_tag_count(); Type: FUNCTION; Schema: public; Owner: glab
--

CREATE FUNCTION public.update_tag_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE public.tags
        SET post_count = post_count + 1
        WHERE id = NEW.tag_id;

    ELSIF TG_OP = 'DELETE' THEN
        UPDATE public.tags
        SET post_count = post_count - 1
        WHERE id = OLD.tag_id;
    END IF;

    RETURN NULL;
END;
$$;


ALTER FUNCTION public.update_tag_count() OWNER TO glab;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: files; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.files (
    id uuid DEFAULT uuidv7() NOT NULL,
    path text NOT NULL,
    hash text,
    media_type smallint NOT NULL,
    status smallint DEFAULT 0 NOT NULL,
    meta jsonb,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.files OWNER TO glab;

--
-- Name: playlist_items; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.playlist_items (
    id uuid DEFAULT uuidv7() NOT NULL,
    playlist_id uuid NOT NULL,
    post_id uuid,
    note_text text,
    created_at timestamp with time zone DEFAULT now(),
    rank text NOT NULL COLLATE pg_catalog."C"
);


ALTER TABLE public.playlist_items OWNER TO glab;

--
-- Name: playlist_tags; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.playlist_tags (
    playlist_id uuid NOT NULL,
    tag_id uuid NOT NULL
);


ALTER TABLE public.playlist_tags OWNER TO glab;

--
-- Name: playlists; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.playlists (
    id uuid DEFAULT uuidv7() NOT NULL,
    title text NOT NULL,
    description text,
    cover_file_id uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    is_public boolean DEFAULT false NOT NULL,
    owner_id uuid
);


ALTER TABLE public.playlists OWNER TO glab;

--
-- Name: post_notes; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.post_notes (
    id uuid DEFAULT uuidv7() NOT NULL,
    post_id uuid NOT NULL,
    text text NOT NULL,
    pos_x real NOT NULL,
    pos_y real NOT NULL
);


ALTER TABLE public.post_notes OWNER TO glab;

--
-- Name: post_tags; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.post_tags (
    post_id uuid NOT NULL,
    tag_id uuid NOT NULL
);


ALTER TABLE public.post_tags OWNER TO glab;

--
-- Name: posts; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.posts (
    id uuid DEFAULT uuidv7() NOT NULL,
    title text NOT NULL,
    file_id uuid NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.posts OWNER TO glab;

--
-- Name: tag_aliases; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.tag_aliases (
    tag_id uuid NOT NULL,
    alias_id uuid NOT NULL,
    CONSTRAINT tag_aliases_order_check CHECK ((tag_id < alias_id))
);


ALTER TABLE public.tag_aliases OWNER TO glab;

--
-- Name: tag_relation_closure; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.tag_relation_closure (
    ancestor_id uuid NOT NULL,
    descendant_id uuid NOT NULL,
    depth integer NOT NULL,
    CONSTRAINT tag_relation_closure_depth_check CHECK ((depth >= 0))
);


ALTER TABLE public.tag_relation_closure OWNER TO glab;

--
-- Name: tag_relation_edges; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.tag_relation_edges (
    parent_id uuid NOT NULL,
    child_id uuid NOT NULL,
    CONSTRAINT tag_relation_edges_no_self_check CHECK ((parent_id <> child_id))
);


ALTER TABLE public.tag_relation_edges OWNER TO glab;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.tags (
    id uuid DEFAULT uuidv7() NOT NULL,
    name text NOT NULL,
    category smallint DEFAULT 3 NOT NULL,
    post_count integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.tags OWNER TO glab;

--
-- Name: thumbnails; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.thumbnails (
    id uuid DEFAULT uuidv7() NOT NULL,
    file_id uuid NOT NULL,
    path text NOT NULL,
    width integer NOT NULL,
    height integer NOT NULL,
    size_type smallint DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.thumbnails OWNER TO glab;

--
-- Name: users; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.users (
    id uuid DEFAULT uuidv7() NOT NULL,
    username text NOT NULL,
    password_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.users OWNER TO glab;

--
-- Name: files files_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_pkey PRIMARY KEY (id);


--
-- Name: playlist_items playlist_items_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlist_items
    ADD CONSTRAINT playlist_items_pkey PRIMARY KEY (id);


--
-- Name: playlist_tags playlist_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlist_tags
    ADD CONSTRAINT playlist_tags_pkey PRIMARY KEY (playlist_id, tag_id);


--
-- Name: playlists playlists_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlists
    ADD CONSTRAINT playlists_pkey PRIMARY KEY (id);


--
-- Name: post_notes post_notes_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.post_notes
    ADD CONSTRAINT post_notes_pkey PRIMARY KEY (id);


--
-- Name: post_tags post_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT post_tags_pkey PRIMARY KEY (post_id, tag_id);


--
-- Name: posts posts_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_pkey PRIMARY KEY (id);


--
-- Name: tag_aliases tag_aliases_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT tag_aliases_pkey PRIMARY KEY (tag_id, alias_id);


--
-- Name: tag_relation_closure tag_relation_closure_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_relation_closure
    ADD CONSTRAINT tag_relation_closure_pkey PRIMARY KEY (ancestor_id, descendant_id);


--
-- Name: tag_relation_edges tag_relation_edges_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_relation_edges
    ADD CONSTRAINT tag_relation_edges_pkey PRIMARY KEY (parent_id, child_id);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: thumbnails thumbnails_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.thumbnails
    ADD CONSTRAINT thumbnails_pkey PRIMARY KEY (id);


--
-- Name: thumbnails unique_file_size; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.thumbnails
    ADD CONSTRAINT unique_file_size UNIQUE (file_id, size_type);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: idx_playlist_items_order; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_playlist_items_order ON public.playlist_items USING btree (playlist_id, rank);


--
-- Name: idx_post_tags_post_id; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_post_tags_post_id ON public.post_tags USING btree (post_id);


--
-- Name: idx_post_tags_tag_id; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_post_tags_tag_id ON public.post_tags USING btree (tag_id);


--
-- Name: idx_posts_file_id; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_posts_file_id ON public.posts USING btree (file_id);


--
-- Name: idx_tag_aliases_alias_id; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_tag_aliases_alias_id ON public.tag_aliases USING btree (alias_id);


--
-- Name: idx_tag_relation_closure_descendant_id; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_tag_relation_closure_descendant_id ON public.tag_relation_closure USING btree (descendant_id);


--
-- Name: idx_tag_relation_edges_child_id; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_tag_relation_edges_child_id ON public.tag_relation_edges USING btree (child_id);


--
-- Name: idx_tags_category; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX idx_tags_category ON public.tags USING btree (category);


--
-- Name: idx_tags_name_lower; Type: INDEX; Schema: public; Owner: glab
--

CREATE UNIQUE INDEX idx_tags_name_lower ON public.tags USING btree (name, category);


--
-- Name: thumbnails_file_id_idx; Type: INDEX; Schema: public; Owner: glab
--

CREATE INDEX thumbnails_file_id_idx ON public.thumbnails USING btree (file_id);


--
-- Name: post_tags tag_count_trigger; Type: TRIGGER; Schema: public; Owner: glab
--

CREATE TRIGGER tag_count_trigger AFTER INSERT OR DELETE ON public.post_tags FOR EACH ROW EXECUTE FUNCTION public.update_tag_count();


--
-- Name: playlist_items playlist_items_playlist_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlist_items
    ADD CONSTRAINT playlist_items_playlist_id_fkey FOREIGN KEY (playlist_id) REFERENCES public.playlists(id) ON DELETE CASCADE;


--
-- Name: playlist_items playlist_items_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlist_items
    ADD CONSTRAINT playlist_items_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE SET NULL;


--
-- Name: playlist_tags playlist_tags_playlist_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlist_tags
    ADD CONSTRAINT playlist_tags_playlist_id_fkey FOREIGN KEY (playlist_id) REFERENCES public.playlists(id) ON DELETE CASCADE;


--
-- Name: playlist_tags playlist_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlist_tags
    ADD CONSTRAINT playlist_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: playlists playlists_cover_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlists
    ADD CONSTRAINT playlists_cover_file_id_fkey FOREIGN KEY (cover_file_id) REFERENCES public.files(id);


--
-- Name: playlists playlists_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.playlists
    ADD CONSTRAINT playlists_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: post_notes post_notes_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.post_notes
    ADD CONSTRAINT post_notes_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: post_tags post_tags_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT post_tags_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: post_tags post_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.post_tags
    ADD CONSTRAINT post_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: posts posts_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_file_id_fkey FOREIGN KEY (file_id) REFERENCES public.files(id);


--
-- Name: tag_aliases tag_aliases_alias_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT tag_aliases_alias_id_fkey FOREIGN KEY (alias_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_aliases tag_aliases_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT tag_aliases_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_relation_closure tag_relation_closure_ancestor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_relation_closure
    ADD CONSTRAINT tag_relation_closure_ancestor_id_fkey FOREIGN KEY (ancestor_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_relation_closure tag_relation_closure_descendant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_relation_closure
    ADD CONSTRAINT tag_relation_closure_descendant_id_fkey FOREIGN KEY (descendant_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_relation_edges tag_relation_edges_child_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_relation_edges
    ADD CONSTRAINT tag_relation_edges_child_id_fkey FOREIGN KEY (child_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_relation_edges tag_relation_edges_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tag_relation_edges
    ADD CONSTRAINT tag_relation_edges_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: thumbnails thumbnails_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.thumbnails
    ADD CONSTRAINT thumbnails_file_id_fkey FOREIGN KEY (file_id) REFERENCES public.files(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

\unrestrict SNOsey0ZcWhXXfVTayRYBhLen7nHgcOxQK9ekjHg0paeZOfgsiQzaxn9c6ZgHjF
