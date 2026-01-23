--
-- PostgreSQL database dump
--

\restrict yR4sYsImA0tnamPIW0O8DKRbg7kB5YpLyI8XKVhTIsVuAoFvac2qtCNzps6DoJU

-- Dumped from database version 18.1
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
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: files; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.files (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    path text NOT NULL,
    hash text,
    media_type smallint NOT NULL,
    meta jsonb,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.files OWNER TO glab;

--
-- Name: playlist_items; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.playlist_items (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    playlist_id uuid NOT NULL,
    "position" integer NOT NULL,
    post_id uuid,
    note_text text,
    created_at timestamp with time zone DEFAULT now()
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
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    title text NOT NULL,
    description text,
    cover_file_id uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    owner_id uuid
);


ALTER TABLE public.playlists OWNER TO glab;

--
-- Name: post_notes; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.post_notes (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
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
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    title text NOT NULL,
    file_id uuid NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);


ALTER TABLE public.posts OWNER TO glab;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.tags (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    value text NOT NULL,
    category smallint DEFAULT 3 NOT NULL
);


ALTER TABLE public.tags OWNER TO glab;

--
-- Name: users; Type: TABLE; Schema: public; Owner: glab
--

CREATE TABLE public.users (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
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
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: tags tags_value_cat_unique; Type: CONSTRAINT; Schema: public; Owner: glab
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_value_cat_unique UNIQUE (value, category);


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

CREATE INDEX idx_playlist_items_order ON public.playlist_items USING btree (playlist_id, "position");


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
-- PostgreSQL database dump complete
--

\unrestrict yR4sYsImA0tnamPIW0O8DKRbg7kB5YpLyI8XKVhTIsVuAoFvac2qtCNzps6DoJU

