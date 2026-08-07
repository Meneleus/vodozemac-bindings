// Verifies test-vectors/megolm-v1.json against the pkg-node wasm build.
// Run: node scripts/verify-test-vectors.mjs
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const require = createRequire(import.meta.url);
const v = require('../pkg-node/kinsh_vodozemac_wasm.js');

const file = join(
  dirname(fileURLToPath(import.meta.url)),
  '..',
  'test-vectors',
  'megolm-v1.json',
);
const vectors = JSON.parse(readFileSync(file, 'utf8'));

// 1. Fresh inbound session from the session key decrypts everything.
const inbound = new v.InboundGroupSession(vectors.sessionKey);
assert.equal(inbound.sessionId(), vectors.sessionId);
assert.equal(inbound.firstKnownIndex(), 0);
for (const m of vectors.messages) {
  const out = JSON.parse(inbound.decrypt(m.ciphertext));
  assert.equal(out.plaintext, m.plaintext);
  assert.equal(out.messageIndex, m.index);
}

// 2. Session imported from the exported key reads only index >= 2.
const imported = v.InboundGroupSession.import(vectors.exportedAtIndex2);
assert.equal(imported.sessionId(), vectors.sessionId);
assert.equal(imported.firstKnownIndex(), 2);
for (const m of vectors.messages) {
  if (m.index >= 2) {
    assert.equal(JSON.parse(imported.decrypt(m.ciphertext)).plaintext, m.plaintext);
  } else {
    assert.throws(() => imported.decrypt(m.ciphertext));
  }
}

// 3. The committed pickle restores and still decrypts.
const restored = v.InboundGroupSession.fromPickle(vectors.inboundPickle);
assert.equal(restored.sessionId(), vectors.sessionId);
for (const m of vectors.messages) {
  assert.equal(JSON.parse(restored.decrypt(m.ciphertext)).plaintext, m.plaintext);
}

console.log(`test vectors verified (${vectors.messages.length} messages)`);
