# llvm cov host

> **This project is under Development ⚠️**

Self hosted coverage host.

Using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), and [llvm-cov-pretty](https://github.com/dnaka91/llvm-cov-pretty), we gather and host the generated HTML.
It also makes a comparison against previously sent report and sent it back to use it in the CI.

**Table of Content**

- [llvm cov host](#llvm-cov-host)
- [Usage](#usage)
  - [Start the server](#start-the-server)
    - [From source](#from-source)
    - [with docker](#with-docker)
  - [Send a coverage report](#send-a-coverage-report)
    - [Request payload for PUT /report](#request-payload-for-put-report)
  - [View reports](#view-reports)
- [Usage in Github Workflow](#usage-in-github-workflow)
- [Persistance](#persistance)
- [Features](#features)
- [Contribute](#contribute)

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
      localhost:8080/report
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

## View reports

Reports are accessible on the `/view/{name}/index.html` route.\
From the example above to see the report we uploaded go to : http://localhost:8080/view/test/index.html

> NOTE ⚠️: The access to reports is not secured by any authentication, thus making the source code accessible publicly


# Usage in Github Workflow

The workflow [coverage](.github/workflows/coverage.yml) is an example on how to send reports to the server

# Persistance

All Json reports received, and HTML export will be stored in the `output` directory \
It also contain repository that where cloned, there is no cleanup or check on the size of this directory for now.

# Features

- [x] Generating the HTML report
- [x] Cloning the repository to have the sources in the report
- [x] Serving HTML reports
- [x] Github Action example
- [x] Works with cargo namespaces
- [x] Compare with previous reports
- [x] Keep coverage % history
- [x] Dashboard see progression and stats
- [ ] Dashboard group project & graphs 
- [ ] Authentication
- [ ] Permissions

# Contribute

Any contributions are welcomed !