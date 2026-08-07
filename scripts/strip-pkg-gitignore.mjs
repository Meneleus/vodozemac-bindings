// wasm-pack writes pkg-web/.gitignore and pkg-node/.gitignore containing "*",
// which makes npm pack/publish omit the wasm artifacts even when listed in
// package.json "files". Remove those ignore files before packing.
import { unlinkSync } from "node:fs";
import { join } from "node:path";

for (const dir of ["pkg-web", "pkg-node"]) {
  const path = join(dir, ".gitignore");
  try {
    unlinkSync(path);
    console.log(`removed ${path}`);
  } catch (err) {
    if (err && err.code === "ENOENT") continue;
    throw err;
  }
}
