pub const FUZZY_SEARCH_DISTINCT_CARDS: &str = r"
select distinct on (card.oracle_id)  card.id                   as front_id,
                                     card.oracle_id            as front_oracle_id,
                                     card.name                 as front_name,
                                     card.normalised_name      as front_normalised_name,
                                     card.scryfall_url         as front_scryfall_url,
                                     card.image_id             as front_image_id,
                                     card.illustration_id      as front_illustration_id,
                                     card.backside_id          as back_id,

                                     rule.mana_cost            as front_mana_cost,
                                     rule.colour_identity      as front_colour_identity,
                                     rule.power                as front_power,
                                     rule.toughness            as front_toughness,
                                     rule.loyalty              as front_loyalty,
                                     rule.defence              as front_defence,
                                     rule.type_line            as front_type_line,
                                     rule.keywords             as front_keywords,
                                     rule.oracle_text          as front_oracle_text,

                                     artist.name               as artist,
                                     set.name                  as set_name
from card
         left join rule on card.oracle_id = rule.id
         left join artist on card.artist_id = artist.id
         left join set on set.id = card.set_id
where card.normalised_name % $1
order by card.oracle_id, random() desc;
";

pub const FUZZY_SEARCH_CARD_AND_SET_NAME: &str = r"
select distinct on (card.oracle_id)  card.id                   as front_id,
                                     card.oracle_id            as front_oracle_id,
                                     card.name                 as front_name,
                                     card.normalised_name      as front_normalised_name,
                                     card.scryfall_url         as front_scryfall_url,
                                     card.image_id             as front_image_id,
                                     card.illustration_id      as front_illustration_id,
                                     card.backside_id          as back_id,
                                     rule.mana_cost            as front_mana_cost,
                                     rule.colour_identity      as front_colour_identity,
                                     rule.power                as front_power,
                                     rule.toughness            as front_toughness,
                                     rule.loyalty              as front_loyalty,
                                     rule.defence              as front_defence,
                                     rule.type_line            as front_type_line,
                                     rule.keywords             as front_keywords,

                                     rule.oracle_text          as front_oracle_text,

                                     artist.name               as artist,
                                     set.name                  as set_name,

                                     similarity(set.normalised_name, $2) as set_sml
from card
         left join rule on card.oracle_id = rule.id
         left join artist on card.artist_id = artist.id
         left join set on set.id = card.set_id
where card.normalised_name % $1
  and set.normalised_name % $2
order by card.oracle_id, set_sml, random() desc;
";

pub const FUZZY_SEARCH_CARD_AND_ARTIST: &str = r"
select distinct on (card.oracle_id)  card.id                   as front_id,
                                     card.oracle_id            as front_oracle_id,
                                     card.name                 as front_name,
                                     card.normalised_name      as front_normalised_name,
                                     card.scryfall_url         as front_scryfall_url,
                                     card.image_id             as front_image_id,
                                     card.illustration_id      as front_illustration_id,
                                     card.backside_id          as back_id,
                                     rule.mana_cost            as front_mana_cost,
                                     rule.colour_identity      as front_colour_identity,
                                     rule.power                as front_power,
                                     rule.toughness            as front_toughness,
                                     rule.loyalty              as front_loyalty,
                                     rule.defence              as front_defence,
                                     rule.type_line            as front_type_line,
                                     rule.keywords             as front_keywords,

                                     rule.oracle_text          as front_oracle_text,

                                     artist.name               as artist,
                                     set.name                  as set_name,

                                     similarity(artist.normalised_name, $2) as artist_sml
from card
         left join rule on card.oracle_id = ruke.id
         left join artist on card.artist_id = artist.id
         left join set on set.id = card.set_id
where card.normalised_name % $1
  and artist.normalised_name % $2
order by card.oracle_id, artist_sml, random() desc;
";

pub const FUZZY_SEARCH_SET_NAME: &str = r"
select array_agg(normalised_name)
    from set
where word_similarity(normalised_name, $1) > 0.25
";

pub const NORMALISED_SET_NAME: &str = r"select normalised_name from set where abbreviation = $1";

pub const RANDOM_CARD: &str = r"
select set.id                     as set_id,
       front.id                   as front_id,
       front.oracle_id            as front_oracle_id,
       front.name                 as front_name,
       front.normalised_name      as front_normalised_name,
       front.scryfall_url         as front_scryfall_url,
       front.image_id             as front_image_id,
       front.illustration_id      as front_illustration_id,
       rule.mana_cost             as front_mana_cost,
       rule.colour_identity       as front_colour_identity,
       rule.power                 as front_power,
       rule.toughness             as front_toughness,
       rule.loyalty               as front_loyalty,
       rule.defence               as front_defence,
       rule.type_line             as front_type_line,
       rule.keywords              as front_keywords,
       rule.oracle_text           as front_oracle_text,

       front.backside_id          as back_id,

       front.release_date         as release_date,
       artist.name                as artist,
       set.name                   as set_name
from (select * from card where random() < 0.0001 limit 25) front
         left join rule on front.oracle_id = rule.id
         join set on front.set_id = set.id
         left join artist on front.artist_id = artist.id
where front.illustration_id is not null
order by random()
limit 1;
";

pub const RANDOM_SET_CARD: &str = r"
select set.id                     as set_id,
       front.id                   as front_id,
       front.oracle_id            as front_oracle_id,
       front.name                 as front_name,
       front.normalised_name      as front_normalised_name,
       front.scryfall_url         as front_scryfall_url,
       front.image_id             as front_image_id,
       front.illustration_id      as front_illustration_id,
       rule.mana_cost             as front_mana_cost,
       rule.colour_identity       as front_colour_identity,
       rule.power                 as front_power,
       rule.toughness             as front_toughness,
       rule.loyalty               as front_loyalty,
       rule.defence               as front_defence,
       rule.type_line             as front_type_line,
       rule.keywords              as front_keywords,
       rule.oracle_text           as front_oracle_text,

       front.backside_id          as back_id,

       front.release_date         as release_date,
       artist.name                as artist,
       set.name                   as set_name
from (select * from card where random() < 0.0001 limit 25) front
         left join rule on front.oracle_id = rule.id
         join set on front.set_id = set.id
         left join artist on front.artist_id = artist.id
where front.illustration_id is not null
  and set.normalised_name = $1
order by random()
limit 1;
";

pub const ALL_PRINTS: &str = r"
select card.id as card_id,
       set.name as set_name
       from card left join set on set.id = card.set_id
where card.oracle_id = $1;
";

pub const CARD_FROM_ID: &str = r"
select card.id                   as front_id,
       card.oracle_id            as front_oracle_id,
       card.name                 as front_name,
       card.normalised_name      as front_normalised_name,
       card.scryfall_url         as front_scryfall_url,
       card.image_id             as front_image_id,
       card.illustration_id      as front_illustration_id,
       rule.mana_cost            as front_mana_cost,
       rule.colour_identity      as front_colour_identity,
       rule.power                as front_power,
       rule.toughness            as front_toughness,
       rule.loyalty              as front_loyalty,
       rule.defence              as front_defence,
       rule.type_line            as front_type_line,
       rule.keywords             as front_keywords,
       rule.oracle_text          as front_oracle_text,

       card.backside_id          as back_id,

       artist.name               as artist,
       set.name                  as set_name
from card
         left join rule on card.oracle_id = rule.id
         left join artist on card.artist_id = artist.id
         left join set on set.id = card.set_id
where card.id = $1;;
";
