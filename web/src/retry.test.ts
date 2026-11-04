import { describe, expect, it } from "vitest";
import { retryOnConflict } from "./retry";

const CONFLICT = new Error("conflict");
const isConflict = (error: unknown) => error === CONFLICT;

describe("rejouer après un conflit", () => {
  it("réussit du premier coup sans relire", async () => {
    let reads = 0;
    const result = await retryOnConflict(() => Promise.resolve("ok"), {
      retries: 3,
      isConflict,
      beforeRetry: () => Promise.resolve(++reads > 0),
    });
    expect(result).toBe("ok");
    expect(reads).toBe(0);
  });

  it("relit puis rejoue après un conflit", async () => {
    let calls = 0;
    let reads = 0;
    const result = await retryOnConflict(
      () => (++calls < 3 ? Promise.reject(CONFLICT) : Promise.resolve("ok")),
      { retries: 3, isConflict, beforeRetry: () => Promise.resolve(++reads > 0) },
    );
    expect(result).toBe("ok");
    expect(calls).toBe(3);
    expect(reads).toBe(2);
  });

  it("abandonne quand l'écriture n'a plus de sens", async () => {
    let calls = 0;
    const result = await retryOnConflict(() => (++calls, Promise.reject(CONFLICT)), {
      retries: 3,
      isConflict,
      beforeRetry: () => Promise.resolve(false),
    });
    expect(result).toBeNull();
    expect(calls).toBe(1);
  });

  it("laisse remonter le dernier conflit et les autres erreurs", async () => {
    const always = () => Promise.reject(CONFLICT);
    await expect(
      retryOnConflict(always, { retries: 2, isConflict, beforeRetry: () => Promise.resolve(true) }),
    ).rejects.toBe(CONFLICT);
    const other = new Error("autre");
    await expect(
      retryOnConflict(() => Promise.reject(other), { retries: 2, isConflict, beforeRetry: () => Promise.resolve(true) }),
    ).rejects.toBe(other);
  });
});
