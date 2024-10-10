
create table sets(
    id uuid primary key,
    name varchar(150),
    code char(3)
);

create table images(
    id uuid primary key,
    png bytea
);

create table cards(
    id uuid primary key,
    name varchar(150),
    flavour_text text,
    set_id uuid references sets(id),
    image_id uuid references images(id),
    artist varchar(50)
);
