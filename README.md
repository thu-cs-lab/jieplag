# JiePlag

JiePlag is a plagiarism checker for checking code similarity. It currently supports the following languages:

1. C/C++
2. Rust
3. Python
4. Verilog

It supports standalone mode (see `Local binaries` section) and client-server mode (see `Run server` section).

## Local binaries

* `core/src/bin/find_pairs.rs`: Find pairs of files that contain possible plagiarism
* `core/src/bin/compute_matches.rs`: Compute matched text blocks from two source files (and optional teamplte file)

Example for `find_pairs`:

```shell
$ RUST_LOG=info cargo run --bin find_pairs -- --source-directory examples/aplusb/students --template-directory examples/aplusb/template --include cpp
Possible plagarism: examples/aplusb/students/student1 and examples/aplusb/students/student3: 3 matches
```

Example for `compute_matches`:

```shell
$ cargo run --bin compute_matches -- --left examples/aplusb/students/student1/main.cpp --right examples/aplusb/students/student3/main.cpp
Match #1:
L0-L14:
#include <stdio.h>

int aplusb(int a, int b) {
  // implement aplusb
  return a+b;
}

int main() {
  // implement input/output
  int a, b;
  scanf("%d%d", &a, &b);
  int c = aplusb(a, b);
  printf("%d\n", c);
  return 0;
}
Match #1:
L0-L14:
#include <stdio.h>

int aplusb(int a, int b)
{
  return a + b;
}

int main()
{
  int a, b;
  scanf("%d %d", &a, &b);
  int c = aplusb(a, b);
  printf("%d\n", c);
  return 0;
}
$ cargo run --bin compute_matches -- --left examples/aplusb/students/student1/main.cpp --right examples/aplusb/students/student4/main.cpp
Match #1:
L4-L14:
  return a+b;
}

int main() {
  // implement input/output
  int a, b;
  scanf("%d%d", &a, &b);
  int c = aplusb(a, b);
  printf("%d\n", c);
  return 0;
}
Match #1:
L4-L14:
  return a-b+b+b;
}

int main() {
  // implement input/output
  int a, b;
  scanf("%d%d", &a, &b);
  int c = aplusb(a, b);
  printf("%d\n", c);
  return 0;
}
```

## Run server

Configuration: `server/.env.sample`.

* `server/src/bin/create_user.rs`: Create users in database
* `server/src/bin/server.rs`: Run web server to accept requests
* `client/srv/bin/cli.rs`: CLI tool to submit to server

After submission via CLI, a link will be generated to view in browser. An example webpage is provided at `examples/aplusb/html`, you can view it via:

```shell
cd examples/aplusb/html
python3 -m http.server
# in another shell
open http://127.0.0.1:8000/
```

To setup jieplag server, the following steps are required:

1. Setup postgresql database
2. Copy `server/.env.sample` to `.env` and change the contents accordingly
3. Build server and run it
4. Use `create_user` to create new user
5. Use `cli` to submit code to jieplag

Setup postgres database in postgres shell:

```sql
create database jieplag;
create user jieplag with encrypted password 'REDACTED';
grant all privileges on database jieplag to jieplag;
\c jieplag postgres
grant all on schema public to jieplag;
```

Copy `.env.sample` to `.env` and edit:

```env
DATABASE_URL=postgres://DB_USER_HERE:DB_PASSWORD_HERE@DB_HOSTNAME_HERE/jieplag
COOKIE_SECRET=PUT_SOME_RANDOM_LONG_SECRET_HERE
PUBLIC_URL=http://PUBLIC_HOSTNAME_HERE/SUBPATH_IS_SUPPORTED
```

Run server:

```shell
RUST_LOG=info cargo run --bin server
```

Please ensure you are running server under the same (or descendant) directory where `.env` is located, due to how `dotenv` works.

Create user:

```shell
cargo run --bin create_user -- --user-name USER_NAME_HERE --password PASSWORD_NAME_HERE [--force]
```

Submit code:

```shell
cargo run --bin cli -- --language cc --user-name USER_NAME_HERE --password PASSWORD_HERE [--template PATH_TO_TEMPLATE_CODE] PATH_TO_STUDENT1_CODE PATH_TO_STUDENT2_CODE ...
```

## Acknowledgements

JiePlag is highly influenced by Stanford MOSS. Due to frequent outage of Stanford MOSS, we created JiePlag as a open source software clone. We re-implemented [winnow](https://theory.stanford.edu/~aiken/publications/papers/sigmod03.pdf) algorithm and mimicked the web interface of Stanford MOSS.

We highly thanked Stanford MOSS for their great contributions to the teaching comunity.
