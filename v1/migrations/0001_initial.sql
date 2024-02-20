-- A table for storing/updating all client's data.
-- A single entry per client.
CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    total_limit INTEGER NOT NULL,
    balance INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT balance_exceeded CHECK (balance > -total_limit)
);

-- A table for storing the transactions executed by all clients.
-- Multiple entries for each client.
CREATE TABLE transactions (
    client_id INTEGER REFERENCES clients(id), -- Ver se da pra usar serial aqui ou se incrementaria
    txn_value INTEGER NOT NULL,
    txn_type CHARACTER(1) NOT NULL,
    txn_description VARCHAR(10),
    executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO clients (total_limit) 
VALUES 
    (1000 * 100),
    (800 * 100),
    (10000 * 100),
    (100000 * 100),
    (5000 * 100);