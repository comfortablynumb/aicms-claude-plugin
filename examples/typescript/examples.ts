// AICMS TypeScript Examples
// Demonstrates @ai:* annotations in TypeScript code

/**
 * @ai:intent Calculate compound interest for a loan or investment
 * @ai:pre principal > 0 && rate >= 0 && years >= 0
 * @ai:post result >= principal
 * @ai:example (1000, 0.05, 10) -> 1628.89
 * @ai:example (1000, 0.0, 5) -> 1000
 * @ai:example (500, 0.1, 1) -> 550
 * @ai:effects pure
 * @ai:complexity O(1)
 * @ai:confidence 0.95
 */
export function calculateCompoundInterest(
    principal: number,
    rate: number,
    years: number
): number {
    return principal * Math.pow(1 + rate, years);
}

/**
 * @ai:intent Find the maximum value in a non-empty array
 * @ai:pre values.length > 0
 * @ai:post result is contained in values
 * @ai:post result >= all elements in values
 * @ai:example ([1, 5, 3, 9, 2]) -> 9
 * @ai:example ([42]) -> 42
 * @ai:example ([-5, -2, -8]) -> -2
 * @ai:effects pure
 * @ai:complexity O(n)
 * @ai:edge_cases single element -> returns that element
 * @ai:edge_cases all equal -> returns that value
 */
export function findMax(values: number[]): number {
    return Math.max(...values);
}

interface HttpResponse {
    status: number;
    body: string;
}

/**
 * @ai:intent Send an HTTP POST request with JSON body and return the response
 * @ai:pre url is a valid HTTP/HTTPS URL
 * @ai:pre body is a valid object
 * @ai:post result contains status code and response body
 * @ai:effects network, io
 * @ai:idempotent false
 * @ai:needs_review Verify timeout handling and error cases
 * @ai:assumes Network connectivity is available
 * @ai:confidence 0.75
 */
export async function postJson(
    url: string,
    body: Record<string, unknown>
): Promise<HttpResponse> {
    const response = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
    });

    return {
        status: response.status,
        body: await response.text(),
    };
}

/**
 * @ai:intent Validate an email address format using basic rules
 * @ai:pre email !== null && email !== undefined
 * @ai:post result === true implies email contains exactly one '@' with text on both sides
 * @ai:example ("user@example.com") -> true
 * @ai:example ("invalid-email") -> false
 * @ai:example ("@example.com") -> false
 * @ai:example ("user@") -> false
 * @ai:example ("") -> false
 * @ai:effects pure
 * @ai:complexity O(n)
 * @ai:confidence 0.85
 * @ai:needs_review Does not validate against RFC 5322, only basic format
 */
export function isValidEmail(email: string): boolean {
    if (!email) {
        return false;
    }

    const parts = email.split('@');

    if (parts.length !== 2) {
        return false;
    }

    const [local, domain] = parts;

    return local.length > 0 && domain.length > 0 && domain.includes('.');
}
