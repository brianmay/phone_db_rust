--
-- PostgreSQL database dump
--

--
-- Name: contacts; Type: TABLE; Schema: public; Owner: phone_db
--

CREATE TABLE public.contacts (
    id bigint NOT NULL,
    phone_number character varying(255) NOT NULL,
    name character varying(255),
    action character varying(255) NOT NULL,
    inserted_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    comments character varying(255)
);


ALTER TABLE public.contacts OWNER TO phone_db;

--
-- Name: contacts_id_seq; Type: SEQUENCE; Schema: public; Owner: phone_db
--

CREATE SEQUENCE public.contacts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.contacts_id_seq OWNER TO phone_db;

--
-- Name: contacts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: phone_db
--

ALTER SEQUENCE public.contacts_id_seq OWNED BY public.contacts.id;


--
-- Name: defaults; Type: TABLE; Schema: public; Owner: phone_db
--

CREATE TABLE public.defaults (
    id bigint NOT NULL,
    "order" integer,
    regexp character varying(255),
    name character varying(255),
    action character varying(255),
    inserted_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);


ALTER TABLE public.defaults OWNER TO phone_db;

--
-- Name: defaults_id_seq; Type: SEQUENCE; Schema: public; Owner: phone_db
--

CREATE SEQUENCE public.defaults_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.defaults_id_seq OWNER TO phone_db;

--
-- Name: defaults_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: phone_db
--

ALTER SEQUENCE public.defaults_id_seq OWNED BY public.defaults.id;


--
-- Name: phone_calls; Type: TABLE; Schema: public; Owner: phone_db
--

CREATE TABLE public.phone_calls (
    id bigint NOT NULL,
    action character varying(255) NOT NULL,
    contact_id bigint NOT NULL,
    inserted_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    destination_number character varying(255)
);


ALTER TABLE public.phone_calls OWNER TO phone_db;

--
-- Name: phone_calls_id_seq; Type: SEQUENCE; Schema: public; Owner: phone_db
--

CREATE SEQUENCE public.phone_calls_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.phone_calls_id_seq OWNER TO phone_db;

--
-- Name: phone_calls_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: phone_db
--

ALTER SEQUENCE public.phone_calls_id_seq OWNED BY public.phone_calls.id;


--
-- Name: schema_migrations; Type: TABLE; Schema: public; Owner: phone_db
--

CREATE TABLE public.schema_migrations (
    version bigint NOT NULL,
    inserted_at timestamp(0) without time zone
);


ALTER TABLE public.schema_migrations OWNER TO phone_db;

--
-- Name: users; Type: TABLE; Schema: public; Owner: phone_db
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    username character varying(255),
    password_hash character varying(255),
    is_admin boolean DEFAULT false,
    is_trusted boolean DEFAULT false,
    is_phone boolean DEFAULT false,
    inserted_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);


ALTER TABLE public.users OWNER TO phone_db;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: phone_db
--

CREATE SEQUENCE public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO phone_db;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: phone_db
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: contacts id; Type: DEFAULT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.contacts ALTER COLUMN id SET DEFAULT nextval('public.contacts_id_seq'::regclass);


--
-- Name: defaults id; Type: DEFAULT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.defaults ALTER COLUMN id SET DEFAULT nextval('public.defaults_id_seq'::regclass);


--
-- Name: phone_calls id; Type: DEFAULT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.phone_calls ALTER COLUMN id SET DEFAULT nextval('public.phone_calls_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: contacts contacts_pkey; Type: CONSTRAINT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.contacts
    ADD CONSTRAINT contacts_pkey PRIMARY KEY (id);


--
-- Name: defaults defaults_pkey; Type: CONSTRAINT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.defaults
    ADD CONSTRAINT defaults_pkey PRIMARY KEY (id);


--
-- Name: phone_calls phone_calls_pkey; Type: CONSTRAINT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.phone_calls
    ADD CONSTRAINT phone_calls_pkey PRIMARY KEY (id);


--
-- Name: schema_migrations schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.schema_migrations
    ADD CONSTRAINT schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: contacts_phone_number_index; Type: INDEX; Schema: public; Owner: phone_db
--

CREATE UNIQUE INDEX contacts_phone_number_index ON public.contacts USING btree (phone_number);


--
-- Name: users_username_index; Type: INDEX; Schema: public; Owner: phone_db
--

CREATE UNIQUE INDEX users_username_index ON public.users USING btree (username);


--
-- Name: phone_calls phone_calls_contact_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: phone_db
--

ALTER TABLE ONLY public.phone_calls
    ADD CONSTRAINT phone_calls_contact_id_fkey FOREIGN KEY (contact_id) REFERENCES public.contacts(id);


--
-- PostgreSQL database dump complete
--
