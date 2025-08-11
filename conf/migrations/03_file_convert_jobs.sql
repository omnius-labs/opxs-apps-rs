-- file_convert_jobs

CREATE TABLE file_convert_jobs (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    type VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    param TEXT,
    in_file_name VARCHAR(255) NOT NULL,
    out_file_name VARCHAR(255) NOT NULL,
    failed_reason TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);
CREATE INDEX file_convert_jobs_id_user_id_index ON file_convert_jobs(id, user_id);
