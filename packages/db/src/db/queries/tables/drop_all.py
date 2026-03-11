DROP_TABLES = """
DO $$
DECLARE
    tbl RECORD;
BEGIN
    FOR tbl IN
        SELECT tablename, schemaname
        FROM pg_tables
        WHERE schemaname = 'public'
    LOOP
        RAISE NOTICE 'Dropping table: %.%', quote_ident(tbl.schemaname), quote_ident(tbl.tablename);
        EXECUTE format('DROP TABLE %I.%I CASCADE;', tbl.schemaname, tbl.tablename);
    END LOOP;
END;
$$ LANGUAGE plpgsql;
"""

DROP_ALL_TABLES = """
drop table if exists alembic_version;
drop table if exists combo;
drop table if exists price;
drop table if exists related_token;
drop table if exists card;
drop table if exists artist;
drop table if exists image;
drop table if exists illustration;
drop table if exists legality;
drop table if exists rule;
drop table if exists set;
"""
