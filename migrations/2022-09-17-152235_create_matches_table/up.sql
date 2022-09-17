CREATE TABLE matches (
    id SERIAL NOT NULL,
    job_id INT NOT NULL,
    left_submission_id INT NOT NULL,
    left_match_rate INT NOT NULL,
    right_submission_id INT NOT NULL,
    right_match_rate INT NOT NULL,
    lines_matched INT NOT NULL,
    PRIMARY KEY (id)
);