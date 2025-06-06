-- 1. Syntax error: missing data type for column 'col'.
CREATE TABLE bad_table (col);

-- 2. Reference to a non‐existent table will also cause an error.
INSERT INTO nonexist VALUES (1);

-- 3. (This won’t be reached because of earlier errors.)
SELECT * FROM bad_table;
