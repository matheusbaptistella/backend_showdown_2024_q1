-- A table for storing/updating all client's data.
-- A single entry per client.
CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    total_limit INTEGER NOT NULL,
    balance INTEGER NOT NULL DEFAULT 0 CHECK (balance > -total_limit)
);

-- A table for storing the transactions executed by all clients.
-- Multiple entries for each client.
CREATE TABLE transactions (
    client_id INTEGER REFERENCES clients(id), -- Ver se da pra usar serial aqui ou se incrementaria
    txn_value INTEGER NOT NULL,
    txn_type VARCHAR(1) NOT NULL CHECK (txn_type = 'd' OR txn_type = 'c'), -- Com ctz seria mais rapido fazer em codigo
    txn_description VARCHAR(10),
    executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO clients (total_limit) 
VALUES 
    (100000),
    (80000),
    (1000000),
    (10000000),
    (500000);