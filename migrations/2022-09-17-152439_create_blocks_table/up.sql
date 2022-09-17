CREATE TABLE blocks (
    id SERIAL NOT NULL,
    match_id INT NOT NULL,
    left_line_from INT NOT NULL,
    right_line_from INT NOT NULL,
    length INT NOT NULL,
    PRIMARY KEY (id)
);