CREATE TABLE blocks (
    id SERIAL NOT NULL,
    match_id INT NOT NULL,
    left_line_from INT NOT NULL,
    left_line_to INT NOT NULL,
    right_line_from INT NOT NULL,
    right_line_to INT NOT NULL,
    PRIMARY KEY (id)
);