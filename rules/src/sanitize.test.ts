import { describe, it, expect } from "bun:test";
import { sanitize, type Message } from "./sanitize";

const user = (content: string): Message => ({ role: "user", content });

describe("sanitize - absolute paths", () => {
  it("redacts Unix home paths", () => {
    const { messages, stripped } = sanitize([user("see /home/alice/src/secrets.py")]);
    expect(messages[0].content).toBe("see <path>");
    expect(stripped.find((s) => s.category === "path")?.count).toBe(1);
  });

  it("redacts macOS /Users paths", () => {
    const { messages } = sanitize([user("file at /Users/bob/repo/key.pem")]);
    expect(messages[0].content).toBe("file at <path>");
  });

  it("redacts Windows paths", () => {
    const { messages } = sanitize([user("open C:\\Users\\bob\\repo\\key.pem")]);
    expect(messages[0].content).toBe("open <path>");
  });
});

describe("sanitize - PII", () => {
  it("redacts emails", () => {
    const { messages, stripped } = sanitize([user("contact jane@corp.com please")]);
    expect(messages[0].content).toBe("contact <email> please");
    expect(stripped.find((s) => s.category === "email")?.count).toBe(1);
  });

  it("redacts IPv4 and IPv6", () => {
    const { messages, stripped } = sanitize([user("host 10.0.0.5 and fe80::1")]);
    expect(messages[0].content).toBe("host <ipv4> and <ipv6>");
    expect(stripped.find((s) => s.category === "ipv4")?.count).toBe(1);
    expect(stripped.find((s) => s.category === "ipv6")?.count).toBe(1);
  });
});

describe("sanitize - secrets", () => {
  it("redacts PEM private key blocks", () => {
    const block = "-----BEGIN OPENSSH PRIVATE KEY-----\nabc123\n-----END OPENSSH PRIVATE KEY-----";
    const { messages, stripped } = sanitize([user(block)]);
    expect(messages[0].content).toBe("<private-key>");
    expect(stripped.find((s) => s.category === "ssh_key")?.count).toBe(1);
  });

  it("redacts bearer tokens", () => {
    const { messages } = sanitize([user("Authorization: Bearer sk-abcdef1234567890")]);
    expect(messages[0].content).toBe("Authorization: Bearer <token>");
  });

  it("redacts API keys", () => {
    const { messages } = sanitize([user("key sk-abcdefghijklmnopqrstuvwx")]);
    expect(messages[0].content).toBe("key <api-key>");
  });

  it("redacts AWS keys", () => {
    const { messages } = sanitize([user("aws AKIAIOSFODNN7EXAMPLE")]);
    expect(messages[0].content).toBe("aws <aws-key>");
  });
});

describe("sanitize - git metadata", () => {
  it("redacts git ssh remotes and commit hashes", () => {
    const { messages } = sanitize([user("git@github.com:acme/private.git and commit 1a2b3c4d5e6f")]);
    expect(messages[0].content).toBe("<git-ref> and commit <git-ref>");
  });
});

describe("sanitize - env secrets are strict-only", () => {
  it("does not redact env secrets unless strict", () => {
    const { messages } = sanitize([user("API_KEY=supersecretvalue")]);
    expect(messages[0].content).toBe("API_KEY=supersecretvalue");
  });

  it("redacts env secrets when strict", () => {
    const { messages, stripped } = sanitize([user("API_KEY=supersecretvalue")], { strict: true });
    expect(messages[0].content).toBe("<secret>");
    expect(stripped.find((s) => s.category === "env_secret")?.count).toBe(1);
  });
});

describe("sanitize - custom rules", () => {
  it("applies user rules last", () => {
    const { messages } = sanitize([user("proj CODENAME=Orion")], {
      rules: [{ pattern: "CODENAME=(\\w+)", replace: "CODENAME=<redacted>" }],
    });
    expect(messages[0].content).toBe("proj CODENAME=<redacted>");
  });
});

describe("sanitize - message shapes", () => {
  it("handles Anthropic content arrays", () => {
    const msgs: Message[] = [
      { role: "user", content: [{ type: "text", text: "read /home/me/a.txt" }] },
    ];
    const { messages } = sanitize(msgs);
    const content = messages[0].content as { type: "text"; text: string }[];
    expect(content[0].text).toBe("read <path>");
  });

  it("is deterministic (same input -> same output)", () => {
    const a = sanitize([user("mail a@b.com and /home/x/y")]);
    const b = sanitize([user("mail a@b.com and /home/x/y")]);
    expect(JSON.stringify(a)).toBe(JSON.stringify(b));
  });

  it("does not throw on empty input", () => {
    const { messages } = sanitize([]);
    expect(messages).toEqual([]);
  });
});
