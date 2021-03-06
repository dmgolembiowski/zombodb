CREATE TABLE index_creation_test (
    id int,
    title text
);

INSERT INTO index_creation_test(id, title) SELECT g, md5(g::text) FROM generate_series(1, 10) g;

--
-- create an index that directly references the entire row from the table we're indexing
--
CREATE INDEX idxtest ON index_creation_test USING zombodb ( (index_creation_test.*) );
DROP INDEX idxtest;


--
-- create an index that indexes the result of a function that returns a composite type
--
CREATE OR REPLACE FUNCTION gigo(anyelement) RETURNS anyelement IMMUTABLE STRICT LANGUAGE sql AS $$
    SELECT $1;
$$;
CREATE INDEX idxtest ON index_creation_test USING zombodb (gigo(index_creation_test));
DROP INDEX idxtest;
DROP FUNCTION gigo(anyelement);

DROP TABLE index_creation_test CASCADE;
