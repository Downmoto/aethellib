/**
 * Home page for the aethellib documentation site.
 *
 * Provides a simple entry point that links users to the main Fumadocs docs route.
 */
import Link from 'next/link';

export default function HomePage() {
  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col items-start justify-center px-6 py-16">
      <p className="mb-3 text-sm font-medium text-fd-muted-foreground">aethellib</p>
      <h1 className="mb-4 text-3xl font-bold tracking-tight sm:text-4xl">
        lightweight tooling for aethel workflows
      </h1>
      <p className="mb-8 max-w-2xl text-base text-fd-muted-foreground sm:text-lg">
        This site is built with Fumadocs and contains the guides, references, and examples you
        need to use aethellib effectively.
      </p>

      <div className="flex flex-wrap gap-3">
        <Link
          href="/docs"
          className="rounded-md bg-fd-primary px-4 py-2 text-sm font-medium text-fd-primary-foreground transition-opacity hover:opacity-90"
        >
          Read the docs
        </Link>
        <Link
          href="/docs/test"
          className="rounded-md border border-fd-border px-4 py-2 text-sm font-medium transition-colors hover:bg-fd-accent"
        >
          View example page
        </Link>
      </div>
    </main>
  );
}
