import React from 'react';
import { cn } from '../../lib/utils';

export interface MarkdownViewProps extends React.HTMLAttributes<HTMLDivElement> {
  content: string;
  ref?: React.Ref<HTMLDivElement>;
}

function renderInlineMarkdown(line: string): React.ReactNode[] {
  const tokens = line.split(/(\*\*[^*]+\*\*|\*[^*]+\*|\[[^\]]+\]\([^)]+\))/g);
  return tokens.map((tok, i) => {
    if (tok.startsWith('**') && tok.endsWith('**')) {
      return (
        <strong key={i} className="font-semibold text-primary">
          {tok.slice(2, -2)}
        </strong>
      );
    }
    if (tok.startsWith('*') && tok.endsWith('*')) {
      return (
        <em key={i} className="italic">
          {tok.slice(1, -1)}
        </em>
      );
    }
    const linkMatch = tok.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
    if (linkMatch) {
      const href = linkMatch[2];
      const safeHref =
        href.startsWith('http://') || href.startsWith('https://') || href.startsWith('/')
          ? href
          : '#';
      return (
        <a
          key={i}
          href={safeHref}
          target="_blank"
          rel="noopener noreferrer"
          className="text-brand underline hover:opacity-90"
        >
          {linkMatch[1]}
        </a>
      );
    }
    return tok;
  });
}

export const MarkdownView: React.FC<MarkdownViewProps> = ({
  content,
  className,
  ref,
  ...props
}) => {
  const lines = content.split('\n');

  return (
    <div
      ref={ref}
      className={cn('space-y-2 text-sm text-primary leading-relaxed', className)}
      {...props}
    >
      {lines.map((line, idx) => {
        const trimmed = line.trim();
        if (!trimmed) return <div key={idx} className="h-1" />;
        if (trimmed.startsWith('### ')) {
          return (
            <h4 key={idx} className="font-serif text-base font-semibold text-primary pt-1">
              {renderInlineMarkdown(trimmed.slice(4))}
            </h4>
          );
        }
        if (trimmed.startsWith('## ')) {
          return (
            <h3 key={idx} className="font-serif text-lg font-bold text-primary pt-2">
              {renderInlineMarkdown(trimmed.slice(3))}
            </h3>
          );
        }
        if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
          return (
            <div key={idx} className="flex items-start gap-2 pl-2">
              <span className="text-brand">•</span>
              <span>{renderInlineMarkdown(trimmed.slice(2))}</span>
            </div>
          );
        }
        return <p key={idx}>{renderInlineMarkdown(line)}</p>;
      })}
    </div>
  );
};
