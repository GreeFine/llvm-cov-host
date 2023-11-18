# llvm cov host

> This project is under Development ⚠️

Self hosted coverage host.

Using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), and [llvm-cov-pretty](https://github.com/dnaka91/llvm-cov-pretty), we gather and host the generated HTML.
It also makes a comparison against previously sent report and sent it back to use it in the CI.

# Usage

## Start the server

### From source

```
cp .env.example .env
cargo run --release
```

### with docker

> NOTE ⚠️: You probably need to pass your ssh-key to the container.\
> But I am not covering this in this example

```sh
docker build -t llvm-cov-host .
docker run -p 8080:8080 --env-file .env llvm-cov-host
```

## Send a coverage report

The script [curl-new-report.sh](curl-new-report.sh) details the steps used to send a report from scratch.

```sh
# Run the tests, and output the result to a json formatted file new-report.json
cargo llvm-cov --json > new-report.json

# Modify the new-report to the format the API expect, more on this bellow in ()
sed -i '1s#^#{ "name": "test", "git": "'$(git remote get-url origin)'", "branch": "main", "json_report": #' new-report.json
echo '}' >> new-report.json

# Send a PUT request to the API with the file as a parameter and the x-api-key authentication.
curl -X PUT \
      -H "Content-type: application/json" \
      -H "x-api-key: secret" \
      -d "@new-report.json" \
      localhost:8080/report/
```

### Request payload for PUT /report

```rs
struct Request {
    // The name used to differentiate the call for different branches/project
    // Will probably be removed latter
    name: String,
    // The git url, we actually need to clone the repo, so the HTML report can have/display the sources of your project.
    git: String,
    // The branch of the repository you ran the coverage on
    branch: String,
    // The coverage json export of llvm-cov
    json_report: serde_json::Value,
}
```

# Features

- [x] Generating the HTML report
- [x] Cloning the repository to have the sources in the report
- [x] Serving HTML reports
- [] Compare with previous reports
- [] Keep coverage % history
- [] Authentication
- [] Permissions
