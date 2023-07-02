-- functions

CREATE FUNCTION refresh_updated_at_none() RETURNS trigger AS
$$
BEGIN
    IF NEW.updated_at = OLD.updated_at THEN
        NEW.updated_at := NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION refresh_updated_at_same() RETURNS trigger AS
$$
BEGIN
    IF NEW.updated_at IS NULL THEN
        NEW.updated_at := OLD.updated_at;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION refresh_updated_at_current() RETURNS trigger AS
$$
BEGIN
    IF NEW.updated_at IS NULL THEN
        NEW.updated_at := CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

--- _world

CREATE TABLE _world (
    key VARCHAR(255) NOT NULL PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_world_updated_at_step1
    BEFORE UPDATE ON _world FOR EACH ROW
    EXECUTE PROCEDURE refresh_updated_at_none();
CREATE TRIGGER update_world_updated_at_step2
    BEFORE UPDATE OF updated_at ON _world FOR EACH ROW
    EXECUTE PROCEDURE refresh_updated_at_same();
CREATE TRIGGER update_world_updated_at_step3
    BEFORE UPDATE ON _world FOR EACH ROW
    EXECUTE PROCEDURE refresh_updated_at_current();
