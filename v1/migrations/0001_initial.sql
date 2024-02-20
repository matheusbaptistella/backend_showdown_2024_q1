-- A table for storing/updating all client's data.
-- A single entry per client.
CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    total_limit INTEGER NOT NULL,
    balance INTEGER NOT NULL CONSTRAINT balance_exceeded CHECK (balance > -total_limit),
);

-- A table for storing the transactions executed by all clients.
-- Multiple entries for each client.
CREATE TABLE transactions (
    client_id INTEGER REFERENCES clients(id), -- Ver se da pra usar serial aqui ou se incrementaria
    txn_value INTEGER NOT NULL,
    txn_type CHARACTER(1) NOT NULL,
    txn_description VARCHAR(10),
    executed_at TIMESTAMP,
);

INSERT INTO clients (id, total_limit, balance) 
VALUES 
    (1, 1000 * 100, 0),
    (2, 800 * 100, 0),
    (3, 10000 * 100, 0),
    (4, 100000 * 100, 0),
    (5, 5000 * 100, 0);