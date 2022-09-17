CREATE TABLE submissions (
    id SERIAL NOT NULL,
    job_id INT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    PRIMARY KEY (id)
);