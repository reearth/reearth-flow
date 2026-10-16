import ReactMarkdown from "react-markdown";
import type { Components } from "react-markdown";
import remarkGfm from "remark-gfm";

import { cn } from "@flow/lib/utils";

// Styled with the theme's own tokens so it reads correctly in every theme.
// Raw HTML in the source is not rendered.
const components: Components = {
  h1: ({ node: _, ...props }) => (
    <h1 className="mt-4 mb-2 border-b pb-1 text-xl font-semibold" {...props} />
  ),
  h2: ({ node: _, ...props }) => (
    <h2 className="mt-4 mb-2 border-b pb-1 text-lg font-semibold" {...props} />
  ),
  h3: ({ node: _, ...props }) => (
    <h3 className="mt-3 mb-1 text-base font-semibold" {...props} />
  ),
  h4: ({ node: _, ...props }) => (
    <h4 className="mt-3 mb-1 text-sm font-semibold" {...props} />
  ),
  p: ({ node: _, ...props }) => <p className="my-2" {...props} />,
  a: ({ node: _, ...props }) => (
    <a
      className="text-logo underline underline-offset-2"
      target="_blank"
      rel="noopener noreferrer"
      {...props}
    />
  ),
  ul: ({ node: _, ...props }) => (
    <ul className="my-2 list-disc pl-6" {...props} />
  ),
  ol: ({ node: _, ...props }) => (
    <ol className="my-2 list-decimal pl-6" {...props} />
  ),
  li: ({ node: _, ...props }) => <li className="my-0.5" {...props} />,
  blockquote: ({ node: _, ...props }) => (
    <blockquote
      className="my-2 border-l-2 pl-3 text-muted-foreground"
      {...props}
    />
  ),
  code: ({ node: _, className, ...props }) => (
    <code
      className={cn("rounded bg-muted px-1 font-mono text-xs", className)}
      {...props}
    />
  ),
  pre: ({ node: _, ...props }) => (
    <pre
      className="my-2 overflow-x-auto rounded-md bg-muted p-3 text-xs [&>code]:bg-transparent [&>code]:p-0"
      {...props}
    />
  ),
  hr: ({ node: _, ...props }) => <hr className="my-4" {...props} />,
  table: ({ node: _, ...props }) => (
    <div className="my-2 overflow-x-auto">
      <table className="w-full border-collapse text-xs" {...props} />
    </div>
  ),
  th: ({ node: _, ...props }) => (
    <th className="border px-2 py-1 text-left font-semibold" {...props} />
  ),
  td: ({ node: _, ...props }) => <td className="border px-2 py-1" {...props} />,
};

type Props = {
  children: string;
  className?: string;
};

const Markdown: React.FC<Props> = ({ children, className }) => (
  <div className={cn("text-sm wrap-break-word", className)}>
    <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
      {children}
    </ReactMarkdown>
  </div>
);

export { Markdown };
