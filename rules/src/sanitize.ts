/**
 * ZRAI sanitization rule layer (TypeScript source of truth).
 *
 * This module scrubs untrusted outbound context before it leaves the device:
 * absolute file paths, PII (emails, IPs), secrets (SSH keys, bearer/api tokens),
 * and git metadata. It is the single source of truth for sanitization rules and is
 * tested independently with `bun test`.
 *
 * The Rust core (`crates/zrai-core`) loads the compiled `dist/sanitize.js` (produced by
 * `bun run build`) via an embedded JS engine and calls `sanitize(messages, opts)`.
 * For that reason this file also assigns the function to `globalThis` so the IIFE build
 * is directly callable by the engine.
 */

export type Role = "system" | "user" | "assistant" | "tool";

export interface TextBlock {
  type: "text";
  text: string;
}

export interface Message {
  role: Role;
  content: string | TextBlock[];
}

export type Category =
  | "path"
  | "email"
  | "ipv4"
  | "ipv6"
  | "ssh_key"
  | "bearer_token"
  | "api_key"
  | "git_metadata"
  | "env_secret";

export interface CustomRule {
  pattern: string;
  replace: string;
  flags?: string;
}

export interface SanitizeOpts {
  /** When true, also scrubs common environment-variable secret assignments. */
  strict?: boolean;
  /** User-supplied replacement rules (applied last, highest priority). */
  rules?: CustomRule[];
  /** Override the redaction marker per category. */
  redact?: Partial<Record<Category, string>>;
}

export interface StrippedItem {
  category: Category;
  count: number;
}

export interface SanitizeResult {
  messages: Message[];
  stripped: StrippedItem[];
}

interface BuiltinRule {
  category: Category;
  regex: RegExp;
  marker: string;
  /**
   * When true, the rule is only applied if `opts.strict` is set. Used for
   * lower-precision patterns (env secrets) that would otherwise over-redact.
   */
  strictOnly?: boolean;
}

