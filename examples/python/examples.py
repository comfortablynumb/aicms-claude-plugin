# AICMS Python Examples
# Demonstrates @ai:* annotations in Python code

from typing import List, Optional
import json
import httpx


# @ai:intent Calculate compound interest for a loan or investment
# @ai:pre principal > 0.0 and rate >= 0.0 and years >= 0
# @ai:post result >= principal
# @ai:example (1000.0, 0.05, 10) -> 1628.89
# @ai:example (1000.0, 0.0, 5) -> 1000.0
# @ai:example (500.0, 0.1, 1) -> 550.0
# @ai:effects pure
# @ai:complexity O(1)
# @ai:confidence 0.95
def calculate_compound_interest(principal: float, rate: float, years: int) -> float:
    return principal * (1 + rate) ** years


# @ai:intent Find the maximum value in a non-empty list
# @ai:pre len(values) > 0
# @ai:post result in values
# @ai:post result >= max(values)
# @ai:example ([1, 5, 3, 9, 2]) -> 9
# @ai:example ([42]) -> 42
# @ai:example ([-5, -2, -8]) -> -2
# @ai:effects pure
# @ai:complexity O(n)
# @ai:edge_cases single element -> returns that element
# @ai:edge_cases all equal -> returns that value
def find_max(values: List[int]) -> int:
    return max(values)


# @ai:intent Send an HTTP POST request with JSON body and return the response
# @ai:pre url is a valid HTTP/HTTPS URL
# @ai:pre body is a JSON-serializable dict
# @ai:post result contains status_code and response body
# @ai:effects network, io
# @ai:idempotent false
# @ai:needs_review Verify timeout handling and error cases
# @ai:assumes Network connectivity is available
# @ai:confidence 0.75
async def post_json(
    client: httpx.AsyncClient,
    url: str,
    body: dict
) -> dict:
    response = await client.post(url, json=body)

    return {
        "status": response.status_code,
        "body": response.text
    }


# @ai:intent Validate an email address format using basic rules
# @ai:pre email is not None
# @ai:post result == True implies '@' in email with text on both sides
# @ai:example ("user@example.com") -> True
# @ai:example ("invalid-email") -> False
# @ai:example ("@example.com") -> False
# @ai:example ("user@") -> False
# @ai:example ("") -> False
# @ai:effects pure
# @ai:complexity O(n)
# @ai:confidence 0.85
# @ai:needs_review Does not validate against RFC 5322, only basic format
def is_valid_email(email: str) -> bool:
    if not email:
        return False

    parts = email.split('@')

    if len(parts) != 2:
        return False

    local, domain = parts

    return bool(local) and bool(domain) and '.' in domain
