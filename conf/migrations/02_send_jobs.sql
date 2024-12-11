-- email_send_jobs

CREATE TABLE email_send_jobs (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    batch_count INTEGER NOT NULL,
    email_address_count INTEGER NOT NULL,
    type VARCHAR(32) NOT NULL,
    param TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

-- email_send_job_batches

CREATE TABLE email_send_job_batches (
    job_id VARCHAR(255) NOT NULL,
    batch_id INTEGER NOT NULL,
    status VARCHAR(32) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    PRIMARY KEY(job_id, batch_id)
);

-- email_send_job_batch_details

CREATE TABLE email_send_job_batch_details (
    job_id VARCHAR(255) NOT NULL,
    batch_id INTEGER NOT NULL,
    email_address VARCHAR(255) NOT NULL,
    retry_count INTEGER NOT NULL,
    status VARCHAR(32) NOT NULL,
    message_id VARCHAR(255),
    failed_reason TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    PRIMARY KEY(job_id, email_address)
);
CREATE INDEX email_send_job_batch_details_job_id_batch_id_index ON email_send_job_batch_details(job_id, batch_id);
CREATE UNIQUE INDEX email_send_job_batch_details_message_id_unique_index ON email_send_job_batch_details(message_id);

-- email_send_blocked_addresses

CREATE TABLE email_send_blocked_addresses (
    email_address VARCHAR(255) NOT NULL,
    reason TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    PRIMARY KEY(email_address, created_at)
);

-- email_send_logs

CREATE TABLE email_send_logs (
    message_id VARCHAR(255),
    email_address VARCHAR(255) NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    event_detail TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    PRIMARY KEY(message_id, created_at)
);
