-- users

CREATE TYPE user_authentication_type AS ENUM ('Email', 'Provider');
CREATE TYPE user_role AS ENUM ('Admin', 'User');

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    authentication_type user_authentication_type NOT NULL,
    role user_role NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER update_users_updated_at_step1 BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_updated_at_step2 BEFORE UPDATE OF updated_at ON users FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_updated_at_step3 BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

-- users_auth_email

CREATE TABLE users_auth_email (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    password_hash VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_users_auth_email_updated_at_step1 BEFORE UPDATE ON users_auth_email FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_auth_email_updated_at_step2 BEFORE UPDATE OF updated_at ON users_auth_email FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_auth_email_updated_at_step3 BEFORE UPDATE ON users_auth_email FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

-- users_auth_provider

CREATE TABLE users_auth_provider (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    provider_type VARCHAR(255) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX users_auth_provider_provider_type_provider_user_id_unique_index ON users_auth_provider(provider_type, provider_user_id);
CREATE TRIGGER update_users_auth_provider_updated_at_step1 BEFORE UPDATE ON users_auth_provider FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_auth_provider_updated_at_step2 BEFORE UPDATE OF updated_at ON users_auth_provider FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_auth_provider_updated_at_step3 BEFORE UPDATE ON users_auth_provider FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

-- users_tokens

CREATE TABLE users_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_users_tokens_updated_at_step1 BEFORE UPDATE ON users_tokens FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_tokens_updated_at_step2 BEFORE UPDATE OF updated_at ON users_tokens FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_tokens_updated_at_step3 BEFORE UPDATE ON users_tokens FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

-- email_send_jobs

CREATE TYPE email_send_job_type AS ENUM ('Unknown', 'EmailConfirm');
CREATE TYPE email_send_job_status AS ENUM ('Unknown', 'Pending', 'Processing', 'Completed', 'Failed');

CREATE TABLE email_send_jobs (
    id BIGSERIAL PRIMARY KEY,
    type email_send_job_type NOT NULL,
    status email_send_job_status NOT NULL,
    param TEXT,
    failed_reason TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER email_send_jobs_updated_at_step1 BEFORE UPDATE ON email_send_jobs FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER email_send_jobs_updated_at_step2 BEFORE UPDATE OF updated_at ON email_send_jobs FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER email_send_jobs_updated_at_step3 BEFORE UPDATE ON email_send_jobs FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

-- email_send_addresses

CREATE TABLE email_send_addresses (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER email_send_addresses_updated_at_step1 BEFORE UPDATE ON email_send_addresses FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER email_send_addresses_updated_at_step2 BEFORE UPDATE OF updated_at ON email_send_addresses FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER email_send_addresses_updated_at_step3 BEFORE UPDATE ON email_send_addresses FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

-- email_send_logs

CREATE TABLE email_send_logs (
    id BIGSERIAL PRIMARY KEY,
    email_send_address_id INTEGER NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    event_detail TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (email_send_address_id) REFERENCES email_send_addresses(id) ON DELETE CASCADE
);
COMMENT ON COLUMN email_send_logs.event_type IS 'bounce or spam';
CREATE TRIGGER email_send_logs_updated_at_step1 BEFORE UPDATE ON email_send_logs FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER email_send_logs_updated_at_step2 BEFORE UPDATE OF updated_at ON email_send_logs FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER email_send_logs_updated_at_step3 BEFORE UPDATE ON email_send_logs FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();
