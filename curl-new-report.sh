set -e

cargo llvm-cov --json > new-report.json
sed -i '1s#^#{ "git": "'$(git remote get-url origin)'", "branch": "'$(git branch --show-current)'", "json_report": #' new-report.json
echo '}' >> new-report.json

STATUS_CODE=$(
  curl -o /tmp/request_log.txt -s -w "%{http_code}\n" \
      -X PUT \
      -T "new-report.json" \
      -H "Content-type: application/json" \
      -H "x-api-key: secret" \
      http://localhost:8080/report
      # -H "x-api-key: SPHa8Bx4nSNBKgrD" \
      # https://llvm-cov-host.greefine.fr/report
  )

if [ $STATUS_CODE -eq '200' ]; then
  echo "Successfully send report";
  exit 0;
else
  echo "Error sending report status: $STATUS_CODE, logs:";
  cat /tmp/request_log.txt
  exit 1;
fi