-- file_convert_jobs

CREATE TABLE file_convert_jobs (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    type VARCHAR(32) NOT NULL,
    param TEXT,
    status VARCHAR(32) NOT NULL,
    failed_reason TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);
