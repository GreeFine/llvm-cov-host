set -e

cargo llvm-cov --json > new-report.json
sed -i '1s#^#{ "name": "test", "git": "'$(git remote get-url origin)'", "branch": "main", "json_report": #' new-report.json
echo '}' >> new-report.json

curl -X PUT \
      -H "Content-type: application/json" \
      -H "x-api-key: secret" \
      -d "@new-report.json" \
      http://localhost:8080/report