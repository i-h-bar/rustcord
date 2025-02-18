
create table sets (
    id uuid primary key,
    name varchar(150),
    code varchar(4)
);

create table images (
    id uuid primary key,
    png bytea
);

create table legalities (
    id uuid primary key,
    alchemy varchar(20),
    brawl varchar(20),
    commander varchar(20),
    duel varchar(20),
    explorer varchar(20),
    future varchar(20),
    gladiator varchar(20),
    historic varchar(20),
    legacy varchar(20),
    modern varchar(20),
    oathbreaker varchar(20),
    oldschool varchar(20),
    pauper varchar(20),
    paupercommander varchar(20),
    penny varchar(20),
    pioneer varchar(20),
    predh varchar(20),
    premodern varchar(20),
    standard varchar(20),
    standardbrawl varchar(20),
    timeless varchar(20),
    vintage varchar(20)
);

create table rules (
    id uuid primary key,
    colour_identity char(1)[],
    mana_cost varchar(50),
    cmc smallint,
    power varchar(2),
    toughness varchar(2),
    loyalty varchar(2),
    defence varchar(2),
    type_line varchar(150),
    oracle_text text,
    keywords varchar(20)[],
    legalities_id uuid references legalities(id)
);

create table cards (
    id uuid primary key,
    name varchar(150),
    normalised_name varchar(150),
    flavour_text text,
    set_id uuid references sets(id),
    image_id uuid references images(id),
    rules_id uuid references rules(id),
    artist varchar(50),
    other_side uuid
);

create extension pg_trgm;
