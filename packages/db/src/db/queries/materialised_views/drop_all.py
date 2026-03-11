DROP_MAT_VIEWS = """
DO $$
DECLARE
    -- Variable to hold the name of each materialized view
    mv RECORD;
BEGIN
    -- Loop through all materialized views in the specified schema
    FOR mv IN
        SELECT matviewname, schemaname
        FROM pg_matviews
        WHERE schemaname = 'public' -- <-- IMPORTANT: Change 'public' to your target schema if needed
    LOOP
        -- Output the name of the view being dropped
        RAISE NOTICE 'Dropping materialized view: %.%', quote_ident(mv.schemaname), quote_ident(mv.matviewname);

        -- Construct and execute the DROP command
        EXECUTE format('DROP MATERIALIZED VIEW %I.%I CASCADE;', mv.schemaname, mv.matviewname);
    END LOOP;
END;
$$ LANGUAGE plpgsql;
"""
