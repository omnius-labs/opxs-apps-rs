--- users

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    authentication_type VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
COMMENT ON COLUMN users.authentication_type IS 'email or provider';
CREATE TRIGGER update_users_updated_at_step1 BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_updated_at_step2 BEFORE UPDATE OF updated_at ON users FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_updated_at_step3 BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

--- users_auth_email

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

--- users_auth_provider

CREATE TABLE users_auth_provider (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    provider_type VARCHAR(255) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX users_auth_provider_provider_type_provider_user_id_index ON users_auth_provider(provider_type, provider_user_id);
CREATE TRIGGER update_users_auth_provider_updated_at_step1 BEFORE UPDATE ON users_auth_provider FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_auth_provider_updated_at_step2 BEFORE UPDATE OF updated_at ON users_auth_provider FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_auth_provider_updated_at_step3 BEFORE UPDATE ON users_auth_provider FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();

--- users_tokens

CREATE TABLE users_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TRIGGER update_users_tokens_updated_at_step1 BEFORE UPDATE ON users_tokens FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_users_tokens_updated_at_step2 BEFORE UPDATE OF updated_at ON users_tokens FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_users_tokens_updated_at_step3 BEFORE UPDATE ON users_tokens FOR EACH ROW EXECUTE PROCEDURE refresh_updated_at_current();
