CREATE TABLE tickets (
  id SERIAL PRIMARY KEY,
  description TEXT NOT NULL
);

CREATE TABLE tickets_to_user (
  id SERIAL PRIMARY KEY,
    ticket_id INT references tickets(id),
    user_id INT references users(id)
);

CREATE INDEX tickets_by_user ON tickets_to_user(user_id);
