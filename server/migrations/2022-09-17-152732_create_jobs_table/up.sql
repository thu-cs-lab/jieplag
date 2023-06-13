CREATE TABLE jobs (
    id SERIAL NOT NULL,
    creator_user_id INT NOT NULL,
    slug TEXT NOT NULL,
    PRIMARY KEY (id)
);