// Absolute paths: Unix (/home/..., /Users/..., /var/...), Windows (C:\Users\...).
const PATH_RE = /(?:\/home\/|\/Users\/|\/var\/|\/root\/|[A-Za-z]:\\(?:Users|home)\\|(?:[A-Za-z]:\\))[^\s"'`]*/g;

// Email addresses.
const EMAIL_RE = /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g;

// IPv4 (loose; sufficient for redaction, not validation).
const IPV4_RE = /\b(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)\b/g;

// IPv6 (covers common forms including compressed "::"; not exhaustive).
const IPV6_RE =
  /\b(?:[A-Fa-f0-9]{1,4}:){2,7}[A-Fa-f0-9]{1,4}\b|\b[A-Fa-f0-9]{1,4}:(?::[A-Fa-f0-9]{1,4}){0,6}\b|\b:(?::[A-Fa-f0-9]{1,4}){1,6}\b/g;

// PEM private key blocks (multi-line).
const SSH_KEY_RE = /-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----[\s\S]*?-----END (?:[A-Z ]+ )?PRIVATE KEY-----/g;

// Bearer / API tokens.
const BEARER_RE = /\bBearer\s+[A-Za-z0-9\-._~+/]+=*/g;
const API_KEY_RE = /\b(?:sk|pk|rk|org)-[A-Za-z0-9]{16,}/g; // OpenAI-style etc.
const AWS_KEY_RE = /\bAKIA[0-9A-Z]{16}\b/g;

// Git metadata: ssh remote urls (fully redacted) and commit hashes (keep the "commit " prefix).
const GIT_REMOTE_RE = /\bgit@[a-zA-Z0-9.-]+:[^\s]+/g;
const GIT_COMMIT_RE = /(\bcommit\s+)[0-9a-f]{7,40}\b/g;

// Environment secret assignments (strict only): KEY=VALUE for likely-secret keys.
const ENV_SECRET_RE =
  /\b(?:API_?KEY|SECRET|TOKEN|PASSWORD|PASSWD|PRIVATE_?KEY|ACCESS_?KEY|CLIENT_?SECRET)[A-Z0-9_]*\s*=\s*\S+/gi;

const BUILTIN: BuiltinRule[] = [
  { category: "path", regex: PATH_RE, marker: "<path>" },
  // git metadata runs before email so SSH remotes (git@host:...) are caught as git-refs,
  // not partially redacted as emails.
  { category: "git_metadata", regex: GIT_REMOTE_RE, marker: "<git-ref>" },
  { category: "git_metadata", regex: GIT_COMMIT_RE, marker: "$1<git-ref>" },
  { category: "email", regex: EMAIL_RE, marker: "<email>" },
  { category: "ipv4", regex: IPV4_RE, marker: "<ipv4>" },
  { category: "ipv6", regex: IPV6_RE, marker: "<ipv6>" },
  { category: "ssh_key", regex: SSH_KEY_RE, marker: "<private-key>" },
  { category: "bearer_token", regex: BEARER_RE, marker: "Bearer <token>" },
  { category: "api_key", regex: API_KEY_RE, marker: "<api-key>" },
  { category: "api_key", regex: AWS_KEY_RE, marker: "<aws-key>" },
  { category: "env_secret", regex: ENV_SECRET_RE, marker: "<secret>", strictOnly: true },
];

function redact(text: string, opts: SanitizeOpts): { text: string; counts: Map<Category, number> } {
  const counts = new Map<Category, number>();
  let out = text;

  for (const rule of BUILTIN) {
    if (rule.strictOnly && !opts.strict) continue;
    const marker = opts.redact?.[rule.category] ?? rule.marker;
    // Reset lastIndex because the regexes are global.
    rule.regex.lastIndex = 0;
    const matches = out.match(rule.regex);
    if (matches) counts.set(rule.category, (counts.get(rule.category) ?? 0) + matches.length);
    out = out.replace(rule.regex, marker);
  }

  // User-supplied rules, applied last so they can override built-ins.
  for (const custom of opts.rules ?? []) {
    const re = new RegExp(custom.pattern, custom.flags ?? "g");
    const matches = out.match(re);
    if (matches) {
      counts.set("api_key", (counts.get("api_key") ?? 0) + matches.length);
    }
    out = out.replace(re, custom.replace);
  }

  return { text: out, counts };
}

function blockText(block: TextBlock): string {
  return block.type === "text" ? block.text : "";
}

/**
 * Sanitize a list of chat messages in place (returns a new array).
 * Never throws on malformed input; unrecognized shapes are passed through.
 */
export function sanitize(messages: Message[], opts: SanitizeOpts = {}): SanitizeResult {
  const stripped: StrippedItem[] = [];
  const outMessages = messages.map((m) => {
    if (typeof m.content === "string") {
      const { text, counts } = redact(m.content, opts);
      for (const [category, count] of counts) stripped.push({ category, count });
      return { role: m.role, content: text };
    }
    if (Array.isArray(m.content)) {
      const counts = new Map<Category, number>();
      const blocks = m.content.map((b) => {
        if (b.type !== "text") return b;
        const r = redact(blockText(b), opts);
        for (const [category, count] of r.counts) {
          counts.set(category, (counts.get(category) ?? 0) + count);
        }
        return { type: "text" as const, text: r.text };
      });
      for (const [category, count] of counts) stripped.push({ category, count });
      return { role: m.role, content: blocks };
    }
    return m;
  });

  // Merge duplicate categories for a clean summary.
  const merged = new Map<Category, number>();
  for (const item of stripped) merged.set(item.category, (merged.get(item.category) ?? 0) + item.count);
  const summary = [...merged.entries()].map(([category, count]) => ({ category, count }));

  return { messages: outMessages, stripped: summary };
}

// Expose for the embedded engine (rquickjs) which loads the IIFE build as a script.
(globalThis as unknown as { sanitize: typeof sanitize }).sanitize = sanitize;
