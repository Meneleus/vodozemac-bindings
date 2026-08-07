// Generates test-vectors/megolm-v1.json — static cross-platform vectors.
//
// The vectors are committed to the repo. CI verifies them against the wasm
// build (scripts/verify-test-vectors.mjs); the RN on-device self-test
// verifies them against the native bridge. Both passing proves the two
// targets interoperate on the wire.
//
// Run only when intentionally regenerating: node scripts/generate-test-vectors.mjs
import { createRequire } from 'node:module';
import { writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const require = createRequire(import.meta.url);
const v = require('../pkg-node/kinsh_vodozemac_wasm.js');

const outbound = new v.GroupSession();
const sessionKey = outbound.sessionKey(); // at index 0
const sessionId = outbound.sessionId();

const plaintexts = [
  'megolm vector 0: plain ascii',
  'megolm vector 1: unicode — çírçlé 🎉',
  'megolm vector 2: {"json":"payload","n":42}',
  'megolm vector 3: after export point',
];
const messages = plaintexts.map((plaintext, index) => ({
  index,
  plaintext,
  ciphertext: outbound.encrypt(plaintext),
}));

const inbound = new v.InboundGroupSession(sessionKey);
const exportedAtIndex2 = inbound.exportAt(2);

const vectors = {
  description:
    'Megolm v1 cross-platform test vectors. An InboundGroupSession built ' +
    'from sessionKey must decrypt every message; one imported from ' +
    'exportedAtIndex2 must decrypt only index >= 2.',
  megolmVersion: 1,
  sessionId,
  sessionKey,
  exportedAtIndex2,
  inboundPickle: inbound.pickle(),
  messages,
};

const dir = join(dirname(fileURLToPath(import.meta.url)), '..', 'test-vectors');
mkdirSync(dir, { recursive: true });
const file = join(dir, 'megolm-v1.json');
writeFileSync(file, JSON.stringify(vectors, null, 2) + '\n');
console.log(`wrote ${file}`);
