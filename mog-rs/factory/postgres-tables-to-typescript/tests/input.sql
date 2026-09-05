--
-- PostgreSQL database dump
--

\restrict 5krVriBmrhFkvowk5M0QgoC3r5cwWoi0f0tQKtzrKdseUnYPr7tpFoJmGjqkxX3

-- Dumped from database version 16.15 (Debian 16.15-1.pgdg13+2)
-- Dumped by pg_dump version 16.15 (Debian 16.15-1.pgdg13+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: actor; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.actor (
    actor_id integer DEFAULT nextval('public.actor_actor_id_seq'::regclass) NOT NULL,
    first_name text NOT NULL,
    last_name text NOT NULL,
    last_update timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: staff; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.staff (
    staff_id integer DEFAULT nextval('public.staff_staff_id_seq'::regclass) NOT NULL,
    first_name text NOT NULL,
    last_name text NOT NULL,
    address_id integer NOT NULL,
    email text,
    store_id integer NOT NULL,
    active boolean DEFAULT true NOT NULL,
    username text NOT NULL,
    password text,
    last_update timestamp with time zone DEFAULT now() NOT NULL,
    picture bytea
);


--
-- Name: widget; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.widget (
    widget_id bigint NOT NULL,
    small_count smallint NOT NULL,
    big_count bigint,
    unit_price numeric(10,2) NOT NULL,
    ratio real,
    precise_ratio double precision,
    sku character varying(45) NOT NULL,
    grade character(1),
    description text,
    is_active boolean NOT NULL,
    created_on date NOT NULL,
    open_time time without time zone,
    made_at timestamp without time zone NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    ext_id uuid NOT NULL,
    payload jsonb,
    meta json,
    thumbnail bytea,
    rating public.mpaa_rating,
    fulltext tsvector,
    special_features text[]
);


--
-- Name: widget_widget_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.widget_widget_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: widget_widget_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.widget_widget_id_seq OWNED BY public.widget.widget_id;


--
-- Name: widget widget_id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.widget ALTER COLUMN widget_id SET DEFAULT nextval('public.widget_widget_id_seq'::regclass);


--
-- Name: actor actor_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.actor
    ADD CONSTRAINT actor_pkey PRIMARY KEY (actor_id);


--
-- Name: staff staff_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.staff
    ADD CONSTRAINT staff_pkey PRIMARY KEY (staff_id);


--
-- Name: idx_actor_last_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_actor_last_name ON public.actor USING btree (last_name);


--
-- Name: actor last_updated; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER last_updated BEFORE UPDATE ON public.actor FOR EACH ROW EXECUTE FUNCTION public.last_updated();


--
-- Name: staff last_updated; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER last_updated BEFORE UPDATE ON public.staff FOR EACH ROW EXECUTE FUNCTION public.last_updated();


--
-- Name: staff staff_address_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.staff
    ADD CONSTRAINT staff_address_id_fkey FOREIGN KEY (address_id) REFERENCES public.address(address_id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- Name: staff staff_store_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.staff
    ADD CONSTRAINT staff_store_id_fkey FOREIGN KEY (store_id) REFERENCES public.store(store_id);


--
-- PostgreSQL database dump complete
--

\unrestrict 5krVriBmrhFkvowk5M0QgoC3r5cwWoi0f0tQKtzrKdseUnYPr7tpFoJmGjqkxX3

