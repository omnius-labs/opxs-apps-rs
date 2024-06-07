-- image_convert_jobs

CREATE TABLE image_convert_jobs (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    param TEXT,
    status VARCHAR(32) NOT NULL,
    failed_reason TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);
