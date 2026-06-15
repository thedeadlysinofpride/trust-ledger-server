#!/bin/bash

# TrustLedger - Member API Test Script
# Run this script to test all member endpoints

echo "=========================================="
echo "TrustLedger Member API Test Suite"
echo "=========================================="
echo ""

# Configuration
BASE_URL="http://localhost:5150"
PHONE="0532423078"

echo "Testing Member Authentication"
echo "----------------------------------------"

# Step 1: Request OTP
echo "1. Requesting OTP..."
OTP_RESPONSE=$(curl -s -X POST ${BASE_URL}/api/member/auth/send-otp \
  -H "Content-Type: application/json" \
  -d "{\"phone\": \"${PHONE}\"}")
echo "$OTP_RESPONSE" | jq
echo ""

# IMPORTANT: Check server logs for the OTP code
echo "CHECK SERVER LOGS FOR THE OTP CODE"
echo "Look for: OTP sent to 233532423078: XXXXXX"
echo ""
read -p "Enter the OTP code from server logs: " OTP_CODE

# Step 2: Verify OTP and get token
echo ""
echo "2. Verifying OTP and getting token..."
AUTH_RESPONSE=$(curl -s -X POST ${BASE_URL}/api/member/auth/verify \
  -H "Content-Type: application/json" \
  -d "{\"phone\": \"${PHONE}\", \"code\": \"${OTP_CODE}\"}")
TOKEN=$(echo "$AUTH_RESPONSE" | jq -r '.token')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "Failed to get token. Response:"
  echo "$AUTH_RESPONSE" | jq
  exit 1
fi

echo "Token obtained successfully"
echo ""

# Save token for use in other commands
export TOKEN

echo "Testing Member Dashboard"
echo "----------------------------------------"

echo "3. Getting member dashboard..."
curl -s -X GET ${BASE_URL}/api/member/dashboard \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "Testing Groups"
echo "----------------------------------------"

echo "4. Listing all available groups..."
curl -s -X GET ${BASE_URL}/api/groups | jq
echo ""

echo "5. Joining a group (code: MKL)..."
curl -s -X POST ${BASE_URL}/api/member/groups/join \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"code": "MKL"}' | jq
echo ""

echo "6. Listing member's groups..."
curl -s -X GET ${BASE_URL}/api/member/groups \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "7. Getting group details (ID: 1)..."
curl -s -X GET ${BASE_URL}/api/member/groups/1 \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "8. Refreshing dashboard after joining group..."
curl -s -X GET ${BASE_URL}/api/member/dashboard \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "Testing Savings Goals"
echo "----------------------------------------"

echo "9. Creating a savings goal..."
curl -s -X POST ${BASE_URL}/api/member/savings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Phone",
    "target_amount": 2000,
    "duration_days": 90,
    "frequency": "weekly",
    "mode": "loose"
  }' | jq
echo ""

echo "10. Listing savings goals..."
curl -s -X GET ${BASE_URL}/api/member/savings \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "11. Depositing to savings goal..."
curl -s -X POST ${BASE_URL}/api/member/savings/deposit \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "goal_id": 1,
    "amount": 100
  }' | jq
echo ""

echo "12. Withdrawing from savings goal..."
curl -s -X POST ${BASE_URL}/api/member/savings/withdraw/1 \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "Testing Trust Score"
echo "----------------------------------------"

echo "13. Getting trust score history..."
curl -s -X GET ${BASE_URL}/api/member/trust-score/history \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "Testing Transactions"
echo "----------------------------------------"

echo "14. Getting transactions summary..."
curl -s -X GET ${BASE_URL}/api/member/transactions/summary \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "15. Getting paginated transactions list..."
curl -s -X GET "${BASE_URL}/api/member/transactions?page=1&per_page=10" \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "16. Filtering transactions by type (contributions)..."
curl -s -X GET "${BASE_URL}/api/member/transactions?type=contribution" \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "17. Filtering transactions by status (completed)..."
curl -s -X GET "${BASE_URL}/api/member/transactions?status=completed" \
  -H "Authorization: Bearer $TOKEN" | jq
echo ""

echo "=========================================="
echo "Test Suite Complete"
echo "=========================================="
