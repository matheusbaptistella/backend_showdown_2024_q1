# phase1_local
In this module, my goal was (essentially) to make it work. Since it's my first experience writing a backend (especially in Rust!), I wanted to comprehend the web framework, how to write tests, set up a database and so on... Mostly I'll cover some introductory aspects. The most important crates used in this project are:
* Tokio, to provide an asynchronous runtime.
* Axum, as the web framework.
* Sqlx, to deal with the queries to a Postgres database.
* Serde, to handle serialization/deserialization of the JSON body.

## main.rs
Being the entry point of our program, I focused on writing code here that has some relation to our program initialization. Initially, on `main`, we get the database URL from a `.env` file (I decided to do so because anybody can set the user/password/port to run the db as they want), and initialize the db through `init_db` and get as return a connection pool (now this is not relevant but in the future we can use it to reuse connections instead of having to keep opening and close them). Then, we set up our Router through `router` and the address to listen to, and finally, we start our server by executing `axum::serve`.

On the router, there are only 2 possible routes for requests (this is because these were the only routes required by the competition), and for each, we define an associated handler function that gets executed when a request for that path comes in. We add a layer for the connection pool so that it effectively gets injected into the application's routing logic (and our handler functions can access it).

## api.rs
Following the order of execution of our program, when a request for an expected route comes in we call the associated handler function. The purpose of these functions is to deconstruct our request into variables that we can access in our code, for example, we can extract the placeholder in the URL for the id of our client with `Path(id): Path<i32>` and access the JSON body via `Json(core_txn): Json<CoreTransaction>`. In this case, CoreTransaction must be annotated with serde, so that we can deserialize it in the structure of our `CoreTransaction` struct.

After separating the information we want from the request into variables we can use in our code we call the functions that actually do the logic on the database. Here on the api file we just want to deal with logic related to the web service that our program provides, which as mentioned before is to deconstruct the request and then reply with an HTTP response: this response will be built based on the return of the functions that talk to the database. According to the competition's specifications, on success we'll reply with a 200 HTTP Status Code and, in some cases, add a JSON body. In contrast, if our queries to the database fail, we check which kind of error we receive, e.g. if the client accessing our service provided an `id` that doesn't exist in our db, we reply with a 404 HTTP Status Code `Not Found`.

## db.rs
Finally, in the db file, we provide the functions that effectively make the queries to the db and deal with the response back from it. Through Sqlx it's possible to create queries, bind dynamic parameters and fetch the db's response into the format of our struct. This way we successfully separate functionality: here the functions are only responsible for making the queries and returning the `sqlx::error` on failure, or a struct that can be serialized to a JSON on success.

## Tests
Inside `api.rs` and `db.rs` I added an additional module: a test module, so we can verify that our functions are working are behaving correctly (even on errors!) and to ensure that we can test future changes to check if they are still working as expected. These are unit tests, used to test the functionalities individually and to achieve that, we use the following function definition:
```rust
#[sqlx::test(migrations = "./migrations")]
async fn add_transaction_valid_id(pool: PgPool) {
```

This allows Sqlx to, for every test, create a new database (because we are passing the pool as a parameter) and initialize it through the migrations folder (by annotating the function).

## Execution
To run our api and test it we make use of 2 terminal windows: one for the database and another for the server. To run the database we can load a docker container containing a Postgres image. This can be done e.g.:
```
docker run -e POSTGRES_PASSWORD=your_pass -e POSTGRES_USER=your_user -e POSTGRES_DB=the_db_name -p 5433:5432 postgres:16.2-alpine3.19
```

And then, to start the server:
```
cargo run
```

After that, we can use Postman to make the HTTP requests and check the response:

![alt text](../images/postman_result.png)

Next, we will check how to deploy this code.
