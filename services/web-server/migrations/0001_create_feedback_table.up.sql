CREATE TABLE feedback (
    id SERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    image_name VARCHAR(255) NOT NULL,
    model VARCHAR(100) NOT NULL,
    user_feedback INTEGER NOT NULL CHECK (user_feedback >= 0 AND user_feedback <= 10),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER update_feedback_updated_at BEFORE
UPDATE ON feedback FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();