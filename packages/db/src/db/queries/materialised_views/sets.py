CREATE = """
        create materialized view set_{set} as
            select set.id                       as set_id,
                   front.id                     as front_id,
                   front.name                   as front_name,
                   front.normalised_name        as front_normalised_name,
                   front.scryfall_url           as front_scryfall_url,
                   front.image_id               as front_image_id,
                   front.illustration_id        as front_illustration_id,
                   front_rule.mana_cost         as front_mana_cost,
                   front_rule.colour_identity   as front_colour_identity,
                   front_rule.power             as front_power,
                   front_rule.toughness         as front_toughness,
                   front_rule.loyalty           as front_loyalty,
                   front_rule.defence           as front_defence,
                   front_rule.type_line         as front_type_line,
                   front_rule.keywords          as front_keywords,
                   front_rule.oracle_text       as front_oracle_text,

                   back.id                      as back_id,
                   back.name                    as back_name,
                   back.scryfall_url            as back_scryfall_url,
                   back.image_id                as back_image_id,
                   back.illustration_id         as back_illustration_id,
                   back_rule.mana_cost          as back_mana_cost,
                   back_rule.colour_identity    as back_colour_identity,
                   back_rule.power              as back_power,
                   back_rule.toughness          as back_toughness,
                   back_rule.loyalty            as back_loyalty,
                   back_rule.defence            as back_defence,
                   back_rule.type_line          as back_type_line,
                   back_rule.keywords           as back_keywords,
                   back_rule.oracle_text        as back_oracle_text,

                   front.release_date           as release_date,
                   artist.name                  as artist,
                   set.name                     as set_name,

                   price.usd                  as usd,
                   price.usd_foil             as usd_foil,
                   price.usd_etched           as usd_etched,
                   price.euro                 as euro,
                   price.euro_foil            as euro_foil,
                   price.tix                  as tix,
                   price.updated_time         as updated_time
            from card front
                     left join card back on front.backside_id = back.id
                     left join rule front_rule on front.oracle_id = front_rule.id
                     left join rule back_rule on back.oracle_id = back_rule.id
                     join set on front.set_id = set.id
                     left join artist on front.artist_id = artist.id
                     left join price on front.id = price.id
            where set.normalised_name = '{normalised_name}';
        """
