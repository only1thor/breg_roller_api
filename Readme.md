# Bedrift roller
prosjekt for å finne personers rolle i en bedrift. 

## Rest api in rust
[resource](https://blog.logrocket.com/building-rest-api-rust-warp/) used for simple rest api. 
in case it's taken down later heres the content: 

# Building a REST API in Rust with warp
[Bastian Gruber](https://blog.logrocket.com/author/bastiangruber/)

Editor’s note: This guide to building a REST API in Rust with warp was last updated on 8 February 2023 to reflect recent changes to the Rust framework. This update also includes new sections on the benefits of warp in Rust. 

Rust is a lot of folks’ favorite programming language, but it can still be hard to find a project for it or even to get a firm grasp of it. A good way to start with any language is to build something you will use daily. If your company operates microservices, it’s even easier. Rust is well-suited to replace such a service, and you could rewrite it in a matter of days.

When starting with Rust, you’ll need to [learn the fundamentals](https://blog.logrocket.com/getting-up-to-speed-with-rust/). Once familiar with the syntax and basic concepts, you can start thinking about [asynchronous Rust](https://blog.logrocket.com/a-practical-guide-to-async-in-rust/). Most modern languages have a built-in runtime that handles async tasks, such as sending off a request or waiting in the background for an answer.

In Rust, you have to choose an async runtime that works for you. Libraries usually have their own runtime. If you work on a larger project, you may want to avoid adding multiple runtimes because by choosing a single, consistent runtime, you can simplify your application architecture, and reduce the application’s complexity, and the risk of compatibility issues.

[Tokio](https://tokio.rs/) is the most production-used and proven runtime that can handle asynchronous tasks, so chances are high that your future employer already uses it. Your choices are, therefore, somewhat limited since you may need to choose a library that already has Tokio built in to create your API. For this tutorial, we’ll use [warp](https://github.com/seanmonstar/warp). Depending on your previous programming experience, it may take a few days to wrap your head around it. But once you understand warp, it can be quite an elegant tool for building APIs.

Let’s get started:

## What is warp?

Warp is a minimal and efficient web framework for building HTTP-based web services in Rust. It provides a high-level API for building HTTP servers, focusing on security, performance, and stability. Warp also includes built-in features such as support for HTTP/1 and HTTP/2, TLS encryption, asynchronous programming, and common middleware for logging, rate limiting, and routing tasks. A lot of its features are borrowed from [Hyper](https://hyper.rs/), as warp is more like a superset of Hyper.

## Setting up your project

You’ll need to install the following libraries to follow along with this tutorial.

    warp for creating the API
    Tokio to run an asynchronous server
    Serde to help serialize incoming JSON
    parking_lot to create a `ReadWriteLock` for your local storage

First, create a new project with cargo:

```bash
cargo new neat-api --bin
```

We’ve included warp in our `Cargo.toml` so we can use it throughout our codebase:

```toml
[dependencies]
warp = "0.2"
parking_lot = "0.10.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "0.2", features = ["macros"] }
```

For the first test, create a simple “Hello, World!” in `main.rs`, as shown below:

```rust
use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

`Filters` are a way to parse a request and match against a route we created. So when you start the server via `cargo run` and point your browser to `localhost:3030/hello/WHATEVER`, warp sends this request through its filters and executes the first one that is triggered.

In `let hello = …` we created a new path, essentially saying that every request with the path `/hello` plus a string gets handled by this method. So, we return `Hello, WHATEVER`.
If we point the browser to `localhost:3030/hello/new/WHATEVER`, we’ll get a 404 since we don’t have a filter for `/hello/new + String`.

## Building the REST API

Let’s build a real API to demonstrate these concepts. A good model is an API for a grocery list. We want to be able to add items to the list, update the quantity, delete items, and view the whole list. Therefore, we need four different routes with the `HTTP methods` `GET`, `DELETE`, `PUT`, and `POST`.
With so many different routes, is it wise to create methods for each instead of handling them all in main.rs?

## Creating local storage

In addition to routes, we need to store a state in a file or local variable. In an [async environment](https://blog.logrocket.com/pinning-rust-async-data-types-memory-safety/), we have to make sure only one method can access the store at a time so there are no inconsistencies between threads.

In Rust, we have `Arc`, so the compiler knows when to drop a value and a read and write lock (`RwLock`). That way, no two methods on different threads are writing to the same memory.
Your store implementation should look like this:

```rust
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

type Items = HashMap<String, i32>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Item {
    name: String,
    quantity: i32,
}

#[derive(Clone)]
struct Store {
  grocery_list: Arc<RwLock<Items>>
}

impl Store {
    fn new() -> Self {
        Store {
            grocery_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
```

`POST`ing an item to the list

Now, we can add our first route. To add items to the list, make an HTTP `POST` request to a path. Our method has to return a proper HTTP code so the caller knows whether their call was successful. Warp offers basic types via its own `http` library, which we need to include as well. Add it like so:

```rust
use warp::{http, Filter};
```

The method for the `POST` request looks like this:


```rust
async fn add_grocery_list_item(
    item: Item,
    store: Store
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let r = store.grocery_list.read();
        Ok(warp::reply::json(&*r))
}
```

The warp framework offers the option to `reply with status` so we can add text plus a generic HTTP status so the caller knows whether the request was successful or if they have to try again. Now, add a new route and call the method you just created for it. Since you can expect a JSON for this, you should create a little `json_body` helper function to extract the `Item` out of the `body` of the HTTP request.

In addition, we need to pass the store down to each method by cloning it and creating a `warp filter`, which we call in the `.and()` during the `warp path` creation:

```rust
fn json_body() -> impl Filter<Extract = (Item,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let add_items = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(json_body())
        .and(store_filter.clone())
        .and_then(add_grocery_list_item);

    warp::serve(add_items)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

You can test the `POST` call via `curl` or an application such as [Postman](https://blog.logrocket.com/how-automate-api-tests-postman/), which is now a standalone application for making HTTP requests. Start the server via `cargo run` and open another terminal window or tab to execute the following `curl`:

```bash
curl --location --request POST 'localhost:3030/v1/groceries' \
--header 'Content-Type: application/json' \
--header 'Content-Type: text/plain' \
--data-raw '{
        "name": "apple",
        "quantity": 3
}'
```

You should get the text response and HTTP code as defined in your method.

## `GET`ting the grocery list

Now, we can post a list of items to our grocery list, but we still can’t retrieve them. We need to create another route for the `GET` request. Our main function will add this new route. For this new route, we don’t need to parse any JSON. Here’s the code:

```rust
#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let add_items = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(json_body())
        .and(store_filter.clone())
        .and_then(add_grocery_list_item);

    let get_items = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_grocery_list);


    let routes = add_items.or(get_items);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

You’ll get a taste of async Rust when you examine the data structure behind your `Arc`. You’ll need to use the `.read()` method to access and dereference the data. Here is how the function looks:

```rust
async fn get_grocery_list(
    store: Store
    ) -> Result<impl warp::Reply, warp::Rejection> {
         let result = store.grocery_list.read();
        Ok(warp::reply::json(&*result))
}
```

Then, create a variable for the `store.grocery_list.read()`, we’ll call it result. Notice that we are returning `&*result;` that’s new, right? Yes, `&*result `dereferences the [RwLockReadGuard](https://doc.rust-lang.org/std/sync/struct.RwLockReadGuard.html) object `result` to a `&HashMap`, which is then passed as a reference to the `warp::reply::json` function that’s been returned.

## `UPDATE` and `DELETE`

The last two missing methods are `UPDATE` and `DELETE`. For `DELETE`, you can almost copy your add_grocery_list_item, but instead of `.insert()`, [.remove()](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.remove) an entry.

A special case is the `UPDATE`. Here the Rust HashMap implementation uses `.insert()` as well, but it updates the value instead of creating a new entry if the key doesn’t exist. Therefore, just rename the method and call it for the `POST` as well as the `PUT`.

For the `DELETE` method, you need to pass just the name of the item, so create a new struct and add another `parse_json()` method for the new type. Rename the first parsing method and add another one.

You can simply rename your `add_grocery_list_item` method to call it `update_grocery_list` and call it for a `warp::post()` and `warp::put()`. Your complete code should look like this:

```rust
use warp::{http, Filter};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

type Items = HashMap<String, i32>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Id {
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Item {
    name: String,
    quantity: i32,
}

#[derive(Clone)]
struct Store {
  grocery_list: Arc<RwLock<Items>>
}

impl Store {
    fn new() -> Self {
        Store {
            grocery_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

async fn update_grocery_list(
    item: Item,
    store: Store
    ) -> Result<impl warp::Reply, warp::Rejection> {
        store.grocery_list.write().insert(item.name, item.quantity);


        Ok(warp::reply::with_status(
            "Added items to the grocery list",
            http::StatusCode::CREATED,
        ))
}

async fn delete_grocery_list_item(
    id: Id,
    store: Store
    ) -> Result<impl warp::Reply, warp::Rejection> {
        store.grocery_list.write().remove(&id.name);


        Ok(warp::reply::with_status(
            "Removed item from grocery list",
            http::StatusCode::OK,
        ))
}

async fn get_grocery_list(
    store: Store
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let r = store.grocery_list.read();
        Ok(warp::reply::json(&*r))
}

fn delete_json() -> impl Filter<Extract = (Id,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn post_json() -> impl Filter<Extract = (Item,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let add_items = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(post_json())
        .and(store_filter.clone())
        .and_then(update_grocery_list);

    let get_items = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_grocery_list);

    let delete_item = warp::delete()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(delete_json())
        .and(store_filter.clone())
        .and_then(delete_grocery_list_item);


    let update_item = warp::put()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(post_json())
        .and(store_filter.clone())
        .and_then(update_grocery_list);



    let routes = add_items.or(get_items).or(delete_item).or(update_item);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

## Understanding testing curls

After you update the code, restart the server via `cargo run` and use these curls to post, update, get, and delete items.

`
POST
`

```bash
curl --location --request POST 'localhost:3030/v1/groceries' \
--header 'Content-Type: application/json' \
--header 'Content-Type: text/plain' \
--data-raw '{
        "name": "apple",
        "quantity": 3
}'

```

`
UPDATE
`

```bash
curl --location --request PUT 'localhost:3030/v1/groceries' \
--header 'Content-Type: application/json' \
--header 'Content-Type: text/plain' \
--data-raw '{
        "name": "apple",
        "quantity": 5
}'
```

`
GET
`

```bash
curl --location --request GET 'localhost:3030/v1/groceries' \
--header 'Content-Type: application/json' \
--header 'Content-Type: text/plain'
```

`
DELETE
`

```bash
curl --location --request DELETE 'localhost:3030/v1/groceries' \
--header 'Content-Type: application/json' \
--header 'Content-Type: text/plain' \
--data-raw '{
        "name": "apple"
}'
```


To summarize the steps we just covered:

    Create an ID for each item so you can update and delete via /v1/groceries/{id}
    Add a 404 route
    Add error handling for malformatted JSON
    Adjust the return messages for each route
    Add test for each route with curls

## Why use warp in Rust

When it comes to building an API in Rust, you have several library options. However, the specific requirements of your project will help guide your choice. If you decide to go with warp, here are some advantages of using it in your Rust project.

  1.  Performance: warp is designed to be fast and efficient, with a focus on asynchronous processing and performance-optimizing features such as automatic HTTP keep-alive and connection pooling
  2.  Security: warp places a strong emphasis on security, with features such as built-in support for TLS encryption to ensure that your data is transmitted securely over the network
  3.  Simplicity: warp provides a high-level API that is easy to use, yet still powerful and customizable. This makes it simple to get started building HTTP servers, and easy to extend your application with additional functionality as needed
  4.  Robustness: warp is designed to be stable and reliable, with a focus on error handling and reporting
  5.  Scalability: warp is designed to be scalable, with support for HTTP/1 and HTTP/2 and efficient resource utilization, making it a great choice for building high-performance and scalable web applications

## Final thoughts

Warp is an interesting tool for building web APIs with Rust. And even though the code is far from perfect, the sample code gives us the tip of the iceberg for what is possible with warp. We could extend it and I hope you do. I’ll love to see your feedback based on what you’ve built already.



## Nix shell
[kilde](https://gutier.io/post/development-using-rust-with-nix/) for oppsett av shell.nix

fra kilden:

# Setting up a Rust environment in Nix

`
Julio César
@ZzAntares
`

When using Nix, we have different mechanisms at our disposal to setup a development environment for Rust, in this post we’ll explore the most common alternatives out there and the nicest of them all.
## Rust provided by Nixpkgs

The most straight forward way of getting a Rust installation up to speed is by using a shell.nix file with the following expression:

```nix
{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
  ];

  RUST_BACKTRACE = 1;
}
```

With that file in our project folder it is enough to call nix-shell to load an environment with Rust and most essential packages provided to us, this is nice, however if you reference a stable channel the versions of these packages you get could be quite old, of course one can choose to use unstable or just pin a newer commit:

```nix
{ nixpkgs ? import <nixpkgs> { }}:

let
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "1fe6ed37fd9beb92afe90671c0c2a662a03463dd";
    sha256 = "1daa0y3p17shn9gibr321vx8vija6bfsb5zd7h4pxdbbwjkfq8n2";
  };
  pkgs = import pinnedPkgs {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      clippy
      rustc
      cargo
      rustfmt
      rust-analyzer
    ];

    RUST_BACKTRACE = 1;
  }
```

Aside from getting a more recent version of everything (by choosing a recent revision), pinning has the added benefit of making a reference to a fixed package set that will not move under us, of course this is at the cost of being leaved behind as Nixpkgs evolve and introduces newer versions of these packages over time.
## Rust provided by a Oxalica’s Nix Overlay

Either of these approaches we already mentioned is fine if we’re getting started with Nix and just want to get a feeling of what’s like to work with nix-shell environments, however, what if at some point we wish to use a nightly version of Rust? or maybe just a specific version of the Rust toolchain? the answer there is quite simple, Mozilla works on a Nix overlay that gives us access to these, the Nixpkgs manual covers this and it basically consists of referencing Mozilla’s Nixpkgs clone, including the Rust overlay and get our dependencies from there.

One caveat with this approach is that we might end up compiling the rust toolchain in our computer rather than just download pre-built binaries, and that my friends… is no fun, it would be desirable then to download already pre-built binaries for us instead, this is where oxalica’s rust-overlay comes in, which I think is the best way to setup Rust in Nix:

```nix
{ nixpkgs ? import <nixpkgs> { }}:

let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "1fe6ed37fd9beb92afe90671c0c2a662a03463dd";
    sha256 = "1daa0y3p17shn9gibr321vx8vija6bfsb5zd7h4pxdbbwjkfq8n2";
  };
  pkgs = import pinnedPkgs {
    overlays = [ (import rustOverlay) ];
  };
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      rust-bin.stable.latest.default
      rust-analyzer
    ];

    RUST_BACKTRACE = 1;
  }
```

The answer is including Oxalica’s Rust overlay while importing the pinned version of the Nixpkgs, by doing this we get the best of both worlds, a Nix overlay that provides the Rust toolchain over a pinned version of the Nixpkgs.

Take note that cargo, clippy and rustfmt are provided by the Rust toolchain so we don’t need to get those individually they all come in when using rust-bin.stable.latest.default, this means “use the latest stable Rust toolchain” but we can also require a specific version, for example to require 1.48.0 refer to the toolchain like so:
```
rust-bin.stable."1.48.0".default
```
and to use a specific version of Rust nightly use this:
```
rust-bin.nightly."2020-12-31".default
```
or the absolute latest for those that like to stand in the edge 👀:

```
rust-bin.nightly.latest.default
```

All of this is particularly useful in NixOS where it seems rustup doesn’t work properly or at least for me it did not.
