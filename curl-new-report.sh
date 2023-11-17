curl -X PUT \
      -H "Content-type: application/json" \
      -H "x-api-key: secret" \
      -d "@new-report.json" \
      localhost:8080/report/